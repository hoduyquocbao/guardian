//! Guardian-Store CLI tool
//! 
//! Provides command-line interface for administrative operations

use clap::{Parser, Subcommand};
use guardian_store::{Store, User, Location, Profile};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "guardian-store")]
#[command(about = "High-performance storage system with architectural elegance")]
struct Cli {
    /// Storage base path
    #[arg(short, long, default_value = "./data")]
    path: PathBuf,
    
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Show system status
    Status,
    
    /// Query a record by ID
    Get {
        /// Record ID
        id: u64,
    },
    
    /// Create a new record
    Create {
        /// Record ID
        id: u64,
        /// User name
        name: String,
        /// Email address
        email: String,
    },
    
    /// Delete a record
    Delete {
        /// Record ID
        id: u64,
    },
    
    /// Trigger compaction
    Compact,
    
    /// Scan all records
    Scan,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    
    // Initialize store
    let mut store = Store::new(&cli.path)?;
    
    match cli.command {
        Commands::Status => {
            let stats = store.stats()?;
            println!("Guardian-Store Status:");
            println!("  Records: {}", stats.records);
            println!("  Segments: {}", stats.segments);
        }
        
        Commands::Get { id } => {
            match store.find(id)? {
                Some(user) => {
                    println!("User ID: {}", user.id);
                    println!("Name: {}", user.name);
                    println!("Email: {}", user.email);
                    println!("Location: {} {}, {}", user.location.street, user.location.city, user.location.country);
                    if let Some(profile) = user.profile {
                        println!("Age: {}", profile.age);
                        println!("Job: {}", profile.job);
                        println!("Interests: {}", profile.interests.join(", "));
                    }
                }
                None => {
                    println!("User with ID {} not found", id);
                }
            }
        }
        
        Commands::Create { id, name, email } => {
            let location = Location {
                street: "Default Street".to_string(),
                city: "Default City".to_string(),
                country: "Default Country".to_string(),
                postal: "00000".to_string(),
            };
            
            let user = User {
                id,
                name,
                email,
                location,
                profile: None,
                created: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)?
                    .as_secs(),
                updated: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)?
                    .as_secs(),
            };
            
            store.save(&user)?;
            println!("User created successfully with ID: {}", id);
        }
        
        Commands::Delete { id } => {
            store.delete(id)?;
            println!("User with ID {} deleted successfully", id);
        }
        
        Commands::Compact => {
            println!("Compaction not yet implemented in CLI");
        }
        
        Commands::Scan => {
            println!("Scanning all records...");
            let mut count = 0;
            for result in store.scan() {
                match result {
                    Ok(user) => {
                        println!("ID: {}, Name: {}, Email: {}", user.id, user.name, user.email);
                        count += 1;
                    }
                    Err(e) => {
                        eprintln!("Error reading record: {}", e);
                    }
                }
            }
            println!("Total records: {}", count);
        }
    }
    
    Ok(())
} 
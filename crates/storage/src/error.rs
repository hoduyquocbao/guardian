//! Error types for Guardian-Store operations
//! 
//! All error types follow the single-word identifier manifesto
//! for maximum clarity and consistency.

use thiserror::Error;

/// Represents all possible errors in Guardian-Store
#[derive(Error, Debug)]
pub enum Error {
    /// Storage operation failed
    #[error("Storage operation failed: {0}")]
    Storage(#[from] std::io::Error),
    
    /// Time operation failed
    #[error("Time operation failed: {0}")]
    Time(#[from] std::time::SystemTimeError),
    
    /// Serialization/deserialization failed
    #[error("Serialization failed: {0}")]
    Serialize(String),
    
    /// Index operation failed
    #[error("Index operation failed: {0}")]
    Index(String),
    
    /// Invalid data format
    #[error("Invalid data format: {0}")]
    Format(String),
    
    /// Resource not found
    #[error("Resource not found: {0}")]
    Missing(String),
    
    /// Operation not supported
    #[error("Operation not supported: {0}")]
    Unsupported(String),
    
    /// System configuration error
    #[error("Configuration error: {0}")]
    Config(String),
    
    /// Compaction operation failed
    #[error("Compaction failed: {0}")]
    Compact(String),
} 
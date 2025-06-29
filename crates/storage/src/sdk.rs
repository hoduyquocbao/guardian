//! High-level SDK for Guardian-Store
//! 
//! Provides a clean abstraction over segment and index operations
//! with zero-copy data access and schema evolution support.

use std::path::Path;
use std::collections::HashMap;
use rkyv::Archive;
use crate::{Error, Result};
use crate::segment::Segment;
use crate::index::{Index, Operation};
use crate::model::User;

/// Main storage interface for Guardian-Store
pub struct Store {
    /// Segment manager
    segment: Segment,
    /// Index manager
    index: Index,
    /// Schema version cache
    schema_cache: HashMap<u64, u32>,
}

impl Store {
    /// Creates a new store instance
    pub fn new<P: AsRef<Path>>(base: P) -> Result<Self> {
        let base = base.as_ref();
        let segment_path = base.join("segments");
        let index_path = base.join("index");
        
        let segment = Segment::new(segment_path)?;
        let index = Index::new(index_path)?;
        
        Ok(Self {
            segment,
            index,
            schema_cache: HashMap::new(),
        })
    }
    
    /// Saves a user to storage
    pub fn save(&mut self, user: &User) -> Result<()> {
        // Append to segment
        let position = self.segment.append(user)?;
        
        // Update index
        let key = user.id.to_le_bytes();
        self.index.put(&key, position)?;
        
        Ok(())
    }
    
    /// Finds a user by ID and deserializes to owned value
    pub fn find(&self, id: u64) -> Result<Option<User>> {
        let key = id.to_le_bytes();
        
        // Look up position in index
        let position = match self.index.get(&key)? {
            Some(pos) => pos,
            None => return Ok(None),
        };
        
        // Read and deserialize from segment
        let user = self.segment.read::<User>(position)?;
        Ok(Some(user))
    }
    
    /// Deletes a user by ID
    pub fn delete(&mut self, id: u64) -> Result<()> {
        let key = id.to_le_bytes();
        self.index.delete(&key)?;
        Ok(())
    }
    
    /// Updates a user (delete + save)
    pub fn update(&mut self, user: &User) -> Result<()> {
        self.delete(user.id)?;
        self.save(user)?;
        Ok(())
    }
    
    /// Performs batch save operations
    pub fn batch(&mut self, users: &[User]) -> Result<()> {
        let mut operations = Vec::with_capacity(users.len());
        
        for user in users {
            let position = self.segment.append(user)?;
            let key = user.id.to_le_bytes();
            
            operations.push(Operation::Put {
                key: key.to_vec(),
                position,
            });
        }
        
        self.index.batch(operations)?;
        Ok(())
    }
    
    /// Scans all users in the store
    pub fn scan(&self) -> impl Iterator<Item = Result<User>> + '_ {
        self.index.scan().map(|result| {
            result.and_then(|(key, position)| {
                // Convert key back to ID
                if key.len() != 8 {
                    return Err(Error::Format("Invalid key length".to_string()));
                }
                
                let _id = u64::from_le_bytes(key.try_into().unwrap());
                
                // Read user data
                let user = self.segment.read::<User>(position)?;
                Ok(user)
            })
        })
    }
    
    /// Gets storage statistics
    pub fn stats(&self) -> Result<Stats> {
        let mut total = 0u64;
        let segments = 0u64;
        
        // Count records and segments
        for result in self.index.scan() {
            result?;
            total += 1;
        }
        
        // TODO: Implement segment counting from filesystem
        
        Ok(Stats {
            records: total,
            segments,
        })
    }
    
    /// Migrates data to a new schema version
    pub fn migrate(&self, _target_schema: u32) -> Result<()> {
        // TODO: Implement schema migration logic
        // This would involve:
        // 1. Reading all records
        // 2. Converting to new schema
        // 3. Writing back with new schema version
        // 4. Updating metadata
        
        Err(Error::Unsupported("Schema migration not yet implemented".to_string()))
    }
}

/// Storage statistics
#[derive(Debug, Clone)]
pub struct Stats {
    /// Total number of records
    pub records: u64,
    /// Total number of segments
    pub segments: u64,
}

impl Drop for Store {
    fn drop(&mut self) {
        // Resources will be cleaned up automatically
    }
} 
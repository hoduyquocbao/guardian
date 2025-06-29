//! Custom index management using binary format
//! 
//! Provides fast key-value lookups using custom binary layout
//! without external dependencies.

use std::collections::HashMap;
use std::path::Path;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write, Seek, SeekFrom};
use crate::{Error, Result};
use crate::model::Position;

/// Binary entry structure for index
#[derive(Debug, Clone)]
struct Entry {
    key_len: u32,
    key: Vec<u8>,
    segment: u64,
    offset: u64,
    length: u64,
}

impl Entry {
    fn new(key: &[u8], position: Position) -> Self {
        Self {
            key_len: key.len() as u32,
            key: key.to_vec(),
            segment: position.segment,
            offset: position.offset,
            length: position.length,
        }
    }
    
    fn unpack(data: &[u8]) -> Result<Self> {
        if data.len() < 29 { // minimum size: 1 + 4 + 8 + 8 + 8
            return Err(Error::Format("Entry data too short".to_string()));
        }
        
        let version = data[0];
        if version != 1 {
            return Err(Error::Format("Unsupported entry version".to_string()));
        }
        
        let key_len = u32::from_le_bytes(data[1..5].try_into().unwrap());
        if data.len() < (5 + key_len + 24) as usize {
            return Err(Error::Format("Entry data incomplete".to_string()));
        }
        
        let key_start = 5;
        let key_end = key_start + key_len as usize;
        let key = data[key_start..key_end].to_vec();
        
        let pos_start = key_end;
        let segment = u64::from_le_bytes(data[pos_start..pos_start+8].try_into().unwrap());
        let offset = u64::from_le_bytes(data[pos_start+8..pos_start+16].try_into().unwrap());
        let length = u64::from_le_bytes(data[pos_start+16..pos_start+24].try_into().unwrap());
        
        Ok(Self {
            key_len,
            key,
            segment,
            offset,
            length,
        })
    }
    
    fn pack(&self) -> Vec<u8> {
        let mut data = Vec::new();
        
        // Version
        data.push(1);
        
        // Key length
        data.extend_from_slice(&self.key_len.to_le_bytes());
        
        // Key data
        data.extend_from_slice(&self.key);
        
        // Position data
        data.extend_from_slice(&self.segment.to_le_bytes());
        data.extend_from_slice(&self.offset.to_le_bytes());
        data.extend_from_slice(&self.length.to_le_bytes());
        
        data
    }
}

/// Manages index operations using custom binary format
pub struct Index {
    /// In-memory index cache
    cache: HashMap<Vec<u8>, Position>,
    /// Index file path
    path: std::path::PathBuf,
    /// File handle
    file: Option<File>,
}

impl Index {
    /// Creates a new index manager
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        std::fs::create_dir_all(path.parent().unwrap())?;
        
        let mut index = Self {
            cache: HashMap::new(),
            path,
            file: None,
        };
        
        // Load existing index data
        index.load()?;
        
        Ok(index)
    }
    
    /// Stores a key-position mapping
    pub fn put(&mut self, key: &[u8], position: Position) -> Result<()> {
        let mut file = self.open()?;
        
        // Create entry
        let entry = Entry::new(key, position);
        let entry_data = entry.pack();
        
        // Write entry length and data
        file.write_all(&(entry_data.len() as u32).to_le_bytes())?;
        file.write_all(&entry_data)?;
        file.flush()?;
        
        // Update cache
        self.cache.insert(key.to_vec(), position);
        
        Ok(())
    }
    
    /// Retrieves a position for a given key
    pub fn get(&self, key: &[u8]) -> Result<Option<Position>> {
        // Check cache first
        if let Some(position) = self.cache.get(key) {
            return Ok(Some(*position));
        }
        
        // Search in file
        if let Some(file) = &self.file {
            let mut file = file.try_clone()?;
            file.seek(SeekFrom::Start(0))?;
            
            while let Ok(entry_len) = self.read_u32(&mut file) {
                let mut entry_data = vec![0u8; entry_len as usize];
                file.read_exact(&mut entry_data)?;
                
                // Parse entry
                let entry = Entry::unpack(&entry_data)?;
                
                if entry.key == key {
                    let position = Position {
                        segment: entry.segment,
                        offset: entry.offset,
                        length: entry.length,
                    };
                    return Ok(Some(position));
                }
            }
        }
        
        Ok(None)
    }
    
    /// Removes a key-position mapping
    pub fn delete(&mut self, key: &[u8]) -> Result<()> {
        // Remove from cache
        self.cache.remove(key);
        
        // TODO: Implement file-based deletion
        // This would require rewriting the index file without the deleted entry
        
        Ok(())
    }
    
    /// Performs batch operations for better performance
    pub fn batch(&mut self, operations: Vec<Operation>) -> Result<()> {
        let mut file = self.open()?;
        
        for op in operations {
            match op {
                Operation::Put { key, position } => {
                    let entry = Entry::new(&key, position);
                    let entry_data = entry.pack();
                    file.write_all(&(entry_data.len() as u32).to_le_bytes())?;
                    file.write_all(&entry_data)?;
                }
                Operation::Delete { key } => {
                    // TODO: Implement batch deletion
                    self.cache.remove(&key);
                }
            }
        }
        
        file.flush()?;
        Ok(())
    }
    
    /// Iterates over all key-position pairs
    pub fn scan(&self) -> impl Iterator<Item = Result<(Vec<u8>, Position)>> + '_ {
        let cache = &self.cache;
        cache.iter().map(|(key, position)| {
            Ok((key.clone(), *position))
        })
    }
    
    /// Ensures the index file is open and ready for writing
    fn open(&self) -> Result<File> {
        if let Some(file) = &self.file {
            Ok(file.try_clone()?)
        } else {
            let file = OpenOptions::new()
                .create(true)
                .write(true)
                .append(true)
                .open(&self.path)?;
            Ok(file)
        }
    }
    
    /// Loads existing index data into memory
    fn load(&mut self) -> Result<()> {
        if !self.path.exists() {
            return Ok(());
        }
        
        let mut file = OpenOptions::new()
            .read(true)
            .open(&self.path)?;
        
        file.seek(SeekFrom::Start(0))?;
        
        while let Ok(entry_len) = self.read_u32(&mut file) {
            let mut entry_data = vec![0u8; entry_len as usize];
            file.read_exact(&mut entry_data)?;
            
            let entry = Entry::unpack(&entry_data)?;
            let position = Position {
                segment: entry.segment,
                offset: entry.offset,
                length: entry.length,
            };
            
            self.cache.insert(entry.key, position);
        }
        
        // Keep file open for future operations
        self.file = Some(file);
        
        Ok(())
    }
    
    /// Reads a u32 from file
    fn read_u32(&self, file: &mut File) -> Result<u32> {
        let mut buf = [0u8; 4];
        file.read_exact(&mut buf)?;
        Ok(u32::from_le_bytes(buf))
    }
}

/// Index operation types
pub enum Operation {
    /// Put operation
    Put {
        /// Key bytes
        key: Vec<u8>,
        /// Position in storage
        position: Position,
    },
    /// Delete operation
    Delete {
        /// Key bytes
        key: Vec<u8>,
    },
}

impl Drop for Index {
    fn drop(&mut self) {
        // File will be closed automatically
    }
} 
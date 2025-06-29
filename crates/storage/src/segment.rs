//! Segment-based storage management
//! 
//! Handles immutable segment files for efficient data storage
//! with automatic segment rotation when size limits are reached.

use std::fs::{File, OpenOptions};
use std::io::{Read, Write, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use rkyv::{to_bytes, Archive, Deserialize, Infallible};
use crate::{Error, Result};
use crate::model::{Position, Header, Metadata};

/// Magic number for segment file validation
const MAGIC: u32 = 0x47535452; // "GSTR"

/// Maximum segment size in bytes (256MB)
const MAXSIZE: u64 = 256 * 1024 * 1024;

/// Manages segment-based storage operations
pub struct Segment {
    /// Base directory for segment files
    base: PathBuf,
    /// Current active segment ID
    current: Arc<Mutex<u64>>,
    /// Current segment file handle
    file: Arc<Mutex<Option<File>>>,
    /// Current segment metadata
    metadata: Arc<Mutex<Metadata>>,
}

impl Segment {
    /// Creates a new segment manager
    pub fn new<P: AsRef<Path>>(base: P) -> Result<Self> {
        let base = base.as_ref().to_path_buf();
        std::fs::create_dir_all(&base)?;
        
        let current = Self::find_next(&base)?;
        let metadata = Metadata {
            id: current,
            created: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_secs(),
            records: 0,
            bytes: 0,
            schema: 1,
        };
        
        Ok(Self {
            base,
            current: Arc::new(Mutex::new(current)),
            file: Arc::new(Mutex::new(None)),
            metadata: Arc::new(Mutex::new(metadata)),
        })
    }
    
    /// Appends data to the current segment
    pub fn append<T>(&self, data: &T) -> Result<Position>
    where
        T: rkyv::Serialize<rkyv::ser::serializers::AllocSerializer<1024>>,
    {
        let mut file = self.open()?;
        let mut metadata = self.metadata.lock().unwrap();
        
        // Check if we need to rotate to a new segment
        if metadata.bytes >= MAXSIZE {
            self.rotate()?;
            file = self.open()?;
            metadata = self.metadata.lock().unwrap();
        }
        
        // Serialize data
        let bytes = to_bytes::<_, 1024>(data)
            .map_err(|e| Error::Serialize(format!("Serialization failed: {:?}", e)))?;
        
        // Get current position
        let offset = file.seek(SeekFrom::End(0))?;
        
        // Write data length and data
        file.write_all(&(bytes.len() as u32).to_le_bytes())?;
        file.write_all(&bytes)?;
        file.flush()?;
        
        // Update metadata
        metadata.records += 1;
        metadata.bytes = file.seek(SeekFrom::End(0))?;
        
        Ok(Position {
            segment: metadata.id,
            offset,
            length: bytes.len() as u64,
        })
    }
    
    /// Reads data from a specific position
    pub fn read<T>(&self, position: Position) -> Result<T>
    where
        T: Archive,
        T::Archived: Deserialize<T, Infallible>,
    {
        let segment_path = self.base.join(format!("segment_{}.dat", position.segment));
        let mut file = File::open(segment_path)?;
        
        // Seek to position
        file.seek(SeekFrom::Start(position.offset))?;
        
        // Read length
        let mut length_bytes = [0u8; 4];
        file.read_exact(&mut length_bytes)?;
        let length = u32::from_le_bytes(length_bytes) as usize;
        
        // Read data
        let mut data = vec![0u8; length];
        file.read_exact(&mut data)?;
        
        // Deserialize using unsafe method for now
        unsafe {
            let archived = rkyv::archived_root::<T>(&data);
            let value = archived.deserialize(&mut Infallible)
                .map_err(|e| Error::Serialize(format!("Deserialization error: {:?}", e)))?;
            Ok(value)
        }
    }
    
    /// Ensures the current segment file is open
    fn open(&self) -> Result<File> {
        let mut file_guard = self.file.lock().unwrap();
        
        if file_guard.is_none() {
            let current = *self.current.lock().unwrap();
            let path = self.base.join(format!("segment_{}.dat", current));
            
            let mut file = OpenOptions::new()
                .create(true)
                .write(true)
                .read(true)
                .open(&path)?;
            
            // Write header if file is new
            if file.metadata()?.len() == 0 {
                let metadata = self.metadata.lock().unwrap();
                let header = Header {
                    magic: MAGIC,
                    metadata: metadata.clone(),
                    checksum: 0, // TODO: Implement checksum calculation
                };
                
                let header_bytes = to_bytes::<_, 1024>(&header)
                    .map_err(|e| Error::Serialize(format!("Header serialization failed: {:?}", e)))?;
                
                file.write_all(&(header_bytes.len() as u32).to_le_bytes())?;
                file.write_all(&header_bytes)?;
            }
            
            *file_guard = Some(file);
        }
        
        Ok(file_guard.as_mut().unwrap().try_clone()?)
    }
    
    /// Rotates to a new segment
    fn rotate(&self) -> Result<()> {
        // Close current file
        {
            let mut file_guard = self.file.lock().unwrap();
            *file_guard = None;
        }
        
        // Increment segment ID
        let mut current_guard = self.current.lock().unwrap();
        *current_guard += 1;
        
        // Update metadata
        let mut metadata_guard = self.metadata.lock().unwrap();
        metadata_guard.id = *current_guard;
        metadata_guard.created = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs();
        metadata_guard.records = 0;
        metadata_guard.bytes = 0;
        
        Ok(())
    }
    
    /// Finds the next available segment ID
    fn find_next(base: &Path) -> Result<u64> {
        let mut max_id = 0u64;
        
        if base.exists() {
            for entry in std::fs::read_dir(base)? {
                let entry = entry?;
                let name = entry.file_name();
                let name_str = name.to_string_lossy();
                
                if name_str.starts_with("segment_") && name_str.ends_with(".dat") {
                    if let Some(id_str) = name_str.strip_prefix("segment_").and_then(|s| s.strip_suffix(".dat")) {
                        if let Ok(id) = id_str.parse::<u64>() {
                            max_id = max_id.max(id);
                        }
                    }
                }
            }
        }
        
        Ok(max_id + 1)
    }
} 
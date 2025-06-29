//! Data compaction service
//! 
//! Handles minor and major compaction operations to optimize
//! storage efficiency and remove deleted records.

use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::time::sleep;
use crate::{Error, Result};
use crate::segment::Segment;
use crate::index::{Index, Operation};
use crate::model::{User, Position};

/// Compaction service configuration
#[derive(Debug, Clone)]
pub struct Config {
    /// Maximum segment size before compaction
    pub max_segment_size: u64,
    /// Compaction threshold (percentage of deleted records)
    pub threshold: f64,
    /// Compaction interval
    pub interval: Duration,
    /// Enable throttling based on system load
    pub throttle: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            max_segment_size: 256 * 1024 * 1024, // 256MB
            threshold: 0.3, // 30% deleted records
            interval: Duration::from_secs(3600), // 1 hour
            throttle: true,
        }
    }
}

/// Compaction service state
#[derive(Debug)]
pub struct State {
    /// Current compaction status
    pub status: Status,
    /// Last compaction timestamp
    pub last_compaction: u64,
    /// Total records processed
    pub processed: u64,
    /// Total records removed
    pub removed: u64,
}

/// Compaction status
#[derive(Debug, Clone)]
pub enum Status {
    /// Idle state
    Idle,
    /// Minor compaction in progress
    Minor,
    /// Major compaction in progress
    Major,
    /// Error state
    Error(String),
}

/// Manages data compaction operations
pub struct Compaction {
    /// Compaction configuration
    config: Config,
    /// Current state
    state: Arc<Mutex<State>>,
    /// Segment manager
    segment: Arc<Segment>,
    /// Index manager
    index: Arc<Mutex<Index>>,
    /// Base storage path
    base_path: String,
}

impl Compaction {
    /// Creates a new compaction service
    pub fn new(
        config: Config,
        segment: Arc<Segment>,
        index: Arc<Mutex<Index>>,
        base_path: String,
    ) -> Self {
        let state = State {
            status: Status::Idle,
            last_compaction: 0,
            processed: 0,
            removed: 0,
        };
        
        Self {
            config,
            state: Arc::new(Mutex::new(state)),
            segment,
            index,
            base_path,
        }
    }
    
    /// Starts the compaction service
    pub async fn start(&self) -> Result<()> {
        let config = self.config.clone();
        let state = Arc::clone(&self.state);
        let segment = Arc::clone(&self.segment);
        let index = Arc::clone(&self.index);
        let base_path = self.base_path.clone();
        
        tokio::spawn(async move {
            loop {
                // Check if compaction is needed
                if let Err(e) = Self::check_and_compact(
                    &config,
                    &state,
                    &segment,
                    &index,
                    &base_path,
                ).await {
                    tracing::error!("Compaction error: {}", e);
                    
                    let mut state_guard = state.lock().await;
                    state_guard.status = Status::Error(e.to_string());
                }
                
                // Wait for next interval
                sleep(config.interval).await;
            }
        });
        
        Ok(())
    }
    
    /// Checks if compaction is needed and performs it
    async fn check_and_compact(
        config: &Config,
        state: &Arc<Mutex<State>>,
        segment: &Arc<Segment>,
        index: &Arc<Mutex<Index>>,
        base_path: &str,
    ) -> Result<()> {
        let mut state_guard = state.lock().await;
        state_guard.status = Status::Minor;
        
        // Perform minor compaction
        let (processed, removed) = Self::minor_compact(segment, index).await?;
        
        state_guard.processed += processed;
        state_guard.removed += removed;
        state_guard.last_compaction = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs();
        
        // Check if major compaction is needed
        let deletion_ratio = if processed > 0 {
            removed as f64 / processed as f64
        } else {
            0.0
        };
        
        if deletion_ratio >= config.threshold {
            state_guard.status = Status::Major;
            drop(state_guard);
            
            let (processed, removed) = Self::major_compact(segment, index, base_path).await?;
            
            let mut state_guard = state.lock().await;
            state_guard.processed += processed;
            state_guard.removed += removed;
            state_guard.status = Status::Idle;
        } else {
            state_guard.status = Status::Idle;
        }
        
        Ok(())
    }
    
    /// Performs minor compaction (removes deleted records from active segment)
    async fn minor_compact(
        segment: &Arc<Segment>,
        index: &Arc<Mutex<Index>>,
    ) -> Result<(u64, u64)> {
        let mut processed = 0u64;
        let mut removed = 0u64;
        let mut to_delete = Vec::new();
        // Thu thập key cần xóa
        {
            let index_guard = index.lock().await;
            for result in index_guard.scan() {
                let (key, position) = result?;
                processed += 1;
                if segment.read::<User>(position).is_err() {
                    to_delete.push(key);
                }
            }
        }
        // Xóa ngoài scope của index_guard
        if !to_delete.is_empty() {
            let mut index_guard = index.lock().await;
            for key in to_delete {
                index_guard.delete(&key)?;
                removed += 1;
            }
        }
        Ok((processed, removed))
    }
    
    /// Performs major compaction (rewrites segments to remove deleted records)
    async fn major_compact(
        segment: &Arc<Segment>,
        index: &Arc<Mutex<Index>>,
        base_path: &str,
    ) -> Result<(u64, u64)> {
        let mut processed = 0u64;
        let mut removed = 0u64;
        
        // Create temporary storage for new segments
        let temp_path = format!("{}/temp_compact", base_path);
        let temp_segment = Arc::new(Segment::new(&temp_path)?);
        let temp_index = Arc::new(Mutex::new(Index::new(format!("{}/temp_index", temp_path))?));
        
        // Scan all valid records
        let index_guard = index.lock().await;
        for result in index_guard.scan() {
            let (key, position) = result?;
            processed += 1;
            
            // Try to read the record
            match segment.read::<User>(position) {
                Ok(user) => {
                    // Record is valid, write to new segment
                    let new_position = temp_segment.append(&user)?;
                    let mut temp_index_guard = temp_index.lock().await;
                    temp_index_guard.put(&key, new_position)?;
                }
                Err(_) => {
                    // Record is deleted, skip it
                    removed += 1;
                }
            }
        }
        
        // TODO: Implement atomic replacement of old segments with new ones
        // This would involve:
        // 1. Creating backup of current segments
        // 2. Moving temp segments to main location
        // 3. Updating index references
        // 4. Cleaning up old segments
        
        Ok((processed, removed))
    }
    
    /// Gets current compaction state
    pub async fn state(&self) -> State {
        self.state.lock().await.clone()
    }
    
    /// Triggers manual compaction
    pub async fn trigger(&self) -> Result<()> {
        let config = self.config.clone();
        let state = Arc::clone(&self.state);
        let segment = Arc::clone(&self.segment);
        let index = Arc::clone(&self.index);
        let base_path = self.base_path.clone();
        
        Self::check_and_compact(&config, &state, &segment, &index, &base_path).await
    }
}

impl Clone for State {
    fn clone(&self) -> Self {
        Self {
            status: self.status.clone(),
            last_compaction: self.last_compaction,
            processed: self.processed,
            removed: self.removed,
        }
    }
} 
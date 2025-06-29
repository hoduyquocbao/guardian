//! Guardian-Store: High-performance storage system with architectural elegance
//! 
//! This library provides a sophisticated storage solution that balances
//! architectural elegance with performance at scale. It follows the
//! single-word identifier manifesto for maximum clarity and maintainability.

pub mod model;
pub mod segment;
pub mod index;
pub mod sdk;
pub mod compaction;
pub mod error;

pub use error::Error;
pub use sdk::Store;

/// Result type for Guardian-Store operations
pub type Result<T> = std::result::Result<T, Error>;

/// Re-export commonly used types
pub use model::{User, Location, Profile, Position}; 
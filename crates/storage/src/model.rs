//! Data models for Guardian-Store
//! 
//! All structs follow the single-word identifier manifesto and
//! support zero-copy serialization with rkyv.

use rkyv::{Archive, Serialize, Deserialize};

/// Represents a user's geographical location.
/// Original concept: "User Address"
#[derive(Archive, Serialize, Deserialize, Debug, Clone)]
pub struct Location {
    /// Street address
    pub street: String,
    /// City name
    pub city: String,
    /// Country code
    pub country: String,
    /// Postal code
    pub postal: String,
}

/// Represents user profile information.
/// Original concept: "User Profile"
#[derive(Archive, Serialize, Deserialize, Debug, Clone)]
pub struct Profile {
    /// User's age
    pub age: u32,
    /// User's occupation
    pub job: String,
    /// User's interests
    pub interests: Vec<String>,
}

/// Represents a system user entity.
/// Original concept: "User Account"
#[derive(Archive, Serialize, Deserialize, Debug, Clone)]
pub struct User {
    /// Unique user identifier
    pub id: u64,
    /// User's display name
    pub name: String,
    /// User's email address
    pub email: String,
    /// User's geographical location
    pub location: Location,
    /// User's profile information (optional for schema evolution)
    pub profile: Option<Profile>,
    /// Account creation timestamp
    pub created: u64,
    /// Last update timestamp
    pub updated: u64,
}

/// Represents a data record position in storage.
/// Original concept: "Storage Location"
#[derive(Archive, Serialize, Deserialize, Debug, Clone, Copy)]
pub struct Position {
    /// Segment identifier
    pub segment: u64,
    /// Byte offset within segment
    pub offset: u64,
    /// Record length in bytes
    pub length: u64,
}

/// Represents metadata for a storage segment.
/// Original concept: "Segment Metadata"
#[derive(Archive, Serialize, Deserialize, Debug, Clone)]
pub struct Metadata {
    /// Segment identifier
    pub id: u64,
    /// Segment creation timestamp
    pub created: u64,
    /// Total records in segment
    pub records: u64,
    /// Total bytes used
    pub bytes: u64,
    /// Schema version for this segment
    pub schema: u32,
}

/// Represents a storage segment header.
/// Original concept: "Segment Header"
#[derive(Archive, Serialize, Deserialize, Debug, Clone)]
pub struct Header {
    /// Magic number for validation
    pub magic: u32,
    /// Segment metadata
    pub metadata: Metadata,
    /// Checksum for integrity
    pub checksum: u64,
}

impl Default for Profile {
    fn default() -> Self {
        Self {
            age: 0,
            job: String::new(),
            interests: Vec::new(),
        }
    }
}

impl Default for Position {
    fn default() -> Self {
        Self {
            segment: 0,
            offset: 0,
            length: 0,
        }
    }
} 
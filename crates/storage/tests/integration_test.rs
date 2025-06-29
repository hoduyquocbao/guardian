//! Integration tests for Guardian-Store
//! 
//! Tests the complete flow from SDK -> Index -> Segment

use guardian_store::{Store, User, Location, Profile, Result};
use tempfile::TempDir;

/// Creates a test user with sample data
fn create_test_user(id: u64) -> User {
    let location = Location {
        street: format!("{} Test Street", id),
        city: "Test City".to_string(),
        country: "Test Country".to_string(),
        postal: "12345".to_string(),
    };
    
    User {
        id,
        name: format!("User {}", id),
        email: format!("user{}@test.com", id),
        location,
        profile: None,
        created: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        updated: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
    }
}

#[test]
fn test_basic_crud() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let mut store = Store::new(temp_dir.path())?;
    
    let user = create_test_user(1);
    store.save(&user)?;
    
    let retrieved = store.find(1)?.expect("User should exist");
    assert_eq!(retrieved.id, user.id);
    
    store.delete(1)?;
    let retrieved = store.find(1)?;
    assert!(retrieved.is_none());
    
    Ok(())
}

#[test]
fn test_batch_operations() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let mut store = Store::new(temp_dir.path())?;
    
    // Create multiple users
    let users: Vec<User> = (1..=10).map(create_test_user).collect();
    
    // Save all users in batch
    store.batch_save(&users)?;
    
    // Verify all users exist
    for user in &users {
        let retrieved = store.find(user.id)?.expect("User should exist");
        assert_eq!(retrieved.id, user.id);
        assert_eq!(retrieved.name, user.name);
    }
    
    Ok(())
}

#[test]
fn test_zero_copy_access() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let store = Store::new(temp_dir.path())?;
    
    // Create and save a user
    let user = create_test_user(1);
    store.save(&user)?;
    
    // Retrieve with zero-copy access
    let archived = store.find(1)?.expect("User should exist");
    
    // Access fields without deserialization
    assert_eq!(archived.id, user.id);
    assert_eq!(archived.name.as_str(), user.name);
    assert_eq!(archived.email.as_str(), user.email);
    
    Ok(())
}

#[test]
fn test_scan_operations() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let store = Store::new(temp_dir.path())?;
    
    // Create multiple users
    let users: Vec<User> = (1..=5).map(create_test_user).collect();
    
    // Save all users
    for user in &users {
        store.save(user)?;
    }
    
    // Scan all users
    let mut scanned_users: Vec<User> = store.scan().collect::<Result<Vec<_>>>()?;
    scanned_users.sort_by_key(|u| u.id);
    
    // Verify all users are found
    assert_eq!(scanned_users.len(), users.len());
    for (original, scanned) in users.iter().zip(scanned_users.iter()) {
        assert_eq!(original.id, scanned.id);
        assert_eq!(original.name, scanned.name);
    }
    
    Ok(())
}

#[test]
fn test_storage_statistics() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let mut store = Store::new(temp_dir.path())?;
    
    // Initially should have no records
    let stats = store.stats()?;
    assert_eq!(stats.records, 0);
    
    // Add some users
    let users: Vec<User> = (1..=3).map(create_test_user).collect();
    for user in &users {
        store.save(user)?;
    }
    
    // Check updated statistics
    let stats = store.stats()?;
    assert_eq!(stats.records, 3);
    
    // Delete one user
    store.delete(1)?;
    
    // Check statistics after deletion
    let stats = store.stats()?;
    assert_eq!(stats.records, 2);
    
    Ok(())
}

#[test]
fn test_schema_evolution() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let mut store = Store::new(temp_dir.path())?;
    
    // Create a user without profile (old schema)
    let mut user = create_test_user(1);
    user.profile = None;
    store.save(&user)?;
    
    // Verify user can be read
    let retrieved = store.find(1)?.expect("User should exist");
    assert_eq!(retrieved.id, user.id);
    assert!(retrieved.profile.is_none());
    
    // Update user with profile (new schema)
    let mut updated_user = user.clone();
    updated_user.profile = Some(Profile {
        age: 30,
        job: "Senior Engineer".to_string(),
        interests: vec!["Architecture".to_string()],
    });
    store.update(&updated_user)?;
    
    // Verify updated user
    let retrieved = store.find(1)?.expect("User should exist");
    assert_eq!(retrieved.id, user.id);
    assert!(retrieved.profile.is_some());
    assert_eq!(retrieved.profile.as_ref().unwrap().age, 30);
    
    Ok(())
} 
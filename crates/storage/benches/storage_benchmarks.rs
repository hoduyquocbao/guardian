//! Performance benchmarks for Guardian-Store

use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use guardian_store::{Store, User, Location};
use tempfile::TempDir;

fn create_benchmark_user(id: u64) -> User {
    let location = Location {
        street: format!("{} Benchmark Street", id),
        city: "Benchmark City".to_string(),
        country: "Benchmark Country".to_string(),
        postal: "54321".to_string(),
    };
    
    User {
        id,
        name: format!("Benchmark User {}", id),
        email: format!("benchmark{}@test.com", id),
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

fn benchmark_write(c: &mut Criterion) {
    let mut group = c.benchmark_group("write_operations");
    
    for size in [10, 100, 1000].iter() {
        group.bench_with_input(BenchmarkId::new("single_write", size), size, |b, &size| {
            b.iter(|| {
                let temp_dir = TempDir::new().unwrap();
                let store = Store::new(temp_dir.path()).unwrap();
                let user = create_benchmark_user(size);
                store.save(&user).unwrap();
            });
        });
    }
    
    group.finish();
}

fn benchmark_read(c: &mut Criterion) {
    let mut group = c.benchmark_group("read_operations");
    
    for size in [10, 100, 1000].iter() {
        group.bench_with_input(BenchmarkId::new("single_read", size), size, |b, &size| {
            let temp_dir = TempDir::new().unwrap();
            let store = Store::new(temp_dir.path()).unwrap();
            let user = create_benchmark_user(size);
            store.save(&user).unwrap();
            
            b.iter(|| {
                let _retrieved = store.find(size).unwrap();
            });
        });
    }
    
    group.finish();
}

fn benchmark_batch_write(c: &mut Criterion) {
    let mut group = c.benchmark_group("batch_operations");
    
    for size in [10, 100, 1000].iter() {
        group.bench_with_input(BenchmarkId::new("batch_write", size), size, |b, &size| {
            b.iter(|| {
                let temp_dir = TempDir::new().unwrap();
                let mut store = Store::new(temp_dir.path()).unwrap();
                let users: Vec<User> = (0..size).map(create_benchmark_user).collect();
                store.batch_save(&users).unwrap();
            });
        });
    }
    
    group.finish();
}

criterion_group!(benches, benchmark_write, benchmark_read, benchmark_batch_write);
criterion_main!(benches); 
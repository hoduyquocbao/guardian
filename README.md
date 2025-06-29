# Guardian-Store

High-performance storage system with architectural elegance, built on the single-word identifier philosophy.

## Architecture

Guardian-Store is organized as a Rust workspace with multiple crates:

- **`crates/storage`**: Main storage system with segment-based storage, custom binary index, and zero-copy serialization
- **`crates/guardian-macros`**: Procedural macros for defining binary layouts with the `#[frame]` attribute

## Key Features

### Storage System
- **Segment-based storage**: Immutable segment files for efficient data management
- **Custom binary index**: High-performance key-value lookups without external dependencies
- **Zero-copy serialization**: Using rkyv for maximum performance
- **Async compaction**: Background compaction for optimal storage efficiency
- **Schema evolution**: Support for evolving data models over time

### Proc-Macro System
- **`#[frame]` attribute**: Declarative binary layout definition
- **Single-word philosophy**: All identifiers follow the one-word rule for maximum clarity
- **Type safety**: Compile-time validation of binary layouts
- **Endianness control**: Configurable byte ordering per field

## Single-Word Identifier Philosophy

This project follows a strict single-word identifier philosophy:

- All structs, enums, functions, and variables use exactly one English word
- Compound concepts are broken down into simpler, atomic components
- The vocabulary is standardized across the entire codebase
- This approach reduces cognitive load and improves code clarity

### Vocabulary Standardization

Key terms are defined in `vocabulary.csv`:
- `Layout`: Binary layout specification (was FrameDefinition)
- `Field`: Individual field within a layout (was FieldDefinition)
- `Kind`: Field type classification (was FieldType)
- `Position`: Storage location information (was StorageLocation)
- `Segment`: Immutable storage file unit (was StorageSegment)

## Project Structure

```
agents/
├── Cargo.toml                 # Workspace manifest
├── crates/
│   ├── storage/              # Main storage system
│   │   ├── src/
│   │   ├── benches/
│   │   ├── tests/
│   │   └── Cargo.toml
│   └── guardian-macros/      # Proc-macro system
│       ├── src/
│       ├── tests/
│       └── Cargo.toml
├── memories.csv              # Architectural decisions and context
├── todo.csv                  # Task management
├── decisions.csv             # Design decisions and rationale
└── vocabulary.csv            # Standardized vocabulary
```

## Development Status

### Completed
- ✅ Workspace restructuring with multiple crates
- ✅ Custom binary index implementation (replacing RocksDB)
- ✅ Segment-based storage with rkyv serialization
- ✅ Proc-macro architecture with single-word identifiers
- ✅ Comprehensive error handling system
- ✅ Async compaction service

### In Progress
- 🔄 Proc-macro feature completion (str(n), bytes(n) parsing)
- 🔄 Attribute parsing implementation
- 🔄 Test and benchmark fixes

### Planned
- 📋 Trybuild UI tests for proc-macro
- 📋 Atomic segment replacement in compaction
- 📋 Checksum calculation for data integrity
- 📋 Schema evolution support
- 📋 Performance optimization

## Building

```bash
# Build all crates
cargo build

# Build specific crate
cargo build -p guardian-store
cargo build -p guardian-macros

# Run tests
cargo test

# Run benchmarks
cargo bench
```

## Usage

### Storage System

```rust
use guardian_store::{Store, User};

// Create store
let store = Store::new("./data")?;

// Save user
let user = User { /* ... */ };
store.save(&user)?;

// Find user
let found = store.find(123)?;
```

### Proc-Macro

```rust
use guardian_macros::frame;

#[frame]
pub struct Packet {
    id: u32,
    data: rest,
}

// Generated struct provides zero-copy access
let packet = Packet::new(&data)?;
let id = packet.id();
let data = packet.data();
```

## Contributing

1. Follow the single-word identifier philosophy
2. Update vocabulary.csv for new terms
3. Document decisions in decisions.csv
4. Add tasks to todo.csv
5. Maintain architectural consistency

## License

MIT License - see LICENSE file for details. 
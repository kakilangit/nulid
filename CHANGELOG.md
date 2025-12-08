# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2025-12-08

### Added

- Initial release of NULID (Nanosecond-Precision Universally Lexicographically Sortable Identifier)
- 150-bit identifier format (70-bit nanosecond timestamp + 80-bit randomness)
- Lexicographically sortable 30-character Base32 string encoding (Crockford's alphabet)
- Monotonic generator for guaranteed ordering within the same nanosecond
- Thread-safe generation with mutex-protected state
- Clock skew protection (never goes backward in time)
- Binary serialization (19 bytes)
- String parsing (case-insensitive)
- Optional serde support for JSON, MessagePack, TOML, and other formats
- Comprehensive test suite (124+ tests)
- Full documentation with examples
- Benchmark suite for performance testing
- Zero unsafe code
- Compatible with Rust 2024 edition

### Features

- **Core Types**
  - `Nulid` - Main identifier type
  - `Timestamp` - 70-bit nanosecond timestamp
  - `Random` - 80-bit cryptographically secure randomness
  - `Generator` - Thread-safe monotonic generator
- **Encoding/Decoding**
  - Base32 string representation (30 characters)
  - Binary representation (19 bytes)
  - Case-insensitive parsing
  - Lexicographic sorting preservation
- **Monotonicity**
  - Guaranteed strictly increasing IDs
  - Same-nanosecond handling via randomness increment
  - Overflow protection with proper error handling
- **Serialization** (optional `serde` feature)
  - JSON serialization/deserialization
  - MessagePack binary format
  - TOML configuration format
  - Any serde-compatible format

### Performance

- ~240,000 NULIDs/second single-threaded generation
- Thread-safe concurrent generation
- Zero-allocation hot paths where possible
- Efficient Base32 encoding/decoding

### Documentation

- Comprehensive API documentation
- Usage examples in docstrings
- Three complete example programs:
  - `basic.rs` - Basic NULID operations
  - `monotonic.rs` - Advanced monotonic generation patterns
  - `serde_example.rs` - Serialization integration
- Detailed README with specification

### Quality

- 92 unit tests
- 32 documentation tests
- 5 serde integration tests
- Zero clippy warnings (pedantic + nursery lints)
- Zero unsafe code
- Comprehensive benchmark suite

[Unreleased]: https://github.com/kakilangit/nulid/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/kakilangit/nulid/releases/tag/v0.1.0

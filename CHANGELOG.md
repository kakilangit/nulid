# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.5.6] - 2025-12-18

### Added

- **`Nulid::min()`** - Returns the minimum possible NULID value (all zeros)
  - Const function that returns `Nulid::MIN`
  - Equivalent to `Nulid::nil()` but provides semantic clarity for range operations
  - Example: `let min = Nulid::min();`

- **`Nulid::max()`** - Returns the maximum possible NULID value (all ones)
  - Const function that returns `Nulid::MAX`
  - Useful for range queries and boundary checks
  - Example: `let max = Nulid::max();`

- **`nulid_derive::Id` - `min()` and `max()` methods**
  - Added `min()` method to derived ID types (e.g., `UserId::min()`)
  - Added `max()` method to derived ID types (e.g., `UserId::max()`)
  - Maintains complete parity with `Nulid` API
  - Both are const functions for compile-time usage

### Changed

- **Version Bump** - All crates bumped to v0.5.6
  - `nulid`: 0.5.5 → 0.5.6
  - `nulid_derive`: 0.5.5 → 0.5.6
  - `nulid_macros`: 0.5.5 → 0.5.6

## [0.5.5] - 2025-12-18

### Added

- **`nulid_derive::Id`** - Complete 1-to-1 parity with Nulid

  **Constructor Methods (7 total):**
  - `new()` - Generate new ID with current timestamp
  - `now()` - Alias for `new()`
  - `nil()` - Create nil/zero ID
  - `from_datetime(SystemTime)` - Create from specific time
  - `from_nanos(u128, u64)` - Create from timestamp and random
  - `from_u128(u128)` - Create from raw u128
  - `from_bytes([u8; 16])` - Create from byte array

  **New Trait Implementations (matching Nulid exactly):**
  - `From<u128>` + `Into<u128>` - Convert to/from u128
  - `From<[u8; 16]>` + `Into<[u8; 16]>` - Convert to/from byte array
  - `AsRef<u128>` - Borrow as u128 reference
  - `TryFrom<&[u8]>` - Create from byte slice with validation

  **Deref Traits:**
  - `Deref<Target = Nulid>` - Direct access to all Nulid instance methods
  - `DerefMut` - Mutable access to inner Nulid
  - All Nulid methods now callable directly: `user_id.nanos()`, `user_id.random()`, `user_id.parts()`, etc.
  - Automatic coercion: `&UserId` can be used where `&Nulid` is expected

  **Complete Feature Parity:**
  - All 7 Nulid constructors now available as wrapper methods
  - All 17 trait implementations from Nulid now implemented
  - All instance methods accessible via Deref
  - 100% feature parity with Nulid while maintaining type safety

  **Testing:**
  - Added 20+ new tests for constructor methods and trait implementations
  - All tests verify 1-to-1 behavior match with Nulid
  - 64 total tests in nulid_derive (all passing)

### Changed

- **Version Bump** - All crates bumped to v0.5.5
  - `nulid`: 0.5.4 → 0.5.5
  - `nulid_derive`: 0.5.4 → 0.5.5
  - `nulid_macros`: 0.5.4 → 0.5.5

- **Documentation** - Updated to reflect complete feature set
  - All constructor methods documented
  - All trait implementations listed
  - Comprehensive usage examples

## [0.5.4] - 2025-12-18

### Added

- **`nulid_derive::Id`** - `Deref` and `DerefMut` trait implementations
  - Added `Deref<Target = Nulid>` to enable direct access to all Nulid methods
  - Added `DerefMut` for mutable access to the inner Nulid
  - Wrapper types can now call any Nulid method directly without extracting the inner value
  - Example: `let nanos = user_id.nanos();` instead of `user_id.0.nanos()` or `user_id.as_ref().nanos()`
  - Example: `let (timestamp, random) = user_id.parts();`
  - All Nulid methods are now available: `nanos()`, `micros()`, `millis()`, `random()`, `parts()`, `as_u128()`, `to_bytes()`, `is_nil()`, etc.
  - Enables automatic coercion: `&UserId` can be used where `&Nulid` is expected
  - Updated documentation with comprehensive examples
  - Added 11 new tests for Deref functionality

### Changed

- **Version Bump** - All crates bumped to v0.5.4
  - `nulid`: 0.5.3 → 0.5.4
  - `nulid_derive`: 0.5.3 → 0.5.4
  - `nulid_macros`: 0.5.3 → 0.5.4

## [0.5.3] - 2025-12-18

### Added

- **`nulid_derive::Id`** - New `new()` method and `Default` trait implementation
  - Added `new()` method to derived Id types that creates a fresh NULID instance
  - Added `Default` trait implementation that returns `Nulid::ZERO`
  - Example: `let user_id = UserId::new()?;` for new IDs
  - Example: `let default_id = UserId::default();` for ZERO IDs
  - Simplifies ID creation without explicit `Nulid::new()` or `From::from()` conversions
  - Updated documentation with usage examples
  - Added comprehensive tests for both `new()` and `Default` implementations

### Changed

- **Version Bump** - All crates bumped to v0.5.3
  - `nulid`: 0.5.2 → 0.5.3
  - `nulid_derive`: 0.5.2 → 0.5.3
  - `nulid_macros`: 0.5.2 → 0.5.3

## [0.5.2] - 2025-12-18

### Added

- **Makefile** - Comprehensive development workflow automation
  - Added Makefile with all CI/CD commands for local development
  - Automatically extracts `rust-version` from workspace Cargo.toml (1.88)
  - Provides targets for: `fmt`, `clippy`, `test`, `bench`, `examples`, `publish`
  - `make verify-version` - Check workspace version consistency
  - `make ci` - Run all CI checks locally
  - `make pre-commit` - Run pre-commit checks
  - Ensures consistency between local development and CI/CD

### Fixed

- **Release Workflow** - Retry-safe publishing with per-crate checks
  - Each crate now checks if already published before attempting to publish
  - Safe to re-run release workflow after partial failures
  - If `nulid_derive` publishes but `nulid_macros` fails, re-running will skip derive and continue
  - Removed obsolete `check-crates-published` and `skip-publish` jobs
  - Simplified job dependencies with `if: always()` conditions

### Changed

- **Documentation** - Updated README with v0.5 version references
  - All installation examples now reference version "0.5"
  - Updated feature examples and Cargo.toml snippets
  - Aligned documentation with current release version

- **Workspace Configuration** - Centralized edition and rust-version
  - Added `edition = "2024"` to workspace.package
  - Added `rust-version = "1.88"` to workspace.package
  - All crates now use `edition.workspace = true` and `rust-version.workspace = true`
  - Ensures consistent toolchain requirements across all workspace members

- **CI/CD Workflows** - Simplified using Makefile commands
  - Updated `.github/workflows/ci.yml` to use Makefile targets
  - Updated `.github/workflows/release.yml` to use Makefile for tests
  - Commands like `cargo fmt --all --check` now replaced with `make fmt-check`
  - Reduces duplication between local development and CI/CD

- **Workspace Version** - All crates bumped to v0.5.2
  - `nulid`: 0.5.1 → 0.5.2
  - `nulid_derive`: 0.5.1 → 0.5.2
  - `nulid_macros`: 0.5.1 → 0.5.2

## [0.5.1] - 2025-12-18

### Added

- **`Id` Derive Macro - Direct Nulid Comparison** - Enhanced comparison capabilities
  - `PartialEq<Nulid>` - Direct equality comparison between wrapper type and `Nulid`
  - `PartialOrd<Nulid>` - Direct ordering comparison between wrapper type and `Nulid`
  - Enables comparisons like `user_id == nulid` and `user_id < nulid` without conversion
  - Added 3 new tests for direct Nulid comparison (total 29 tests)

### Changed

- **Workspace Version** - All crates bumped to v0.5.1
  - `nulid`: 0.5.0 → 0.5.1
  - `nulid_derive`: 0.5.0 → 0.5.1
  - `nulid_macros`: 0.5.0 → 0.5.1

### Fixed

- **CI/CD** - Replaced deprecated `actions/create-release@v1` with modern `gh` CLI

## [0.5.0] - 2025-12-18

### Added

- **Enhanced `Id` Derive Macro** - Added automatic implementation of standard traits
  - `std::fmt::Debug` - Debug formatting with type name
  - `Copy` - Value semantics matching `Nulid` (automatically provides `Clone`)
  - `PartialEq` and `Eq` - Equality comparison
  - `PartialOrd` and `Ord` - Ordering comparison (delegates to inner `Nulid`)
  - `Hash` - Hashing support for use in `HashMap` and `HashSet`
  - These traits are now automatically derived, no need to add them manually
  - Updated examples to remove redundant trait derives
  - Added 8 new tests for the enhanced traits (total 27 tests)

### Changed

- **Workspace Version** - All crates now use workspace version 0.5.0
  - `nulid`: 0.4.0 → 0.5.0
  - `nulid_derive`: 0.4.0 → 0.5.0
  - `nulid_macros`: 0.4.0 → 0.5.0
  - Shared version and metadata across workspace for consistency

- **`nulid_derive`** - Enhanced with automatic trait implementations
  - Breaking: Users no longer need to manually derive `Debug`, `Copy`, `PartialEq`, `Eq`, `PartialOrd`, `Ord`, or `Hash`
  - These are now automatically provided by `#[derive(Id)]`
  - Note: `Clone` is automatically provided by `Copy`

## [0.4.0] - 2025-12-18

### Added

- **MSRV Update** - Bumped minimum supported Rust version from 1.86 to 1.88
  - Updated CI to use Rust 1.88
- **`Id` Derive Macro** (`derive` feature, `nulid_derive` crate)
  - Automatically implements common traits for types wrapping `Nulid`
  - `TryFrom<String>` and `TryFrom<&str>` - Parse from strings
  - `From<Nulid>` and `From<WrapperType> for Nulid` - Bidirectional conversion
  - `AsRef<Nulid>` - Borrow inner Nulid
  - `std::fmt::Display` - Format as Base32 string
  - `std::str::FromStr` - Parse using `.parse()`
  - Enables type-safe wrapper types for different ID kinds
  - Example: `examples/derive_wrapper.rs`
  - Comprehensive test suite (19 tests)

- **`nulid!()` Macro** (`macros` feature, `nulid_macros` crate)
  - Convenient NULID generation with flexible error handling
  - `nulid!()` - Generate NULID, panicking on error (for tests/initialization)
  - `nulid!(?)` - Generate NULID, returning `Result<Nulid, Error>` (for error handling)
  - Zero runtime overhead (compile-time expansion)
  - Example: `examples/macros.rs`

- **Combined Features Example**
  - `examples/combined_features.rs` demonstrates both features working together
  - Shows type-safe ID wrappers with convenient generation
  - Demonstrates error handling patterns and conversions

- **Workspace Structure**
  - Organized as Cargo workspace with three crates:
    - `nulid` - Core library (v0.4.0)
    - `nulid_derive` - Derive macros (v0.4.0)
    - `nulid_macros` - Procedural macros (v0.4.0)
  - Clean separation of concerns
  - Reusable derive and macro crates

### Changed

- **Version Bump** - All crates updated to v0.4.0
  - `nulid`: 0.3.2 → 0.4.0
  - `nulid_derive`: new crate at 0.4.0
  - `nulid_macros`: new crate at 0.4.0

- **Feature Flags**
  - Added `derive` feature for `Id` derive macro
  - Added `macros` feature for `nulid!()` macro
  - Optional dependencies properly configured

- **Documentation**
  - Updated README with new features
  - Added comprehensive READMEs for derive and macro crates
  - Updated examples to demonstrate new functionality
  - Import instructions clarified (proc macros imported from derive crate)

### Developer Experience

- **Type Safety** - Different ID types cannot be accidentally mixed

  ```rust
  #[derive(Id)]
  struct UserId(Nulid);

  #[derive(Id)]
  struct OrderId(Nulid);

  // Compile error: type mismatch
  // let user: UserId = order_id;
  ```

- **Ergonomic API** - Less boilerplate for common patterns

  ```rust
  // Before v0.4.0
  let id = Nulid::new().expect("Failed to generate NULID");

  // After v0.4.0
  let id = nulid!();
  ```

- **Automatic Trait Implementation** - No manual trait implementations needed

  ```rust
  #[derive(Id, Debug, Clone, Copy, PartialEq, Eq, Hash)]
  pub struct UserId(Nulid);

  // Automatically get: TryFrom, Display, FromStr, AsRef, etc.
  let id: UserId = "01H0JQ4VEFSBV974PRXXWEK5ZW".parse()?;
  println!("{}", id); // Works!
  ```

### Testing

- **97 tests total** across all features
  - Core library: 72 tests
  - Derive macro: 19 tests
  - Integration tests for all features
  - All tests passing with `--all-features`

### Quality

- Zero clippy warnings with pedantic + nursery lints
- Comprehensive documentation for all new features
- Examples for all feature combinations
- Backward compatible with v0.3.2 core API

## [0.3.2] - 2024-12-13

### Added

- **Standard trait implementations** - Additional conversions and references for better ergonomics
  - `From<u128> for Nulid` - Convert from u128: `let id: Nulid = value.into()`
  - `From<Nulid> for u128` - Convert to u128: `let value: u128 = id.into()`
  - `From<[u8; 16]> for Nulid` - Convert from byte array: `let id: Nulid = bytes.into()`
  - `From<Nulid> for [u8; 16]` - Convert to byte array: `let bytes: [u8; 16] = id.into()`
  - `AsRef<u128> for Nulid` - Borrow internal u128 value: `let value_ref: &u128 = id.as_ref()`
  - `TryFrom<&[u8]> for Nulid` - Safe conversion from byte slices with length validation

### Changed

- **Improved API ergonomics** - More idiomatic Rust patterns following standard library conventions
  - Conversions now available via trait implementations in addition to existing methods
  - Better integration with generic code that expects standard traits

## [0.3.1] - 2024-12-13

### Changed

- **Improved nanosecond precision** - Use `quanta` for true nanosecond-precision timestamps
  - Achieves true nanosecond precision on all platforms (macOS, Linux, Windows)
  - Previous implementation using system clocks only provided microsecond precision on macOS
  - Uses high-resolution monotonic clock combined with wall-clock time for accurate ordering
  - Values no longer rounded to nearest microsecond (ending in 000)
  - Better uniqueness and ordering guarantees for high-frequency NULID generation

### Dependencies

- Added `quanta = "0.12"` for high-precision cross-platform timing

## [0.3.0] - 2024-12-12

### Breaking Changes

- **Removed `timestamp_nanos()`** - Use `nanos()` instead
- **Renamed `from_timestamp_nanos()` to `from_nanos()`** - Shorter, cleaner API

### Added

- **New timestamp accessor methods**
  - `nanos()` - Returns timestamp in nanoseconds
  - `micros()` - Returns timestamp in microseconds
  - `millis()` - Returns timestamp in milliseconds
  - Comprehensive test coverage for all new methods

- **rkyv Support** - Zero-copy serialization via `rkyv` feature
  - Archive, Serialize, and Deserialize derive macros
  - Compatible with rkyv 0.8 (with `alloc` and `bytecheck` features)
  - Example: `examples/rkyv_example.rs`
  - Efficient serialization: 16 bytes per NULID
  - Zero-copy access to archived data

- **postgres-types Support** - PostgreSQL integration via `postgres-types` feature
  - `ToSql` and `FromSql` trait implementations
  - Store NULIDs as PostgreSQL UUIDs (16 bytes, big-endian)
  - Full compatibility with PostgreSQL UUID columns
  - Maintains nanosecond precision and lexicographic sortability
  - Example: `examples/postgres_types_example.rs`
  - Comprehensive test coverage (6 tests)

### Changed

- **MSRV Update** - Bumped minimum supported Rust version from 1.85 to 1.86
  - Required for criterion 0.8 (dev-dependency for benchmarks)
  - Updated CI to use Rust 1.86

- **Documentation Updates**
  - Removed version comparison references from README (no longer mentions v0.1.0)
  - Updated randomness source documentation from `getrandom` to `rand` crate
  - Rephrased feature descriptions to reflect current state without historical comparisons
  - Updated all examples and documentation to use new method names

- **Code Quality Improvements**
  - Removed manual `#[cfg(feature = "...")]` directives from all examples
  - Added `[[example]]` sections with `required-features` in Cargo.toml
  - Fixed all clippy warnings in library and examples
  - Improved format string consistency (inlined format args)
  - Better hex literal grouping for readability

### Fixed

- **CI/CD Pipeline**
  - Added new examples to GitHub Actions workflow
  - Examples now run automatically on CI for both features

## [0.2.1] - 2024-12-11

### Fixed

- **Serde binary serialization** - Fixed binary format serialization to use fixed-size arrays
  - Changed from `serialize_bytes()` to `serialize_tuple()` for consistent 16-byte encoding
  - Fixes compatibility with bincode 2.0 and other binary formats
  - Ensures no length prefix overhead in binary serialization
  - All binary formats (Bincode, MessagePack, etc.) now serialize consistently

### Added

- **Bincode 2.0 Support** (via `serde` feature)
  - Efficient binary serialization using bincode 2.0
  - Fixed 16-byte encoding per NULID (no length prefix overhead)
  - ~1.75x more compact than JSON serialization
  - Support for both standard and legacy bincode configurations
  - Works automatically through serde implementation
  - Comprehensive test suite with 5 bincode tests in serde module
  - Examples included in `examples/serde_example.rs`

## [0.2.0] - 2024-12-09

### Breaking Changes

- **Reduced identifier size**: Changed from 150-bit to **128-bit** (UUID-compatible)
  - Timestamp: 70 bits → **68 bits** (nanoseconds, valid until year ~11,326 AD)
  - Randomness: 80 bits → **60 bits** (1.15 quintillion unique IDs per nanosecond)
- **String encoding**: 30 characters → **26 characters** (still Base32)
- **Binary format**: 19 bytes → **16 bytes** (standard UUID size)
- Migration: Existing 0.1.0 NULIDs are incompatible with 0.2.0 format

### Added

- **UUID Interoperability** (`uuid` feature)
  - `to_uuid()` / `from_uuid()` methods for seamless UUID conversion
  - `From<uuid::Uuid>` and `From<Nulid>` trait implementations
  - Full 128-bit value preservation
  - Compatible with UUID-based databases and APIs
- **SQLx PostgreSQL Support** (`sqlx` feature)
  - Native PostgreSQL UUID storage support
  - Automatic `Encode`/`Decode` implementations
  - `FromRow` derive macro support for easy struct mapping
  - PostgreSQL array type support (`PgHasArrayType`)
  - Comprehensive example: `examples/sqlx_postgres.rs`
- **Modular Feature Architecture**
  - Organized features into `src/features/` module
  - Separate files: `uuid.rs`, `serde.rs`, `sqlx.rs`
  - Clean separation between core and optional functionality
- **CLI Enhancements**
  - Updated help text with valid 26-character NULID examples
  - All commands verified and working

### Performance Improvements

- **21x faster generation**: 704ns → **35ns** per NULID (~28.5M IDs/sec)
  - Switched from direct `getrandom` to buffered `rand` crate
  - Amortized syscall costs through RNG buffering
- **2.8x faster encoding**: 26.78ns → **9.18ns** for Base32 encoding
  - Optimized encoding algorithm with unrolled loops
- **Consistent batch performance**: ~21M IDs/sec sustained across all batch sizes
- **Zero-copy operations**: Minimized allocations throughout hot paths

### Changed

- **Core Structure**
  - Simplified to two-part design (timestamp + random, no separate components)
  - Direct u128 internal representation for optimal performance
  - Removed custom randomness module in favor of `rand` crate
- **Dependencies**
  - Added: `rand = "0.9"` (replaces direct getrandom usage)
  - Optional: `uuid = "1.0"` (for UUID feature)
  - Optional: `sqlx = "0.8"` (for PostgreSQL support)
- **API Refinements**
  - All methods return `Result` instead of panicking
  - More ergonomic error handling throughout
  - Improved const fn support for compile-time operations

### Security & Safety

- **Zero unsafe code**: Enforced with `#![forbid(unsafe_code)]`
- **Panic-free production paths**: All `unwrap`/`expect` removed from library code
- **Strict clippy lints**: Pedantic and nursery lints enabled and enforced
- **Memory safety**: No buffer overflows, no undefined behavior
- **Thread-safe**: Concurrent generation without data races

### Documentation

- **Updated README** with v0.2.0 features and benchmarks
- **Comprehensive examples**:
  - UUID conversion patterns
  - SQLx/PostgreSQL integration
  - Database migration strategies
- **Feature flag documentation**: Clear guide for `uuid`, `serde`, and `sqlx` features
- **Performance comparison**: Detailed before/after benchmarks

### Testing

- **67 tests** with all features enabled (up from 53 core tests)
- **Feature-specific test suites**:
  - UUID: 6 tests for conversion and round-trips
  - Serde: 5 tests for JSON and binary formats
  - SQLx: 3 tests for PostgreSQL type system
- **Live integration testing**: Verified with PostgreSQL in Docker
- **CI/CD ready**: All tests pass without external dependencies (features optional)

### Benchmarks (v0.2.0)

Performance on Apple M-series processor:

| Operation                       | Time     | Throughput     | vs v0.1.0       |
| ------------------------------- | -------- | -------------- | --------------- |
| Generate new NULID              | 35.03 ns | 28.5M ops/sec  | **21x faster**  |
| From datetime                   | 14.73 ns | 67.9M ops/sec  | New             |
| Monotonic generation            | 48.01 ns | 20.8M ops/sec  | **15x faster**  |
| Sequential generation (100 IDs) | 4.78 µs  | 20.9M IDs/sec  | **15x faster**  |
| Encode to string (array)        | 9.18 ns  | 109M ops/sec   | **2.9x faster** |
| Encode to String (heap)         | 33.49 ns | 29.9M ops/sec  | 2.1x faster     |
| Decode from string              | 8.81 ns  | 114M ops/sec   | 11x faster      |
| Convert to bytes                | 295 ps   | 3.39B ops/sec  | ~same           |
| Convert from bytes              | 395 ps   | 2.53B ops/sec  | ~same           |
| Sort 1000 IDs                   | 13.17 µs | 75.9M elem/sec | 5.7x faster     |

### Quality Metrics

- **Clippy**: 0 errors, 0 warnings (pedantic + nursery lints)
- **Code coverage**: Comprehensive unit and integration tests
- **Memory safety**: Zero unsafe code, compiler-enforced
- **API stability**: Semantic versioning, clear migration path

### Migration Guide (0.1.0 → 0.2.0)

**Format Changes:**

- String length: 30 chars → 26 chars
- Binary size: 19 bytes → 16 bytes
- Bit layout: Different (not compatible)

**Code Changes:**

```rust
// v0.1.0 (150-bit)
let nulid = Nulid::new()?; // 30-character string

// v0.2.0 (128-bit, UUID-compatible)
let nulid = Nulid::new()?; // 26-character string

// New: UUID interoperability
let uuid = nulid.to_uuid();
let nulid2 = Nulid::from_uuid(uuid);

// New: SQLx PostgreSQL support
#[derive(sqlx::FromRow)]
struct User {
    id: Nulid,  // Stored as UUID in PostgreSQL
    name: String,
}
```

**Recommendation**: Use v0.2.0 for all new projects. The UUID compatibility and performance improvements make it significantly better than v0.1.0.

## [0.1.0] - 2024-12-08

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
- Command-line tool (`nulid` binary) for generating and inspecting NULIDs
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
- **CLI Tool**
  - Generate NULIDs from the command line
  - Inspect NULID components (timestamp, randomness, bytes)
  - Validate NULID strings
  - Decode to hex format
  - Parse and validate from stdin
  - Installable via `cargo install nulid`
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

Benchmark results (measured on modern hardware):

- **Generation**: ~1.1 µs per NULID (~900,000 NULIDs/second)
- **Encoding**:
  - to_string: ~71 ns
  - from_string: ~97 ns
  - round_trip: ~168 ns
- **Byte Serialization**:
  - to_bytes: ~0.9 ns
  - from_bytes: ~1.5 ns
  - round_trip: ~2.1 ns
- **Comparison**:
  - equality: ~1.3 ns
  - ordering: ~1.0 ns
- **Sorting**: ~2.3 µs for 1,000 NULIDs (436 Melem/s)
- **Batch Generation**: ~900,000 NULIDs/second sustained
- **Concurrent Generation**: ~3.3 ms for 10,000 NULIDs across 10 threads
- **Serde (JSON)**:
  - serialize: ~104 ns
  - deserialize: ~132 ns
  - round_trip: ~237 ns

Thread-safe concurrent generation with zero-allocation hot paths where possible.

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

[Unreleased]: https://github.com/kakilangit/nulid/compare/v0.4.0...HEAD
[0.4.0]: https://github.com/kakilangit/nulid/compare/v0.3.2...v0.4.0
[0.3.2]: https://github.com/kakilangit/nulid/compare/v0.3.1...v0.3.2
[0.3.1]: https://github.com/kakilangit/nulid/compare/v0.3.0...v0.3.1
[0.3.0]: https://github.com/kakilangit/nulid/compare/v0.2.1...v0.3.0
[0.2.1]: https://github.com/kakilangit/nulid/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/kakilangit/nulid/releases/tag/v0.2.0
[0.1.0]: https://github.com/kakilangit/nulid/releases/tag/v0.1.0

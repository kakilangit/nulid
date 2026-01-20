# NULID

**Nanosecond-Precision Universally Lexicographically Sortable Identifier**

[![Crates.io](https://img.shields.io/crates/v/nulid.svg)](https://crates.io/crates/nulid)
[![Documentation](https://docs.rs/nulid/badge.svg)](https://docs.rs/nulid)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/kakilangit/nulid/blob/main/LICENSE)
[![Rust Version](https://img.shields.io/badge/rust-1.88%2B-blue.svg)](https://www.rust-lang.org)
<a href="https://github.com/kakilangit/nulid"><img alt="github" src="https://img.shields.io/badge/github-kakilangit/nulid-37a8e0?style=for-the-badge&labelColor=555555&logo=github" height="20"></a>

---

## Overview

NULID is a 128-bit identifier with **true nanosecond-precision timestamps** designed for high-throughput, distributed systems. It combines the simplicity of ULID with sub-millisecond precision for systems that require fine-grained temporal ordering.

**True nanosecond precision** is achieved using the `quanta` crate, which provides high-resolution monotonic timing combined with wall-clock synchronization. This ensures proper ordering even on systems where the OS clock only provides microsecond precision.

### Why NULID?

**The Challenge:**

- ULID's 48-bit millisecond timestamp is insufficient for high-throughput systems generating thousands of IDs per millisecond
- Systems processing millions of operations per second need nanosecond precision for proper chronological ordering

**The Solution:**

- NULID uses a **68-bit nanosecond timestamp** for precise chronological ordering
- Maintains **60-bit cryptographically secure randomness** for collision resistance
- **128-bit total** - same size as UUID, smaller than original 150-bit designs
- **26-character encoding** - compact and URL-safe

### Features

**128-bit identifier** (16 bytes) - UUID-compatible size  
**High-performance** - 11.78ns per ID generation  
**Lexicographically sortable** with true nanosecond precision  
**26-character canonical encoding** using Crockford's Base32  
**Extended lifespan** - valid until year **~11,326 AD**  
**Memory safe** - zero unsafe code, panic-free production paths  
**URL safe** - no special characters  
**Monotonic sort order** within the same nanosecond  
**UUID interoperability** - seamless conversion to/from UUID  
**1.15 quintillion unique IDs per nanosecond** (60 bits of randomness)  
**True nanosecond precision** - powered by `quanta` for high-resolution timing on all platforms

---

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
nulid = "0.7"
```

With optional features:

```toml
[dependencies]
nulid = { version = "0.7", features = ["uuid"] }        # UUID conversion
nulid = { version = "0.7", features = ["derive"] }      # Id derive macro
nulid = { version = "0.7", features = ["macros"] }      # nulid!() macro
nulid = { version = "0.7", features = ["serde"] }       # Serialization
nulid = { version = "0.7", features = ["sqlx"] }        # PostgreSQL support
nulid = { version = "0.7", features = ["postgres-types"] } # PostgreSQL types
nulid = { version = "0.7", features = ["rkyv"] }        # Zero-copy serialization
nulid = { version = "0.7", features = ["chrono"] }      # DateTime<Utc> support
```

---

## Quick Start

### Basic Usage

```rust
use nulid::Nulid;

# fn main() -> nulid::Result<()> {
// Generate a new NULID
let id = Nulid::new()?;
println!("{}", id); // "01AN4Z07BY79K47PAZ7R9SZK18"

// Parse from string
let parsed: Nulid = "01AN4Z07BY79K47PAZ7R9SZK18".parse()?;

// Extract components
let nanos = id.nanos();    // u128: nanoseconds since epoch
let micros = id.micros();  // u128: microseconds since epoch
let millis = id.millis();  // u128: milliseconds since epoch
let random = id.random();  // u64: 60-bit random value
# Ok(())
# }
```

### Convenient Generation with `nulid!()` Macro

With the `macros` feature:

```rust,ignore
use nulid::nulid;

// Simple generation (panics on error)
let id = nulid!();

// With error handling
fn example() -> Result<(), Box<dyn std::error::Error>> {
    let id = nulid!(?)?;
    Ok(())
}

// Multiple IDs
let (id1, id2, id3) = (nulid!(), nulid!(), nulid!());
```

### Type-Safe ID Wrappers with `Id` Derive

With the `derive` feature:

```rust,ignore
use nulid::Nulid;
use nulid_derive::Id;

#[derive(Id, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UserId(Nulid);

#[derive(Id, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct OrderId(Nulid);

fn example() -> Result<(), Box<dyn std::error::Error>> {
    // Type-safe IDs that can't be mixed
    let user_id = UserId::from(nulid::Nulid::new()?);
    let order_id = OrderId::from(nulid::Nulid::new()?);

    // Parse from strings
    let user_id: UserId = "01H0JQ4VEFSBV974PRXXWEK5ZW".parse()?;

    // Display, FromStr, TryFrom, AsRef all implemented automatically
    println!("{}", user_id);
    Ok(())
}
```

### Conversions and Traits

```rust
use nulid::Nulid;

# fn main() -> nulid::Result<()> {
let id = Nulid::new()?;

// Convert to/from bytes
let bytes = id.to_bytes();          // [u8; 16]
let id2 = Nulid::from_bytes(bytes);

// Ergonomic conversions using standard traits
let id3: Nulid = bytes.into();      // From<[u8; 16]>
let bytes2: [u8; 16] = id.into();   // Into<[u8; 16]>
let value: u128 = id.into();        // Into<u128>
let id4: Nulid = value.into();      // From<u128>

// Safe conversion from byte slices
let slice: &[u8] = &bytes;
let id5 = Nulid::try_from(slice)?;  // TryFrom<&[u8]>
# Ok(())
# }
```

### Monotonic Generation

```rust
use nulid::Generator;

# fn main() -> nulid::Result<()> {
let generator = Generator::new();

// Generate multiple IDs - guaranteed strictly increasing
let id1 = generator.generate()?;
let id2 = generator.generate()?;
let id3 = generator.generate()?;

assert!(id1 < id2);
assert!(id2 < id3);
# Ok(())
# }
```

### Distributed Generation (Multi-Node)

For distributed systems requiring guaranteed cross-node uniqueness:

```rust
use nulid::generator::{Generator, SystemClock, CryptoRng, WithNodeId};

# fn main() -> nulid::Result<()> {
// Each node gets a unique ID (0-65535)
let generator = Generator::<SystemClock, CryptoRng, WithNodeId>::with_node_id(1);
let id = generator.generate()?;

// Node ID is embedded in the random bits
assert_eq!(generator.node_id(), Some(1));
# Ok(())
# }
```

### Testing with Mock Clock

The generator supports dependency injection for testing clock skew scenarios:

```rust
use nulid::generator::{Generator, MockClock, SeededRng, NoNodeId};
use std::time::Duration;

# fn main() -> nulid::Result<()> {
// Create mock clock and seeded RNG for reproducible tests
let clock = MockClock::new(1_000_000_000);
let rng = SeededRng::new(42);
let generator = Generator::<_, _, NoNodeId>::with_deps(&clock, &rng);

let id1 = generator.generate()?;

// Simulate clock regression (NTP correction)
clock.regress(Duration::from_millis(100));

let id2 = generator.generate()?;

// Still monotonic despite clock going backward!
assert!(id2 > id1);
# Ok(())
# }
```

### `SQLx` `PostgreSQL` Support

With the optional `sqlx` feature, you can store NULIDs directly in `PostgreSQL` as UUIDs:

```rust,ignore
use nulid::Nulid;
use sqlx::{PgPool, Row};

#[derive(sqlx::FromRow)]
struct User {
    id: Nulid,
    name: String,
}

async fn insert_user(pool: &PgPool, id: Nulid, name: &str) -> sqlx::Result<()> {
    sqlx::query("INSERT INTO users (id, name) VALUES ($1, $2)")
        .bind(id)  // Automatically converts to UUID
        .bind(name)
        .execute(pool)
        .await?;
    Ok(())
}

async fn get_user(pool: &PgPool, id: Nulid) -> sqlx::Result<User> {
    sqlx::query_as::<_, User>("SELECT id, name FROM users WHERE id = $1")
        .bind(id)
        .fetch_one(pool)
        .await
}
```

This enables:

- **Native UUID storage** - NULIDs are stored as `PostgreSQL` UUID type
- **Automatic conversion** - Seamless encoding/decoding with sqlx
- **Time-ordered queries** - Query by ID for chronological ordering
- **Index efficiency** - Use `PostgreSQL`'s native UUID indexes
- **Type safety** - Compile-time checked queries with sqlx

### UUID Interoperability

With the optional `uuid` feature, you can seamlessly convert between NULID and UUID:

```rust,ignore
use nulid::Nulid;
use uuid::Uuid;

// Generate a NULID
let nulid = Nulid::new()?;

// Convert to UUID
let uuid: Uuid = nulid.into();
println!("UUID: {}", uuid); // "01234567-89ab-cdef-0123-456789abcdef"

// Convert back to NULID
let nulid2: Nulid = uuid.into();
assert_eq!(nulid, nulid2);

// Or use explicit methods
let uuid2 = nulid.to_uuid();
let nulid3 = Nulid::from_uuid(uuid2);
```

This enables:

- **Database compatibility** - Store as UUID in Postgres, `MySQL`, etc.
- **API compatibility** - Accept/return UUIDs while using NULID internally
- **Migration path** - Gradually migrate from UUID to NULID
- **Interoperability** - Work with existing UUID-based systems

### Chrono `DateTime` Support

With the optional `chrono` feature, you can convert between NULIDs and `chrono::DateTime<Utc>`:

```rust,ignore
use nulid::Nulid;
use chrono::{DateTime, Utc, TimeZone};

// Generate a NULID
let id = Nulid::new()?;

// Convert to DateTime<Utc>
let dt: DateTime<Utc> = id.chrono_datetime();
println!("Timestamp: {}", dt); // "2025-12-23 10:30:45.123456789 UTC"

// Create NULID from DateTime<Utc>
let dt = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
let id = Nulid::from_chrono_datetime(dt)?;

// Works with derived Id types too
#[derive(Id)]
struct UserId(Nulid);

let user_id = UserId::new()?;
let created_at = user_id.chrono_datetime();

let dt = Utc.with_ymd_and_hms(2024, 1, 1, 12, 0, 0).unwrap();
let user_id = UserId::from_chrono_datetime(dt)?;
```

This enables:

- **Human-readable timestamps** - Convert NULID timestamps to standard `DateTime` format
- **Time-based queries** - Easy integration with chrono-based time operations
- **Nanosecond precision** - Full nanosecond precision is preserved
- **Bidirectional conversion** - Create NULIDs from `DateTime` or extract `DateTime` from NULIDs
- **Timezone support** - Uses `DateTime` in UTC for consistency


### Sorting

```rust
use nulid::Nulid;

# fn main() -> nulid::Result<()> {
let mut ids = vec![
    Nulid::new()?,
    Nulid::new()?,
    Nulid::new()?,
];

// NULIDs are naturally sortable by timestamp
ids.sort();

// Verify chronological order
assert!(ids.windows(2).all(|w| w[0] < w[1]));
# Ok(())
# }
```

---

## Specification

The NULID is a **128-bit** (16 byte) binary identifier composed of:

```text
 68-bit timestamp (nanoseconds)         60-bit randomness
|--------------------------------|  |--------------------------|
          Timestamp                       Randomness
          68 bits                          60 bits
```

### Components

#### **Timestamp** (68 bits)

- **Size:** 68-bit integer
- **Representation:** UNIX time in **nanoseconds** (ns) since epoch (1970-01-01 00:00:00 UTC)
- **Range:** 0 to 2^68-1 nanoseconds (~9,356 years from 1970)
- **Valid Until:** Year ~11,326 AD
- **Encoding:** Most Significant Bits (MSB) first to ensure lexicographical sortability

#### **Randomness** (60 bits)

- **Size:** 60 bits
- **Source:** Cryptographically secure randomness via `rand` crate with system entropy
- **Collision Probability:** 1.15 × 10^18 unique values per nanosecond
- **Purpose:** Ensures uniqueness when multiple IDs are generated within the same nanosecond

### Total: 128 bits (16 bytes)

---

## Canonical String Representation

```text
ttttttttttttt rrrrrrrrrrrrr
```

where:

- **`t`** = Timestamp (13 characters)
- **`r`** = Randomness (13 characters)

**Total Length:** 26 characters

### Encoding

NULID uses **Crockford's Base32** encoding:

```text
0123456789ABCDEFGHJKMNPQRSTVWXYZ
```

**Character Exclusions:** The letters `I`, `L`, `O`, and `U` are excluded to avoid confusion.

#### Encoding Breakdown

| Component  | Bits | Characters | Calculation |
| ---------- | ---- | ---------- | ----------- |
| Timestamp  | 68   | 14         | ⌈68 ÷ 5⌉    |
| Randomness | 60   | 12         | ⌈60 ÷ 5⌉    |
| **Total**  | 128  | 26         | ⌈128 ÷ 5⌉   |

**Note:** Due to Base32 encoding (5 bits per character), we need 26 characters for 128 bits (130 bits capacity, with 2 bits unused).

---

## Sorting

NULIDs are **lexicographically sortable**:

- The **timestamp** occupies the most significant bits, ensuring time-based sorting
- The **randomness** provides secondary ordering for IDs within the same nanosecond
- String representation preserves binary sort order

### Example Sort Order

```text
01AN4Z07BY79K47PAZ7R9SZK18  ← Earlier
01AN4Z07BY79K47PAZ7R9SZK19
01AN4Z07BY79K47PAZ7R9SZK1A
01AN4Z07BY79K47PAZ7R9SZK1B  ← Later
```

---

## Monotonicity

The `Generator` ensures strictly monotonic IDs:

1. If the timestamp advances, use new timestamp with fresh random bits
2. If the timestamp is the same, increment the previous NULID by 1
3. This guarantees strict ordering even when generating millions of IDs per second

### Example

```rust
use nulid::Generator;

# fn main() -> nulid::Result<()> {
let generator = Generator::new();

// Even if called within the same nanosecond
let id1 = generator.generate()?; // ...XYZ
let id2 = generator.generate()?; // ...XYZ + 1
let id3 = generator.generate()?; // ...XYZ + 2

assert!(id1 < id2 && id2 < id3);
# Ok(())
# }
```

### Overflow

With 60 bits of randomness, you can generate 2^60 (1.15 quintillion) IDs within the same nanosecond before overflow. This is practically impossible in real-world usage.

---

## Binary Layout and Byte Order

The NULID is encoded as **16 bytes** with **Most Significant Byte (MSB) first** (network byte order / big-endian).

### Structure

```text
Byte:     0       1       2       3       4       5       6       7
      +-------+-------+-------+-------+-------+-------+-------+-------+
Bits: |  Timestamp (68 bits) - nanoseconds since epoch                |
      +-------+-------+-------+-------+-------+-------+-------+-------+

Byte:     8       9      10      11      12      13      14      15
      +-------+-------+-------+-------+-------+-------+-------+-------+
Bits: | T |    Randomness (60 bits)                                  |
      +-------+-------+-------+-------+-------+-------+-------+-------+
```

**Detailed Layout:**

- **Bytes 0-7, bits 0-3 of byte 8:** 68-bit timestamp (upper 68 bits of u128)
- **Bits 4-7 of byte 8, bytes 9-15:** 60-bit randomness (lower 60 bits of u128)

This structure ensures:

- Natural lexicographic ordering (timestamp in most significant bits)
- Simple bit operations (just shift and mask)
- Maximum precision (nanosecond resolution)
- UUID compatibility (128 bits / 16 bytes)

---

## Comparison: ULID vs NULID

| Feature               | ULID              | NULID            |
| --------------------- | ----------------- | ---------------- |
| **Total Bits**        | 128               | 128              |
| **String Length**     | 26 chars          | 26 chars         |
| **Timestamp Bits**    | 48 (milliseconds) | 68 (nanoseconds) |
| **Randomness Bits**   | 80                | 60               |
| **Time Precision**    | 1 millisecond     | 1 nanosecond     |
| **Lifespan**          | Until 10889 AD    | Until 11,326 AD  |
| **IDs per Time Unit** | 1.21e+24 / ms     | 1.15e+18 / ns    |
| **Sortable**          | ✅                | ✅               |
| **Monotonic**         | ✅                | ✅               |
| **URL Safe**          | ✅                | ✅               |
| **UUID Compatible**   | ✅                | ✅               |

---

## Performance & Safety

### Performance

- **High-performance generation** - 11.78ns per ID
- **Optimized Base32 encoding** - 9.1ns
- **Buffered RNG** - Uses `rand` crate for amortized cryptographic randomness
- **Zero-copy operations** - Minimal allocations and copies

### Safety Guarantees

- **Zero unsafe code** - Enforced with `#![forbid(unsafe_code)]`
- **Panic-free production paths** - All errors handled via `Result`
- **Strict linting** - Comprehensive clippy checks for safety
- **Memory safe** - No buffer overflows, no undefined behavior
- **Thread-safe** - Concurrent generation without data races

### Additional Features

- **Optional serde support** for serialization (JSON, TOML, `MessagePack`, Bincode, etc.)
  - Binary formats (Bincode, `MessagePack`) use efficient 16-byte encoding
  - Text formats (JSON, TOML) use 26-character string representation
- **Optional UUID interoperability** for seamless conversion
- **Optional `SQLx` support** for `PostgreSQL` UUID storage
- **Thread-safe** monotonic generation
- **Comprehensive test coverage**
- **Optimized bit operations**

---

## Command-Line Interface

The `nulid` binary provides a powerful CLI for working with NULIDs.

### Installation

Install the CLI with all features enabled:

```bash
cargo install nulid --all-features
```

Or build from source:

```bash
cargo build --bin nulid --release --features "uuid,chrono"
```

### Usage

```bash
# Generate NULIDs
nulid generate      # Generate one NULID
nulid gen 10        # Generate 10 NULIDs

# Inspect NULID details
nulid inspect 01GZWQ22K2MNDR0GAQTE834QRV
# Output shows: timestamp, random bits, bytes, datetime, UUID (if feature enabled)

# Parse and validate
nulid parse 01GZWQ22K2MNDR0GAQTE834QRV
nulid validate 01GZWQ22K2MNDR0GAQTE834QRV 01GZWQ22K2TKVGHH1Z1G0AK1EK

# Compare two NULIDs
nulid compare 01GZWQ22K2MNDR0GAQTE834QRV 01GZWQ22K2TKVGHH1Z1G0AK1EK
# Shows which is earlier and time difference in nanoseconds

# Sort NULIDs chronologically
nulid sort 01GZWQ22K2TKVGHH1Z1G0AK1EK 01GZWQ22K2MNDR0GAQTE834QRV
cat nulids.txt | nulid sort

# Decode to hex
nulid decode 01GZWQ22K2MNDR0GAQTE834QRV
```

### UUID Commands (requires `--features uuid`)

```bash
# Convert NULID to UUID
nulid uuid 01GZWQ22K2MNDR0GAQTE834QRV

# Convert UUID to NULID
nulid from-uuid 018d3f9c-5a2e-7b4d-8f1c-3e6a9d2c5b7e
```

### `DateTime` Commands (requires `--features chrono`)

```bash
# Convert NULID to ISO 8601 datetime
nulid datetime 01GZWQ22K2MNDR0GAQTE834QRV
# Output: 2024-01-01T00:00:00.123456789+00:00

# Create NULID from datetime
nulid from-datetime 2024-01-01T00:00:00Z
```

---

## Use Cases

NULID is ideal for:

- **High-frequency trading systems** requiring nanosecond-level event ordering
- **Distributed databases** with high write throughput (`PostgreSQL` UUID storage via sqlx)
- **Event sourcing systems** where precise ordering is critical
- **Microservices architectures** generating many concurrent IDs
- **`IoT` platforms** processing millions of sensor readings per second
- **Real-time analytics** systems requiring precise event sequencing
- **`PostgreSQL` applications** - Store as native UUID with time-based ordering
- **Migration from UUID** - Drop-in replacement with better time ordering
- **Any system** needing UUID-sized IDs with nanosecond precision and sortability

---

## API Reference

### Core Type

```rust,ignore
pub struct Nulid(u128);

impl Nulid {
    // Generation
    pub fn new() -> Result<Self>;
    pub fn now() -> Result<Self>;

    // Construction
    pub const fn from_nanos(timestamp_nanos: u128, rand: u64) -> Self;
    pub const fn from_u128(value: u128) -> Self;
    pub const fn from_bytes(bytes: [u8; 16]) -> Self;
    pub fn from_str(s: &str) -> Result<Self>;

    // Extraction
    pub const fn nanos(self) -> u128;                    // Nanoseconds
    pub const fn micros(self) -> u128;                   // Microseconds
    pub const fn millis(self) -> u128;                   // Milliseconds
    pub const fn random(self) -> u64;
    pub const fn parts(self) -> (u128, u64);

    // Conversion
    pub const fn as_u128(self) -> u128;
    pub const fn to_bytes(self) -> [u8; 16];
    pub fn encode(self, buf: &mut [u8; 26]);

    // UUID interoperability (with `uuid` feature)
    #[cfg(feature = "uuid")]
    pub fn to_uuid(self) -> uuid::Uuid;
    #[cfg(feature = "uuid")]
    pub fn from_uuid(uuid: uuid::Uuid) -> Self;

    // Time utilities
    pub fn datetime(self) -> SystemTime;
    pub fn duration_since_epoch(self) -> Duration;

    // Chrono DateTime (with `chrono` feature)
    #[cfg(feature = "chrono")]
    pub fn chrono_datetime(self) -> chrono::DateTime<chrono::Utc>;
    #[cfg(feature = "chrono")]
    pub fn from_chrono_datetime(dt: chrono::DateTime<chrono::Utc>) -> Result<Self>;

    // Utilities
    pub const fn nil() -> Self;
    pub const fn is_nil(self) -> bool;

    // Constants
    pub const MIN: Self;
    pub const MAX: Self;
    pub const ZERO: Self;
}

// Standard trait implementations for ergonomic conversions
impl From<u128> for Nulid { }
impl From<Nulid> for u128 { }
impl From<[u8; 16]> for Nulid { }
impl From<Nulid> for [u8; 16] { }
impl AsRef<u128> for Nulid { }
impl TryFrom<&[u8]> for Nulid { }

// UUID conversions (with `uuid` feature)
#[cfg(feature = "uuid")]
impl From<uuid::Uuid> for Nulid { }
#[cfg(feature = "uuid")]
impl From<Nulid> for uuid::Uuid { }

// Standard traits
impl Display for Nulid { }
impl FromStr for Nulid { }
impl Ord for Nulid { }
impl Default for Nulid { }  // Returns Nulid::ZERO
```

### Generator

```rust,ignore
// Unified generator with injectable dependencies
pub struct Generator<C: Clock = SystemClock, R: Rng = CryptoRng, N: NodeId = NoNodeId> { }

impl Generator<SystemClock, CryptoRng, NoNodeId> {
    pub const fn new() -> Self;                    // Production single-node
}

impl Generator<SystemClock, CryptoRng, WithNodeId> {
    pub fn with_node_id(node_id: u16) -> Self;     // Production distributed
}

impl<C: Clock, R: Rng, N: NodeId> Generator<C, R, N> {
    pub fn with_deps(clock: C, rng: R) -> Self;    // Testing
    pub fn with_deps_and_node_id(clock: C, rng: R, node_id: N) -> Self;
    pub fn generate(&self) -> Result<Nulid>;
    pub fn last(&self) -> Option<Nulid>;
    pub fn reset(&self);
    pub fn node_id(&self) -> Option<u16>;
}

// Type aliases
pub type DefaultGenerator = Generator<SystemClock, CryptoRng, NoNodeId>;
pub type DistributedGenerator = Generator<SystemClock, CryptoRng, WithNodeId>;
```

### Clock and RNG Traits (for testing)

```rust,ignore
// Clock abstraction
pub trait Clock: Send + Sync {
    fn now_nanos(&self) -> Result<u128>;
}

pub struct SystemClock;      // Production: uses quanta
pub struct MockClock;        // Testing: controllable time

impl MockClock {
    pub fn new(initial_nanos: u64) -> Self;
    pub fn set(&self, nanos: u64);
    pub fn advance(&self, duration: Duration);
    pub fn regress(&self, duration: Duration);  // Simulate clock going backward
}

// RNG abstraction
pub trait Rng: Send + Sync {
    fn random_u64(&self) -> u64;
}

pub struct CryptoRng;        // Production: cryptographic RNG
pub struct SeededRng;        // Testing: reproducible sequences
pub struct SequentialRng;    // Debugging: 0, 1, 2, 3...

// Node ID abstraction
pub trait NodeId: Send + Sync + Default + Copy {
    fn get(&self) -> Option<u16>;
}

pub struct NoNodeId;         // Default: 60 bits random (ZST, zero overhead)
pub struct WithNodeId(u16);  // Distributed: 16 bits node + 44 bits random
```

### Error Handling

```rust,ignore
pub enum Error {
    RandomError,
    InvalidChar(char, usize),
    InvalidLength { expected: usize, found: usize },
    MutexPoisoned,
}

pub type Result<T> = std::result::Result<T, Error>;
```

---

## Cargo Features

- `default = ["std"]` - Standard library support
- `std` - Enable standard library features (`SystemTime`, etc.)
- `derive` - Enable `Id` derive macro for type-safe wrapper types (requires `nulid_derive`)
- `macros` - Enable `nulid!()` macro for convenient generation (requires `nulid_macros`)
- `serde` - Enable serialization/deserialization support (JSON, TOML, `MessagePack`, Bincode, etc.)
- `uuid` - Enable UUID interoperability (conversion to/from `uuid::Uuid`)
- `sqlx` - Enable `SQLx` `PostgreSQL` support (stores as UUID, requires `uuid` feature)
- `postgres-types` - Enable `PostgreSQL` `postgres-types` crate support
- `rkyv` - Enable zero-copy serialization support
- `chrono` - Enable `chrono::DateTime<Utc>` conversion support 

Examples:

```toml
# With serde (supports JSON, TOML, MessagePack, Bincode, etc.)
[dependencies]
nulid = { version = "0.7", features = ["serde"] }

# With UUID interoperability
[dependencies]
nulid = { version = "0.7", features = ["uuid"] }

# With derive macro for type-safe IDs
[dependencies]
nulid = { version = "0.7", features = ["derive"] }
nulid_derive = "0.7"

# With convenient nulid!() macro
[dependencies]
nulid = { version = "0.7", features = ["macros"] }

# With both derive and macros
[dependencies]
nulid = { version = "0.7", features = ["derive", "macros"] }
nulid_derive = "0.7"

# With SQLx PostgreSQL support
[dependencies]
nulid = { version = "0.7", features = ["sqlx"] }

# With chrono DateTime support
[dependencies]
nulid = { version = "0.7", features = ["chrono"] }
 

# All features
[dependencies]
nulid = { version = "0.7", features = ["derive", "macros", "serde", "uuid", "sqlx", "postgres-types", "rkyv", "chrono"] }
nulid_derive = "0.7"
```

The `serde_example` demonstrates multiple formats including JSON, `MessagePack`, TOML, and Bincode:

```bash
# Run the serde examples (includes Bincode)
cargo run --example serde_example --features serde
```

For the `sqlx` example, see `examples/sqlx_postgres.rs`:

```bash
# Set up PostgreSQL database
export DATABASE_URL="postgresql://localhost/nulid_example"
createdb nulid_example

# Run the example
cargo run --example sqlx_postgres --features sqlx
```

---

## Security Considerations

1. **Cryptographically secure randomness** - Uses `rand` crate with system entropy for high-quality randomness
2. **Timestamp information is exposed** - NULIDs reveal when they were created (down to the nanosecond)
3. **Not for security purposes** - Use proper authentication/authorization mechanisms
4. **Collision resistance** - 60 bits of randomness provides strong collision resistance within the same nanosecond
5. **Memory safety** - Zero unsafe code, preventing memory-related vulnerabilities

---

## Development

### Building

```bash
cargo build --release
```

### Testing

```bash
cargo test
```

### Benchmarks

```bash
cargo bench
```

#### Results

| Operation                       | Time      | Throughput      |
| ------------------------------- | --------- | --------------- |
| Generate new NULID              | 11.78 ns  | 84.9M ops/sec   |
| From datetime                   | 14.11 ns  | 70.9M ops/sec   |
| Monotonic generation            | 20.96 ns  | 47.7M ops/sec   |
| Sequential generation (100 IDs) | 2.10 µs   | 47.5M IDs/sec   |
| Encode to string (array)        | 9.10 ns   | 110M ops/sec    |
| Encode to String (heap)         | 32.84 ns  | 30.5M ops/sec   |
| Decode from string              | 8.87 ns   | 113M ops/sec    |
| Round-trip string               | 42.04 ns  | 23.8M ops/sec   |
| Convert to bytes                | 293.75 ps | 3.40B ops/sec   |
| Convert from bytes              | 392.82 ps | 2.55B ops/sec   |
| Equality comparison             | 2.80 ns   | 357M ops/sec    |
| Ordering comparison             | 2.82 ns   | 355M ops/sec    |
| Sort 1000 IDs                   | 13.02 µs  | 76.8M elem/sec  |
| Concurrent (10 threads)         | 183.60 µs | 5.45K batch/sec |
| Batch generate 10               | 234.25 ns | 42.7M elem/sec  |
| Batch generate 100              | 2.23 µs   | 44.7M elem/sec  |
| Batch generate 1000             | 21.53 µs  | 46.4M elem/sec  |

_Benchmarked on Apple M2 Pro processor with `cargo bench`_

### Linting

```bash
cargo clippy -- -D warnings
```

---

## Background & Evolution

NULID builds upon the excellent [ULID specification](https://github.com/ulid/spec) and addresses:

- Millisecond precision limitation of ULID

NULID achieves:

- Nanosecond precision for high-throughput systems
- 128-bit size (UUID-compatible)
- Simple two-part design (timestamp + randomness)
- Lexicographic sortability
- Compact 26-character encoding

---

## Design Philosophy

1. **Simplicity** - Two parts (timestamp + random) instead of three
2. **Compatibility** - 128 bits like UUID, seamless interoperability
3. **Precision** - Nanosecond timestamps for modern systems
4. **Performance** - Optimized for performance (11.78ns generation, 9.1ns encoding)
5. **Safety** - Zero unsafe code, panic-free production paths, strict linting
6. **Reliability** - Comprehensive tests, memory-safe by design

---

## License

Licensed under the MIT License. See [LICENSE](https://github.com/kakilangit/nulid/blob/main/LICENSE) for details.

---

**Built with by developers who need nanosecond precision in 128 bits**

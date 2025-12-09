# NULID

**Nanosecond-Precision Universally Lexicographically Sortable Identifier**

[![Crates.io](https://img.shields.io/crates/v/nulid.svg)](https://crates.io/crates/nulid)
[![Documentation](https://docs.rs/nulid/badge.svg)](https://docs.rs/nulid)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/kakilangit/nulid/blob/main/LICENSE)
[![Rust Version](https://img.shields.io/badge/rust-1.85%2B-blue.svg)](https://www.rust-lang.org)

---

## Overview

NULID is a 128-bit identifier with **nanosecond-precision timestamps** designed for high-throughput, distributed systems. It combines the simplicity of ULID with sub-millisecond precision for systems that require fine-grained temporal ordering.

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

✨ **128-bit identifier** (16 bytes) - UUID-compatible size  
⚡ **Blazing fast** - 35ns per ID generation (21x faster than v0.1)  
📊 **Lexicographically sortable** with nanosecond precision  
🔤 **26-character canonical encoding** using Crockford's Base32  
🕐 **Extended lifespan** - valid until year **~11,326 AD**  
🔒 **Memory safe** - zero unsafe code, panic-free production paths  
🌐 **URL safe** - no special characters  
⚙️ **Monotonic sort order** within the same nanosecond  
🔄 **UUID interoperability** - seamless conversion to/from UUID  
🎯 **1.15 quintillion unique IDs per nanosecond** (60 bits of randomness)

---

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
nulid = "0.2"
```

For UUID interoperability:

```toml
[dependencies]
nulid = { version = "0.2", features = ["uuid"] }
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
let timestamp_nanos = id.timestamp_nanos();  // u128: nanoseconds since epoch
let random = id.random();                     // u64: 60-bit random value

// Convert to/from bytes
let bytes = id.to_bytes();          // [u8; 16]
let id2 = Nulid::from_bytes(bytes);
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

### UUID Interoperability

With the optional `uuid` feature, you can seamlessly convert between NULID and UUID:

```rust
use nulid::Nulid;
use uuid::Uuid;

# #[cfg(feature = "uuid")]
# fn main() -> nulid::Result<()> {
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
# Ok(())
# }
# #[cfg(not(feature = "uuid"))]
# fn main() {}
```

This enables:

- **Database compatibility** - Store as UUID in Postgres, MySQL, etc.
- **API compatibility** - Accept/return UUIDs while using NULID internally
- **Migration path** - Gradually migrate from UUID to NULID
- **Interoperability** - Work with existing UUID-based systems

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

## 🛠️ Specification

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
- **Source:** Cryptographically secure randomness via `getrandom` crate
- **Collision Probability:** 1.15 × 10^18 unique values per nanosecond
- **Purpose:** Ensures uniqueness when multiple IDs are generated within the same nanosecond

### Total: 128 bits (16 bytes)

---

## 📝 Canonical String Representation

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

## 🔢 Sorting

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

## ⚙️ Monotonicity

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

## 🗂️ Binary Layout and Byte Order

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

- ✅ Natural lexicographic ordering (timestamp in most significant bits)
- ✅ Simple bit operations (just shift and mask)
- ✅ Maximum precision (nanosecond resolution)
- ✅ UUID compatibility (128 bits / 16 bytes)

---

## 📊 Comparison: ULID vs NULID

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

## 🚀 Performance & Safety

### Performance

- **21x faster generation** - Reduced from 704ns to 35ns per ID
- **2.8x faster encoding** - Optimized Base32 encoding (9.2ns)
- **Buffered RNG** - Uses `rand` crate for amortized cryptographic randomness
- **Zero-copy operations** - Minimal allocations and copies

### Safety Guarantees

- **Zero unsafe code** - Enforced with `#![forbid(unsafe_code)]`
- **Panic-free production paths** - All errors handled via `Result`
- **Strict linting** - Comprehensive clippy checks for safety
- **Memory safe** - No buffer overflows, no undefined behavior
- **Thread-safe** - Concurrent generation without data races

### Additional Features

- **Optional serde support** for serialization
- **Optional UUID interoperability** for seamless conversion
- **Thread-safe** monotonic generation
- **Comprehensive test coverage**
- **Optimized bit operations**

---

## 🎯 Use Cases

NULID is ideal for:

- **High-frequency trading systems** requiring nanosecond-level event ordering
- **Distributed databases** with high write throughput (UUID-compatible storage)
- **Event sourcing systems** where precise ordering is critical
- **Microservices architectures** generating many concurrent IDs
- **IoT platforms** processing millions of sensor readings per second
- **Real-time analytics** systems requiring precise event sequencing
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
    pub const fn from_timestamp_nanos(timestamp_nanos: u128, rand: u64) -> Self;
    pub const fn from_u128(value: u128) -> Self;
    pub const fn from_bytes(bytes: [u8; 16]) -> Self;
    pub fn from_str(s: &str) -> Result<Self>;

    // Extraction
    pub const fn timestamp_nanos(self) -> u128;
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

    // Utilities
    pub const fn nil() -> Self;
    pub const fn is_nil(self) -> bool;

    // Constants
    pub const MIN: Self;
    pub const MAX: Self;
    pub const ZERO: Self;
}

// UUID conversions (with `uuid` feature)
#[cfg(feature = "uuid")]
impl From<uuid::Uuid> for Nulid { }
#[cfg(feature = "uuid")]
impl From<Nulid> for uuid::Uuid { }

// Traits
impl Display for Nulid { }
impl FromStr for Nulid { }
impl Ord for Nulid { }
```

### Generator

```rust,ignore
pub struct Generator { }

impl Generator {
    pub const fn new() -> Self;
    pub fn generate(&self) -> Result<Nulid>;
    pub fn reset(&self);
    pub fn last(&self) -> Option<Nulid>;
}
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

## 📦 Cargo Features

- `default = ["std"]` - Standard library support
- `std` - Enable standard library features (`SystemTime`, etc.)
- `serde` - Enable serialization/deserialization support
- `uuid` - Enable UUID interoperability (conversion to/from `uuid::Uuid`)

Examples:

```toml
# With serde
[dependencies]
nulid = { version = "0.2", features = ["serde"] }

# With UUID interoperability
[dependencies]
nulid = { version = "0.2", features = ["uuid"] }

# With both
[dependencies]
nulid = { version = "0.2", features = ["serde", "uuid"] }
```

---

## 🔒 Security Considerations

1. **Cryptographically secure randomness** - Uses `rand` crate with system entropy for high-quality randomness
2. **Timestamp information is exposed** - NULIDs reveal when they were created (down to the nanosecond)
3. **Not for security purposes** - Use proper authentication/authorization mechanisms
4. **Collision resistance** - 60 bits of randomness provides strong collision resistance within the same nanosecond
5. **Memory safety** - Zero unsafe code, preventing memory-related vulnerabilities

---

## 🛠️ Development

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

#### Results (v0.2.0)

| Operation                       | Time     | Throughput      | vs v0.1.0       |
| ------------------------------- | -------- | --------------- | --------------- |
| Generate new NULID              | 35.03 ns | 28.5M ops/sec   | **21x faster**  |
| From datetime                   | 14.73 ns | 67.9M ops/sec   | -               |
| Monotonic generation            | 48.01 ns | 20.8M ops/sec   | **15x faster**  |
| Sequential generation (100 IDs) | 4.78 µs  | 20.9M IDs/sec   | **15x faster**  |
| Encode to string (array)        | 9.18 ns  | 109M ops/sec    | **2.9x faster** |
| Encode to String (heap)         | 33.49 ns | 29.9M ops/sec   | -               |
| Decode from string              | 8.81 ns  | 114M ops/sec    | 1.1x faster     |
| Round-trip string               | 43.38 ns | 23.1M ops/sec   | -               |
| Convert to bytes                | 295 ps   | 3.39B ops/sec   | ~same           |
| Convert from bytes              | 395 ps   | 2.53B ops/sec   | ~same           |
| Equality comparison             | 2.75 ns  | 364M ops/sec    | ~same           |
| Ordering comparison             | 2.74 ns  | 365M ops/sec    | ~same           |
| Sort 1000 IDs                   | 13.17 µs | 75.9M elem/sec  | 1.2x faster     |
| Concurrent (10 threads)         | 290 µs   | 3.45K batch/sec | -               |
| Batch generate 10               | 488 ns   | 20.5M elem/sec  | -               |
| Batch generate 100              | 4.82 µs  | 20.8M elem/sec  | -               |
| Batch generate 1000             | 48.1 µs  | 20.8M elem/sec  | -               |

**Key Improvements in v0.2.0:**

- **Generation:** 704ns → 35ns (21x faster) - Switched to buffered RNG
- **Encoding:** 26.78ns → 9.18ns (2.9x faster) - Optimized Base32 algorithm
- **Safety:** Removed all unsafe code and panics
- **Consistency:** Predictable performance across all batch sizes (~21M IDs/sec)

_Benchmarked on Apple M-series processor with `cargo bench`_

### Linting

```bash
cargo clippy -- -D warnings
```

---

## 📚 Background & Evolution

NULID builds upon the excellent [ULID specification](https://github.com/ulid/spec) and addresses:

- ❌ Millisecond precision limitation of ULID
- ❌ 150-bit size of previous NULID designs (not UUID-compatible)

NULID achieves:

- ✅ Nanosecond precision for high-throughput systems
- ✅ 128-bit size (UUID-compatible)
- ✅ Simple two-part design (timestamp + randomness)
- ✅ Lexicographic sortability
- ✅ Compact 26-character encoding

### Version 0.2.0 Highlights

- **21x performance boost** - Optimized RNG using buffered randomness
- **Memory safety** - Zero unsafe code, compiler-enforced
- **UUID interoperability** - Seamless conversion for database compatibility
- **Production-ready** - No panics in production paths
- **Simplified codebase** - Easier to audit and maintain

---

## Design Philosophy

1. **Simplicity** - Two parts (timestamp + random) instead of three
2. **Compatibility** - 128 bits like UUID, seamless interoperability
3. **Precision** - Nanosecond timestamps for modern systems
4. **Performance** - Optimized operations (35ns generation, 9ns encoding)
5. **Safety** - Zero unsafe code, panic-free production paths, strict linting
6. **Reliability** - Comprehensive tests, memory-safe by design

---

## 📜 License

Licensed under the MIT License. See [LICENSE](https://github.com/kakilangit/nulid/blob/main/LICENSE) for details.

---

**Built with ⚡ by developers who need nanosecond precision in 128 bits**

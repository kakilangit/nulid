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
⚡ **1.15 quintillion unique NULIDs per nanosecond** (60 bits of randomness)  
📊 **Lexicographically sortable** with nanosecond precision  
🔤 **26-character canonical encoding** using Crockford's Base32  
🕐 **Extended lifespan** - valid until year **~11,326 AD**  
🔒 **Case insensitive** for flexible string handling  
🌐 **URL safe** - no special characters  
⚙️ **Monotonic sort order** within the same nanosecond  
🚀 **Minimal dependencies** - only uses `std::time` and `getrandom`

---

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
nulid = "1.0"
```

---

## Quick Start

### Basic Usage

```rust
use nulid::Nulid;

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
```

### Monotonic Generation

```rust
use nulid::Generator;

let generator = Generator::new();

// Generate multiple IDs - guaranteed strictly increasing
let id1 = generator.generate()?;
let id2 = generator.generate()?;
let id3 = generator.generate()?;

assert!(id1 < id2);
assert!(id2 < id3);
```

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

let generator = Generator::new();

// Even if called within the same nanosecond
let id1 = generator.generate()?; // ...XYZ
let id2 = generator.generate()?; // ...XYZ + 1
let id3 = generator.generate()?; // ...XYZ + 2

assert!(id1 < id2 && id2 < id3);
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

## 🚀 Features

- **Minimal external dependencies** - only uses `std::time` and `getrandom`
- **Optional serde support** for serialization
- **Thread-safe** monotonic generation
- **No panics in production code** - all errors are handled via `Result`
- **Comprehensive test coverage**
- **Optimized bit operations**

---

## 🎯 Use Cases

NULID is ideal for:

- **High-frequency trading systems** requiring nanosecond-level event ordering
- **Distributed databases** with high write throughput
- **Event sourcing systems** where precise ordering is critical
- **Microservices architectures** generating many concurrent IDs
- **`IoT` platforms** processing millions of sensor readings per second
- **Real-time analytics** systems requiring precise event sequencing
- **Any system** needing UUID-sized IDs with nanosecond precision

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

To use with serde:

```toml
[dependencies]
nulid = { version = "1.0", features = ["serde"] }
```

---

## 🔒 Security Considerations

1. **Cryptographically secure randomness** - Uses `getrandom` crate for high-quality entropy
2. **Timestamp information is exposed** - NULIDs reveal when they were created (down to the nanosecond)
3. **Not for security purposes** - Use proper authentication/authorization mechanisms
4. **Collision resistance** - 60 bits of randomness provides strong collision resistance within the same nanosecond

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

#### Results

| Operation                       | Time      | Throughput      |
| ------------------------------- | --------- | --------------- |
| Generate new NULID              | 704.81 ns | 1.42M ops/sec   |
| Monotonic generation            | 702.14 ns | 1.42M ops/sec   |
| Sequential generation (100 IDs) | 70.24 µs  | 1.42M IDs/sec   |
| Encode to string                | 26.78 ns  | 37.3M ops/sec   |
| Decode from string              | 9.40 ns   | 106M ops/sec    |
| Convert to bytes                | 292.39 ps | 3.42B ops/sec   |
| Convert from bytes              | 389.58 ps | 2.57B ops/sec   |
| Equality comparison             | 2.76 ns   | 362M ops/sec    |
| Ordering comparison             | 2.73 ns   | 366M ops/sec    |
| Sort 1000 IDs                   | 11.06 µs  | 90.4K sorts/sec |

### Linting

```bash
cargo clippy -- -D warnings
```

---

## 📚 Background

NULID builds upon the excellent [ULID specification](https://github.com/ulid/spec) and addresses:

- ❌ Millisecond precision limitation of ULID
- ❌ 150-bit size of previous NULID designs (not UUID-compatible)

NULID achieves:

- ✅ Nanosecond precision for high-throughput systems
- ✅ 128-bit size (UUID-compatible)
- ✅ Simple two-part design (timestamp + randomness)
- ✅ Lexicographic sortability
- ✅ Compact 26-character encoding

---

## Design Philosophy

1. **Simplicity** - Two parts (timestamp + random) instead of three
2. **Compatibility** - 128 bits like UUID
3. **Precision** - Nanosecond timestamps for modern systems
4. **Performance** - Optimized bit operations, zero-copy where possible
5. **Safety** - No panics in production code, comprehensive error handling

---

## 📜 License

Licensed under the MIT License. See [LICENSE](https://github.com/kakilangit/nulid/blob/main/LICENSE) for details.

---

**Built with ⚡ by developers who need nanosecond precision in 128 bits**

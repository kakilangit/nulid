# NULID

**Nanosecond-Precision Universally Lexicographically Sortable Identifier**

[![Crates.io](https://img.shields.io/crates/v/nulid.svg)](https://crates.io/crates/nulid)
[![Documentation](https://docs.rs/nulid/badge.svg)](https://docs.rs/nulid)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Rust Version](https://img.shields.io/badge/rust-1.91%2B-blue.svg)](https://www.rust-lang.org)

---

## Overview

NULID is an extension of [ULID](https://github.com/ulid/spec) that provides **nanosecond-precision timestamps** for high-throughput, distributed systems. While the original ULID is optimal for many use-cases, some high-concurrency systems require finer granularity for true chronological sorting and enhanced collision resistance.

### Why NULID?

**The Challenge:**

- ULID's 48-bit millisecond timestamp is insufficient for high-throughput, distributed systems that generate thousands of IDs within the same millisecond
- In systems processing millions of operations per second, millisecond precision can lead to sorting ambiguities

**The Solution:**

- NULID uses a **70-bit nanosecond timestamp** for precise chronological ordering
- Preserves ULID's robust **80-bit randomness** for collision resistance
- Maintains all the benefits of ULID while extending precision

### Features

✨ **150-bit identifier** (18.75 bytes) for maximum feature set  
⚡ **1.21e+24 unique NULIDs per nanosecond** (80 bits of randomness)  
📊 **Lexicographically sortable** with nanosecond precision  
🔤 **30-character canonical encoding** using Crockford's Base32  
🕐 **Extended lifespan** — valid until **~45,526 AD** (4× longer than ULID)  
🔒 **Case insensitive** for flexible string handling  
🌐 **URL safe** — no special characters  
⚙️ **Monotonic sort order** within the same nanosecond

---

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
nulid = "0.1"
```

## Performance

Benchmark results measured on modern hardware with `cargo bench`:

| Operation                        | Time    | Throughput          |
| -------------------------------- | ------- | ------------------- |
| **Generation**                   | ~1.1 µs | ~900,000 NULIDs/sec |
| **String Encoding**              | ~71 ns  | -                   |
| **String Decoding**              | ~97 ns  | -                   |
| **String Round-trip**            | ~168 ns | -                   |
| **Byte Serialization**           | ~0.9 ns | -                   |
| **Byte Deserialization**         | ~1.5 ns | -                   |
| **Byte Round-trip**              | ~2.1 ns | -                   |
| **Equality Check**               | ~1.3 ns | -                   |
| **Ordering Check**               | ~1.0 ns | -                   |
| **Sort 1,000 NULIDs**            | ~2.3 µs | 436 Melem/s         |
| **Batch (1,000)**                | ~1.1 ms | 900K NULIDs/sec     |
| **Concurrent (10 threads, 10K)** | ~3.3 ms | -                   |
| **Serde JSON Serialize**         | ~104 ns | -                   |
| **Serde JSON Deserialize**       | ~132 ns | -                   |
| **Serde JSON Round-trip**        | ~237 ns | -                   |

Key performance characteristics:

- ⚡ **Sub-microsecond generation** - ~900K IDs per second
- 🚀 **Sub-nanosecond byte operations** - extremely fast binary serialization
- 📦 **~71 ns string encoding** - efficient Base32 encoding
- 🔄 **~237 ns JSON round-trip** - fast serde integration
- 🔒 **Thread-safe** - concurrent generation across multiple threads
- 💾 **Zero-allocation hot paths** - minimal memory overhead

## Quick Start

```rust
use nulid::Nulid;

// Generate a new NULID
let id = Nulid::new()?;
println!("{}", id); // 01GZTV7EQ056J0E6N276XD6F3DNGMY
# Ok::<(), nulid::Error>(())
```

## Usage Examples

### Basic Generation

```rust
use nulid::Nulid;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Generate a new NULID
    let id = Nulid::new()?;
    println!("Generated NULID: {}", id);

    // Convert to string (30 characters)
    let id_string = id.to_string();
    println!("String: {}", id_string); // 01GZTV7EQ056J0E6N276XD6F3DNGMY

    // Parse from string (case-insensitive)
    let parsed: Nulid = id_string.parse()?;
    assert_eq!(id, parsed);

    Ok(())
}
```

### Byte Serialization

```rust
use nulid::Nulid;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let id = Nulid::new()?;

    // Convert to bytes (19 bytes)
    let bytes = id.to_bytes();
    println!("Bytes: {:02X?}", bytes);

    // Reconstruct from bytes
    let restored = Nulid::from_bytes(&bytes)?;
    assert_eq!(id, restored);

    Ok(())
}
```

### Lexicographic Sorting

```rust
use nulid::Nulid;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut ids = vec![];
    for _ in 0..5 {
        ids.push(Nulid::new()?);
    }

    // NULIDs are naturally sortable by timestamp
    ids.sort();

    // Verify chronological order
    assert!(ids.windows(2).all(|w| w[0] < w[1]));

    Ok(())
}
```

---

## 🛠️ Specification

The NULID is a **150-bit** (18.75 byte) binary identifier composed of:

```
70_bit_time_high_precision                    80_bit_randomness
|--------------------------------|            |--------------------------------|
           Timestamp                                    Randomness
            70 bits                                       80 bits
```

### Components

#### **Timestamp** (70 bits)

- **Size:** 70-bit integer
- **Representation:** UNIX time in **nanoseconds** (ns)
- **Rationale:** 70 bits provides 4× the lifespan of the original 68-bit design — valid until the year **45,526 AD**
- **Encoding:** Most Significant Bits (MSB) first to ensure lexicographical sortability based on time

#### **Randomness** (80 bits)

- **Size:** 80 bits
- **Source:** Cryptographically secure source of randomness (when possible)
- **Rationale:** Preserves ULID's collision resistance with **1.21e+24** unique values per nanosecond
- **Collision Probability:** Astronomically low, even at extreme throughput

---

## 📝 Canonical String Representation

```
tttttttttttttt rrrrrrrrrrrrrrrr
```

where:

- **`t`** = Timestamp (14 characters)
- **`r`** = Randomness (16 characters)

**Total Length:** 30 characters

### Encoding

NULID uses **Crockford's Base32** encoding, preserving the original ULID alphabet:

```
0123456789ABCDEFGHJKMNPQRSTVWXYZ
```

**Character Exclusions:** The letters `I`, `L`, `O`, and `U` are excluded to avoid confusion and abuse.

#### Encoding Breakdown

| Component  | Bits    | Characters | Calculation |
| ---------- | ------- | ---------- | ----------- |
| Timestamp  | 70      | 14         | ⌈70 ÷ 5⌉    |
| Randomness | 80      | 16         | ⌈80 ÷ 5⌉    |
| **Total**  | **150** | **30**     | ⌈150 ÷ 5⌉   |

---

## 🔢 Sorting

NULIDs are **lexicographically sortable**:

- The **left-most character** is sorted first
- The **right-most character** is sorted last
- Nanosecond precision ensures IDs are sorted correctly even when multiple IDs are generated within the same millisecond

### Example Sort Order

```
7VVV09D8H01ARZ3NDEKTSV4RRFFQ69G5FAV  ← Earlier
7VVV09D8H01ARZ3NDEKTSV4RRFFQ69G5FAW
7VVV09D8H01ARZ3NDEKTSV4RRFFQ69G5FAX
7VVV09D8H01ARZ3NDEKTSV4RRFFQ69G5FAY  ← Later
```

---

## ⚙️ Monotonicity

When generating multiple NULIDs within the same nanosecond:

1. The **80-bit random component** is treated as a **monotonic counter**
2. It increments by **1 bit** in the least significant bit position (with carrying)
3. This ensures deterministic sort order within the same nanosecond

### Example

```rust
use nulid::Nulid;

// Assume these calls occur within the same nanosecond
Nulid::new(); // 7VVV09D8H01ARZ3NDEKTSV4RRFFQ69G5FAV
Nulid::new(); // 7VVV09D8H01ARZ3NDEKTSV4RRFFQ69G5FAW
Nulid::new(); // 7VVV09D8H01ARZ3NDEKTSV4RRFFQ69G5FAX
```

### Overflow Condition

If more than **2^80** NULIDs are generated within the same nanosecond (an extremely unlikely scenario), the generation will fail with an overflow error.

```rust
// After 2^80 generations in the same nanosecond:
Nulid::new(); // panics with: "NULID overflow!"
```

---

## 🗂️ Binary Layout and Byte Order

The NULID components are encoded as **19 bytes (150 bits used, with 2 bits reserved)** with the **Most Significant Byte (MSB) first** (network byte order).

### Structure

```
Byte:     0       1       2       3       4       5       6       7       8       9      ...     18
      +-------+-------+-------+-------+-------+-------+-------+-------+-------+-------+-------+
Bits: |RR|   Timestamp (70 bits)      |  T|R  |    Randomness (80 bits)               ...
      +-------+-------+-------+-------+-------+-------+-------+-------+-------+-------+-------+
       ^^
       Reserved (2 bits) - must be 0
```

**Detailed Layout:**

- **Byte 0 (bits 0-1):** 2 reserved bits (must be 0)
- **Bytes 0-8:** 70-bit timestamp (using remaining 70 bits)
- **Bytes 9-18:** 80-bit randomness

**Reserved Bits:**

- The 2 most significant bits in byte 0 are reserved for future use
- Current specification requires these bits to be set to `00`
- Future versions may define meaning for these bits
- Decoders SHOULD accept any value but MUST preserve them

This structure ensures:

- ✅ Maximum chronological sortability (nanosecond precision)
- ✅ Maximum lifespan (valid until 45,526 AD)
- ✅ Maximum collision resistance (80 bits of randomness)
- ✅ Future extensibility (2 reserved bits)

---

## 📊 Comparison: ULID vs NULID

| Feature               | ULID              | NULID            |
| --------------------- | ----------------- | ---------------- |
| **Total Bits**        | 128               | 150              |
| **String Length**     | 26 chars          | 30 chars         |
| **Timestamp Bits**    | 48 (milliseconds) | 70 (nanoseconds) |
| **Randomness Bits**   | 80                | 80               |
| **Time Precision**    | 1 millisecond     | 1 nanosecond     |
| **Lifespan**          | Until 10889 AD    | Until 45,526 AD  |
| **IDs per Time Unit** | 1.21e+24 / ms     | 1.21e+24 / ns    |
| **Sortable**          | ✅                | ✅               |
| **Monotonic**         | ✅                | ✅               |
| **URL Safe**          | ✅                | ✅               |

---

## 🚀 Features

- **Zero dependencies** for core functionality
- **Optional serde support** for serialization
- **Thread-safe** monotonic generation
- **No unsafe code**
- **Comprehensive test coverage**
- **Benchmark suite** included

## 🎯 Use Cases

NULID is ideal for:

- **High-frequency trading systems** requiring nanosecond-level event ordering
- **Distributed databases** with high write throughput
- **Event sourcing systems** where precise ordering is critical
- **Microservices architectures** generating many concurrent IDs
- **IoT platforms** processing millions of sensor readings per second
- **Real-time analytics** systems requiring precise event sequencing

---

## 🔒 Security Considerations

1. **Use cryptographically secure random number generators** when possible
2. **Do not rely on NULID for security purposes** — use proper authentication/authorization
3. **Timestamp information is exposed** — NULIDs reveal when they were created
4. **Randomness must be unpredictable** — avoid weak PRNG implementations

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

## 📚 Background

NULID builds upon the excellent work of the [ULID specification](https://github.com/ulid/spec). The original ULID addressed many shortcomings of UUID:

- ❌ UUID v1/v2 requires access to MAC addresses
- ❌ UUID v3/v5 requires unique seeds and produces random distribution
- ❌ UUID v4 provides no temporal information
- ❌ UUID uses inefficient encoding (36 characters for 128 bits)

NULID extends this foundation by addressing the millisecond precision limitation while maintaining ULID's core benefits.

---

## 📦 Cargo Features

- `default` - Core NULID functionality
- `serde` - Enable serialization/deserialization support
- `std` - Standard library support (enabled by default)

To use with serde:

```toml
[dependencies]
nulid = { version = "0.1", features = ["serde"] }
```

---

## 📜 License

Licensed under the MIT License. See [LICENSE](LICENSE) for details.

---

## 🤝 Contributing

Contributions, suggestions, and discussions are welcome! Please open an issue or pull request to contribute to the NULID specification.

---

## 🔗 Related Projects

- [ULID Specification](https://github.com/ulid/spec)
- [ULID Rust Implementation](https://github.com/dylanhart/ulid-rs)

---

**Built with ⚡ by developers who need nanosecond precision**

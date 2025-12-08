# NULID

**Nanosecond-Precision Universally Lexicographically Sortable Identifier**

[![Crates.io](https://img.shields.io/crates/v/nulid.svg)](https://crates.io/crates/nulid)
[![Documentation](https://docs.rs/nulid/badge.svg)](https://docs.rs/nulid)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

---

## Overview

NULID is an extension of [ULID](https://github.com/ulid/spec) that provides **nanosecond-precision timestamps** for high-throughput, distributed systems. While the original ULID is optimal for many use-cases, some high-concurrency systems require finer granularity for true chronological sorting and enhanced collision resistance.

### Why NULID?

**The Challenge:**

- ULID's 48-bit millisecond timestamp is insufficient for high-throughput, distributed systems that generate thousands of IDs within the same millisecond
- In systems processing millions of operations per second, millisecond precision can lead to sorting ambiguities

**The Solution:**

- NULID uses a **68-bit nanosecond timestamp** for precise chronological ordering
- Preserves ULID's robust **80-bit randomness** for collision resistance
- Maintains all the benefits of ULID while extending precision

### Features

✨ **148-bit identifier** (18.5 bytes) for maximum feature set  
⚡ **1.21e+24 unique NULIDs per nanosecond** (80 bits of randomness)  
📊 **Lexicographically sortable** with nanosecond precision  
🔤 **30-character canonical encoding** using Crockford's Base32  
🕐 **Extended lifespan** — valid until **~10889 AD** (matching original ULID)  
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

## Quick Start

```rust
use nulid::Nulid;

// Generate a new NULID
let id = Nulid::new();
println!("{}", id); // 7VVV09D8H01ARZ3NDEKTSV4RRFFQ69G5FAV
```

## Usage Examples

### Basic Generation

```rust
use nulid::Nulid;

fn main() {
    // Generate a new NULID
    let id = Nulid::new();
    println!("Generated NULID: {}", id);

    // Convert to string
    let id_string = id.to_string();

    // Parse from string
    let parsed = Nulid::from_string(&id_string).unwrap();
    assert_eq!(id, parsed);
}
```

### Monotonic Generation

```rust
use nulid::Nulid;

fn main() {
    // Create a monotonic generator
    let mut generator = Nulid::monotonic();

    // Generate multiple IDs - guaranteed to be sorted
    let id1 = generator.generate();
    let id2 = generator.generate();
    let id3 = generator.generate();

    assert!(id1 < id2);
    assert!(id2 < id3);
}
```

### Working with Timestamps

```rust
use nulid::Nulid;
use std::time::SystemTime;

fn main() {
    // Create NULID with specific timestamp
    let timestamp = SystemTime::now();
    let id = Nulid::with_timestamp(timestamp);

    // Extract timestamp from NULID
    let extracted_time = id.timestamp();
    println!("NULID created at: {:?}", extracted_time);
}
```

### Serialization (with serde)

```rust
use nulid::Nulid;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct Event {
    id: Nulid,
    name: String,
}

fn main() {
    let event = Event {
        id: Nulid::new(),
        name: "User Login".to_string(),
    };

    let json = serde_json::to_string(&event).unwrap();
    println!("{}", json);
}
```

---

## 🛠️ Specification

The NULID is a **148-bit** (18.5 byte) binary identifier composed of:

```
68_bit_time_high_precision                    80_bit_randomness
|--------------------------------|            |--------------------------------|
           Timestamp                                    Randomness
            68 bits                                       80 bits
```

### Components

#### **Timestamp** (68 bits)

- **Size:** 68-bit integer
- **Representation:** UNIX time in **nanoseconds** (ns)
- **Rationale:** 68 bits provides the same lifespan as ULID's 48-bit millisecond timestamp — valid until the year **10889 AD**
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
| Timestamp  | 68      | 14         | ⌈68 ÷ 5⌉    |
| Randomness | 80      | 16         | ⌈80 ÷ 5⌉    |
| **Total**  | **148** | **30**     | ⌈148 ÷ 5⌉   |

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

The NULID components are encoded as **18.5 octets (148 bits)** with the **Most Significant Byte (MSB) first** (network byte order).

### Structure

```
Byte:     0       1       2       3       4       5       6       7       8       9      ...
      +-------+-------+-------+-------+-------+-------+-------+-------+-------+-------+
Bits: |      Timestamp (68 bits)      |  T|R  |    Randomness (80 bits)      ...
      +-------+-------+-------+-------+-------+-------+-------+-------+-------+-------+
```

**Detailed Layout:**

- **Bytes 0-7:** First 64 bits of timestamp
- **Byte 8 (high nibble):** Last 4 bits of timestamp
- **Byte 8 (low nibble):** First 4 bits of randomness (MSBs)
- **Bytes 9-18:** Remaining 76 bits of randomness

This structure ensures:

- ✅ Maximum chronological sortability (nanosecond precision)
- ✅ Maximum lifespan (valid until 10889 AD)
- ✅ Maximum collision resistance (80 bits of randomness)

---

## 📊 Comparison: ULID vs NULID

| Feature               | ULID              | NULID            |
| --------------------- | ----------------- | ---------------- |
| **Total Bits**        | 128               | 148              |
| **String Length**     | 26 chars          | 30 chars         |
| **Timestamp Bits**    | 48 (milliseconds) | 68 (nanoseconds) |
| **Randomness Bits**   | 80                | 80               |
| **Time Precision**    | 1 millisecond     | 1 nanosecond     |
| **Lifespan**          | Until 10889 AD    | Until 10889 AD   |
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

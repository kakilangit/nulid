//! NULID: Nanosecond-Precision Universally Lexicographically Sortable Identifier
//!
//! NULID is an extension of ULID that provides nanosecond-precision timestamps
//! for high-throughput, distributed systems.
//!
//! # Features
//!
//! - 150-bit identifiers (18.75 bytes)
//! - 70-bit nanosecond timestamp (valid until ~45526 AD)
//! - 80-bit cryptographic randomness
//! - Lexicographically sortable
//! - 30-character Base32 encoding
//! - Monotonic generation support
//!
//! # Example
//!
//! ```rust
//! use nulid::Nulid;
//!
//! // Generate a new NULID
//! let id = Nulid::new().unwrap();
//! println!("{}", id);
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]

pub mod base32;
pub mod error;
pub mod generator;
pub mod nulid;
pub mod randomness;
pub mod timestamp;

pub use error::{Error, Result};
pub use generator::Generator;
pub use nulid::Nulid;
pub use randomness::Random;
pub use timestamp::Timestamp;

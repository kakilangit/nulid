//! NULID: Nanosecond-Precision Universally Lexicographically Sortable Identifier
//!
//! NULID is an extension of ULID that provides nanosecond-precision timestamps
//! for high-throughput, distributed systems.
//!
//! # Features
//!
//! - 148-bit identifiers (18.5 bytes)
//! - 68-bit nanosecond timestamp
//! - 80-bit cryptographic randomness
//! - Lexicographically sortable
//! - 30-character Base32 encoding
//! - Monotonic generation support
//!
//! # Example
//!
//! ```rust,ignore
//! use nulid::Nulid;
//!
//! // Generate a new NULID
//! let id = Nulid::new();
//! println!("{}", id);
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]

pub mod error;
pub mod timestamp;

pub use error::{Error, Result};
pub use timestamp::Timestamp;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_module_exists() {
        let err = Error::InvalidTimestamp;
        assert_eq!(err, Error::InvalidTimestamp);
    }
}

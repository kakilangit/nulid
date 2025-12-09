#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::unwrap_used)]
#![warn(clippy::expect_used)]
#![warn(clippy::panic)]
#![warn(clippy::unimplemented)]
#![warn(clippy::todo)]

//! NULID: Nanosecond-Precision Universally Lexicographically Sortable Identifier
//!
//! A 128-bit identifier with nanosecond-precision timestamps designed for
//! high-throughput, distributed systems.

pub mod base32;
pub mod error;
pub mod generator;
pub mod nulid;
pub mod time;

pub use error::{Error, Result};
pub use generator::Generator;
pub use nulid::Nulid;

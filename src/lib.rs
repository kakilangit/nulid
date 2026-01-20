#![doc = include_str!("../README.md")]
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

pub mod features;

pub use error::{Error, Result};
pub use generator::{
    // Clock trait and implementations
    Clock,
    CryptoRng,
    // Type aliases
    DefaultGenerator,
    DistributedGenerator,
    // Main generator type
    Generator,
    MockClock,
    NoNodeId,
    // NodeId trait and implementations
    NodeId,
    // Rng trait and implementations
    Rng,
    SeededRng,
    SequentialRng,
    SystemClock,
    WithNodeId,
};
pub use nulid::Nulid;

#[cfg(feature = "derive")]
pub use nulid_derive::Id;

#[cfg(feature = "macros")]
pub use nulid_macros::nulid;

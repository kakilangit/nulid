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

/// Paths the `Id` derive generates code against, so a crate using it needs no
/// direct dependency on the optional crates. Not public API.
#[doc(hidden)]
pub mod __private {
    #[cfg(feature = "postgres-types")]
    pub use bytes;
    #[cfg(feature = "chrono")]
    pub use chrono;
    #[cfg(feature = "jiff")]
    pub use jiff;
    #[cfg(feature = "postgres-types")]
    pub use postgres_types;
    #[cfg(feature = "serde")]
    pub use serde;
    #[cfg(any(
        feature = "sqlx-postgres",
        feature = "sqlx-sqlite",
        feature = "sqlx-mysql"
    ))]
    pub use sqlx_core;
    #[cfg(feature = "sqlx-mysql")]
    pub use sqlx_mysql;
    #[cfg(feature = "sqlx-postgres")]
    pub use sqlx_postgres;
    #[cfg(feature = "sqlx-sqlite")]
    pub use sqlx_sqlite;
    #[cfg(feature = "uuid")]
    pub use uuid;
}

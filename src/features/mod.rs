//! Third-party feature integrations for NULID.
//!
//! This module contains optional integrations with external crates:
//! - `uuid`: UUID interoperability (conversion to/from `uuid::Uuid`)
//! - `sqlx`: `PostgreSQL` database support via `SQLx`
//! - `serde`: Serialization/deserialization support
//! - `postgres-types`: `PostgreSQL` type support via `postgres-types`
//! - `rkyv`: Zero-copy serialization support
//! - `chrono`: `chrono::DateTime<Utc>` support
//! - `jiff`: `jiff::Timestamp` support

#[cfg(feature = "uuid")]
pub mod uuid;

#[cfg(feature = "sqlx")]
pub mod sqlx;

#[cfg(feature = "serde")]
pub mod serde;

#[cfg(feature = "postgres-types")]
pub mod postgres_types;

#[cfg(feature = "rkyv")]
pub mod rkyv;

#[cfg(feature = "chrono")]
pub mod chrono;

#[cfg(feature = "jiff")]
pub mod jiff;

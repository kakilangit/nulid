//! Third-party feature integrations for NULID.
//!
//! This module contains optional integrations with external crates:
//! - `uuid`: UUID interoperability (conversion to/from `uuid::Uuid`)
//! - `sqlx`: `PostgreSQL` database support via `SQLx`
//! - `serde`: Serialization/deserialization support

#[cfg(feature = "uuid")]
pub mod uuid;

#[cfg(feature = "sqlx")]
pub mod sqlx;

#[cfg(feature = "serde")]
pub mod serde;

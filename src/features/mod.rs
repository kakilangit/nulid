//! Third-party feature integrations for NULID.
//!
//! This module contains optional integrations with external crates:
//! - `uuid`: UUID interoperability (conversion to/from [`uuid::Uuid`])
//! - `sqlx-postgres`: `PostgreSQL` database support via `SQLx`
//! - `sqlx-sqlite`: `SQLite` database support via `SQLx`
//! - `sqlx-mysql`: `MySQL`/`MariaDB` database support via `SQLx`
//! - `serde`: Serialization/deserialization support
//! - `postgres-types`: `PostgreSQL` type support via `postgres-types`
//! - `rkyv`: Zero-copy serialization support
//! - `jiff`: [`jiff::Timestamp`] support

#[cfg(feature = "uuid")]
pub mod uuid;

#[cfg(all(feature = "sqlx-postgres-uuid", feature = "sqlx-postgres-bytea"))]
compile_error!(
    "features `sqlx-postgres-uuid` and `sqlx-postgres-bytea` are mutually exclusive: \
     pick the storage mapping matching the id column type (UUID or BYTEA)"
);

#[cfg(any(feature = "sqlx-postgres-uuid", feature = "sqlx-postgres-bytea"))]
pub mod sqlx_postgres;

#[cfg(feature = "sqlx-sqlite")]
pub mod sqlx_sqlite;

#[cfg(feature = "sqlx-mysql")]
pub mod sqlx_mysql;

#[cfg(feature = "serde")]
pub mod serde;

#[cfg(feature = "postgres-types")]
pub mod postgres_types;

#[cfg(feature = "rkyv")]
pub mod rkyv;

#[cfg(feature = "jiff")]
pub mod jiff;

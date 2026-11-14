//! `SQLx` support for `PostgreSQL` storage.
//!
//! Two storage mappings, chosen by feature:
//!
//! - default (`sqlx-postgres`): NULIDs are stored as **UUIDs**
//! - `sqlx-postgres-bytea`: NULIDs are stored as **16-byte `BYTEA`** values
//!
//! The mappings are mutually exclusive — both implement `Type`, `Encode` and
//! `Decode` for [`Nulid`], and Rust rejects conflicting impls. Pick one; a
//! column's type decides which one you need, and the encoded bytes are the
//! same 16 bytes either way.
//!
//! # Example
//!
//! ```ignore
//! use nulid::Nulid;
//! use sqlx::{PgPool, postgres::PgRow, Row};
//!
//! #[derive(sqlx::FromRow)]
//! struct User {
//!     id: Nulid,
//!     name: String,
//! }
//!
//! async fn insert_user(pool: &PgPool, id: Nulid, name: &str) -> sqlx::Result<()> {
//!     sqlx::query("INSERT INTO users (id, name) VALUES ($1, $2)")
//!         .bind(id)
//!         .bind(name)
//!         .execute(pool)
//!         .await?;
//!     Ok(())
//! }
//!
//! async fn get_user(pool: &PgPool, id: Nulid) -> sqlx::Result<User> {
//!     sqlx::query_as::<_, User>("SELECT id, name FROM users WHERE id = $1")
//!         .bind(id)
//!         .fetch_one(pool)
//!         .await
//! }
//! ```

#[cfg(feature = "sqlx-postgres-bytea")]
mod bytea_mode;

#[cfg(not(feature = "sqlx-postgres-bytea"))]
mod uuid_mode;

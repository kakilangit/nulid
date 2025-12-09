//! SQLx support for PostgreSQL UUID storage.
//!
//! This module provides implementations for storing NULIDs as UUIDs in PostgreSQL
//! databases using the sqlx crate.
//!
//! # Examples
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

use crate::Nulid;
use sqlx::encode::IsNull;
use sqlx::error::BoxDynError;
use sqlx::postgres::{PgArgumentBuffer, PgHasArrayType, PgTypeInfo, PgValueRef, Postgres};
use sqlx::{Decode, Encode, Type};
use uuid::Uuid;

impl Type<Postgres> for Nulid {
    fn type_info() -> PgTypeInfo {
        <Uuid as Type<Postgres>>::type_info()
    }

    fn compatible(ty: &PgTypeInfo) -> bool {
        <Uuid as Type<Postgres>>::compatible(ty)
    }
}

impl PgHasArrayType for Nulid {
    fn array_type_info() -> PgTypeInfo {
        <Uuid as PgHasArrayType>::array_type_info()
    }

    fn array_compatible(ty: &PgTypeInfo) -> bool {
        <Uuid as PgHasArrayType>::array_compatible(ty)
    }
}

impl<'q> Encode<'q, Postgres> for Nulid {
    fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> Result<IsNull, BoxDynError> {
        let uuid = self.to_uuid();
        <Uuid as Encode<Postgres>>::encode_by_ref(&uuid, buf)
    }
}

impl<'r> Decode<'r, Postgres> for Nulid {
    fn decode(value: PgValueRef<'r>) -> Result<Self, BoxDynError> {
        let uuid = <Uuid as Decode<Postgres>>::decode(value)?;
        Ok(Nulid::from_uuid(uuid))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_decode_roundtrip() {
        use crate::Nulid;

        let original = Nulid::new().expect("Failed to create NULID");

        // Convert to UUID and back to verify encoding path
        let uuid = original.to_uuid();
        let decoded = Nulid::from_uuid(uuid);

        assert_eq!(original, decoded);
        assert_eq!(original.timestamp_nanos(), decoded.timestamp_nanos());
        assert_eq!(original.random(), decoded.random());
    }

    #[test]
    fn test_nil_nulid() {
        let nil = Nulid::nil();
        let uuid = nil.to_uuid();
        let decoded = Nulid::from_uuid(uuid);

        assert!(decoded.is_nil());
        assert_eq!(decoded.timestamp_nanos(), 0);
        assert_eq!(decoded.random(), 0);
    }

    #[test]
    fn test_nulid_uuid_equivalence() {
        // Test that NULID and UUID store the same 128-bit value
        let original = Nulid::new().expect("Failed to create NULID");
        let uuid = original.to_uuid();

        // Convert UUID bytes back to NULID
        let uuid_bytes = uuid.as_bytes();
        let nulid_bytes = original.to_bytes();

        assert_eq!(uuid_bytes, &nulid_bytes);
    }
}

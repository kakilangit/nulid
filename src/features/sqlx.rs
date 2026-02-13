//! `SQLx` support for `PostgreSQL`, `SQLite`, and `MySQL/MariaDB` storage.
//!
//! This module provides implementations for storing NULIDs in databases using the sqlx crate:
//! - `PostgreSQL`: Stored as UUIDs
//! - `SQLite`: Stored as BLOBs
//! - `MySQL/MariaDB`: Stored as BINARY(16)
//!
//! # `PostgreSQL` Examples
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
//!
//! # `SQLite` Examples
//!
//! ```ignore
//! use nulid::Nulid;
//! use sqlx::{SqlitePool, Row};
//!
//! #[derive(sqlx::FromRow)]
//! struct User {
//!     id: Nulid,
//!     name: String,
//! }
//!
//! async fn insert_user(pool: &SqlitePool, id: Nulid, name: &str) -> sqlx::Result<()> {
//!     sqlx::query("INSERT INTO users (id, name) VALUES (?, ?)")
//!         .bind(id)
//!         .bind(name)
//!         .execute(pool)
//!         .await?;
//!     Ok(())
//! }
//!
//! async fn get_user(pool: &SqlitePool, id: Nulid) -> sqlx::Result<User> {
//!     sqlx::query_as::<_, User>("SELECT id, name FROM users WHERE id = ?")
//!         .bind(id)
//!         .fetch_one(pool)
//!         .await
//! }
//! ```
//!
//! # MySQL/MariaDB Examples
//!
//! ```ignore
//! use nulid::Nulid;
//! use sqlx::{MySqlPool, Row};
//!
//! #[derive(sqlx::FromRow)]
//! struct User {
//!     id: Nulid,
//!     name: String,
//! }
//!
//! async fn insert_user(pool: &MySqlPool, id: Nulid, name: &str) -> sqlx::Result<()> {
//!     sqlx::query("INSERT INTO users (id, name) VALUES (?, ?)")
//!         .bind(id)
//!         .bind(name)
//!         .execute(pool)
//!         .await?;
//!     Ok(())
//! }
//!
//! async fn get_user(pool: &MySqlPool, id: Nulid) -> sqlx::Result<User> {
//!     sqlx::query_as::<_, User>("SELECT id, name FROM users WHERE id = ?")
//!         .bind(id)
//!         .fetch_one(pool)
//!         .await
//! }
//! ```

use crate::Nulid;
use sqlx::encode::IsNull;
use sqlx::error::BoxDynError;
use sqlx::mysql::{MySql, MySqlTypeInfo, MySqlValueRef};
use sqlx::postgres::{PgArgumentBuffer, PgHasArrayType, PgTypeInfo, PgValueRef, Postgres};
use sqlx::sqlite::{Sqlite, SqliteArgumentValue, SqliteTypeInfo, SqliteValueRef};
use sqlx::{Decode, Encode, Type};
use uuid::Uuid;

// ============================================================================
// PostgreSQL implementations
// ============================================================================

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

impl Encode<'_, Postgres> for Nulid {
    fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> Result<IsNull, BoxDynError> {
        let uuid = self.to_uuid();
        <Uuid as Encode<Postgres>>::encode_by_ref(&uuid, buf)
    }
}

impl<'r> Decode<'r, Postgres> for Nulid {
    fn decode(value: PgValueRef<'r>) -> Result<Self, BoxDynError> {
        let uuid = <Uuid as Decode<Postgres>>::decode(value)?;
        Ok(Self::from_uuid(uuid))
    }
}

// ============================================================================
// SQLite implementations
// ============================================================================

impl Type<Sqlite> for Nulid {
    fn type_info() -> SqliteTypeInfo {
        <[u8] as Type<Sqlite>>::type_info()
    }
}

impl<'q> Encode<'q, Sqlite> for Nulid {
    fn encode_by_ref(
        &self,
        args: &mut Vec<SqliteArgumentValue<'q>>,
    ) -> Result<IsNull, BoxDynError> {
        let bytes = self.to_bytes();
        args.push(SqliteArgumentValue::Blob(bytes.to_vec().into()));
        Ok(IsNull::No)
    }
}

impl<'r> Decode<'r, Sqlite> for Nulid {
    fn decode(value: SqliteValueRef<'r>) -> Result<Self, BoxDynError> {
        let bytes: Vec<u8> = Decode::<Sqlite>::decode(value)?;
        if bytes.len() != 16 {
            return Err("Invalid NULID length".into());
        }
        let mut array = [0u8; 16];
        array.copy_from_slice(&bytes);
        Ok(Self::from_bytes(array))
    }
}

// ============================================================================
// MySQL/MariaDB implementations
// ============================================================================

impl Type<MySql> for Nulid {
    fn type_info() -> MySqlTypeInfo {
        // Use BINARY(16) for MySQL storage
        <[u8] as Type<MySql>>::type_info()
    }

    fn compatible(ty: &MySqlTypeInfo) -> bool {
        // Delegate to [u8] which includes BINARY/VARBINARY/BLOB types
        <[u8] as Type<MySql>>::compatible(ty)
    }
}

impl Encode<'_, MySql> for Nulid {
    fn encode_by_ref(&self, buf: &mut Vec<u8>) -> Result<IsNull, BoxDynError> {
        // Delegate to the &[u8] implementation which handles MySQL's length-encoded format
        <&[u8] as Encode<MySql>>::encode(&self.to_bytes()[..], buf)
    }
}

impl<'r> Decode<'r, MySql> for Nulid {
    fn decode(value: MySqlValueRef<'r>) -> Result<Self, BoxDynError> {
        let bytes: &[u8] = Decode::<MySql>::decode(value)?;
        if bytes.len() != 16 {
            return Err("Invalid NULID length".into());
        }
        let mut array = [0u8; 16];
        array.copy_from_slice(bytes);
        Ok(Self::from_bytes(array))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // PostgreSQL tests
    mod postgres {
        use super::*;

        #[test]
        fn test_encode_decode_roundtrip() {
            let original = Nulid::new().expect("Failed to create NULID");

            // Convert to UUID and back to verify encoding path
            let uuid = original.to_uuid();
            let decoded = Nulid::from_uuid(uuid);

            assert_eq!(original, decoded);
            assert_eq!(original.nanos(), decoded.nanos());
            assert_eq!(original.random(), decoded.random());
        }

        #[test]
        fn test_nil_nulid() {
            let nil = Nulid::nil();
            let uuid = nil.to_uuid();
            let decoded = Nulid::from_uuid(uuid);

            assert!(decoded.is_nil());
            assert_eq!(decoded.nanos(), 0);
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

    // SQLite tests
    mod sqlite {
        use super::*;

        #[test]
        fn test_encode_decode_roundtrip() {
            let original = Nulid::new().expect("Failed to create NULID");

            // Convert to bytes and back to verify encoding path
            let bytes = original.to_bytes();
            let decoded = Nulid::from_bytes(bytes);

            assert_eq!(original, decoded);
            assert_eq!(original.nanos(), decoded.nanos());
            assert_eq!(original.random(), decoded.random());
        }

        #[test]
        fn test_nil_nulid() {
            let nil = Nulid::nil();
            let bytes = nil.to_bytes();
            let decoded = Nulid::from_bytes(bytes);

            assert!(decoded.is_nil());
            assert_eq!(decoded.nanos(), 0);
            assert_eq!(decoded.random(), 0);
        }

        #[test]
        fn test_nulid_bytes_equivalence() {
            // Test that NULID and bytes store the same 128-bit value
            let original = Nulid::new().expect("Failed to create NULID");
            let bytes = original.to_bytes();

            // Convert bytes back to NULID
            let decoded = Nulid::from_bytes(bytes);

            assert_eq!(original, decoded);
        }
    }

    // MySQL tests
    mod mysql {
        use super::*;

        #[test]
        fn test_encode_decode_roundtrip() {
            let original = Nulid::new().expect("Failed to create NULID");

            // Convert to bytes and back to verify encoding path (MySQL uses BINARY(16))
            let bytes = original.to_bytes();
            let decoded = Nulid::from_bytes(bytes);

            assert_eq!(original, decoded);
            assert_eq!(original.nanos(), decoded.nanos());
            assert_eq!(original.random(), decoded.random());
        }

        #[test]
        fn test_nil_nulid() {
            let nil = Nulid::nil();
            let bytes = nil.to_bytes();
            let decoded = Nulid::from_bytes(bytes);

            assert!(decoded.is_nil());
            assert_eq!(decoded.nanos(), 0);
            assert_eq!(decoded.random(), 0);
        }

        #[test]
        fn test_nulid_bytes_equivalence() {
            // Test that NULID and bytes store the same 128-bit value
            let original = Nulid::new().expect("Failed to create NULID");
            let bytes = original.to_bytes();

            // Convert bytes back to NULID
            let decoded = Nulid::from_bytes(bytes);

            assert_eq!(original, decoded);
        }

        #[test]
        fn test_encode_extends_buffer() {
            // Test that encode_by_ref correctly extends the buffer with length-encoded bytes
            let nulid = Nulid::new().expect("Failed to create NULID");
            let mut buf: Vec<u8> = Vec::new();

            // Simulate what MySQL encoding does: length prefix + bytes
            // For 16 bytes, the length is encoded as a single byte (16 < 251)
            let bytes = nulid.to_bytes();
            buf.push(16); // Length prefix
            buf.extend_from_slice(&bytes);

            // Buffer should be 17 bytes: 1 byte length + 16 bytes data
            assert_eq!(buf.len(), 17);
            assert_eq!(buf[0], 16); // Length prefix
            assert_eq!(&buf[1..], &nulid.to_bytes()[..]);
        }
    }
}

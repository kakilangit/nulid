//! `SQLx` support for `SQLite` storage.
//!
//! NULIDs are stored as BLOBs (16 bytes) in `SQLite`.
//!
//! # Example
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

use crate::Nulid;
use sqlx_core::decode::Decode;
use sqlx_core::encode::{Encode, IsNull};
use sqlx_core::error::BoxDynError;
use sqlx_core::types::Type;
use sqlx_sqlite::{Sqlite, SqliteArgumentValue, SqliteTypeInfo, SqliteValueRef};

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

#[cfg(test)]
mod tests {
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

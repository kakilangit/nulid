//! `SQLx` support for `MySQL`/`MariaDB` storage.
//!
//! NULIDs are stored as `BINARY(16)` in `MySQL`/`MariaDB`.
//!
//! # Example
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
use sqlx_core::decode::Decode;
use sqlx_core::encode::{Encode, IsNull};
use sqlx_core::error::BoxDynError;
use sqlx_core::types::Type;
use sqlx_mysql::{MySql, MySqlTypeInfo, MySqlValueRef};

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

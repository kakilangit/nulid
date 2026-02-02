//! `PostgreSQL` type support for NULID via postgres-types crate.
//!
//! This module provides implementations for encoding and decoding NULIDs
//! as `PostgreSQL` UUID types using the `postgres-types` crate.

use crate::Nulid;
use core::error::Error as StdError;
use postgres_types::{FromSql, IsNull, ToSql, Type, accepts, to_sql_checked};

impl<'a> FromSql<'a> for Nulid {
    fn from_sql(_ty: &Type, raw: &'a [u8]) -> Result<Self, Box<dyn StdError + Sync + Send>> {
        // PostgreSQL UUIDs are stored as 16 bytes in big-endian format
        if raw.len() != 16 {
            return Err("invalid UUID length".into());
        }

        let mut bytes = [0u8; 16];
        bytes.copy_from_slice(raw);
        Ok(Self::from_bytes(bytes))
    }

    accepts!(UUID);
}

impl ToSql for Nulid {
    fn to_sql(
        &self,
        _ty: &Type,
        out: &mut bytes::BytesMut,
    ) -> Result<IsNull, Box<dyn StdError + Sync + Send>> {
        // Convert NULID to bytes and write to buffer
        let bytes = self.to_bytes();
        out.extend_from_slice(&bytes);
        Ok(IsNull::No)
    }

    accepts!(UUID);
    to_sql_checked!();
}

#[cfg(test)]
mod tests {
    use super::*;
    use postgres_types::{FromSql, ToSql, Type};

    #[test]
    fn test_to_sql() {
        let nulid = Nulid::new().expect("Failed to create NULID");
        let ty = Type::UUID;
        let mut buf = bytes::BytesMut::new();

        let result = nulid.to_sql(&ty, &mut buf);
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), IsNull::No));
        assert_eq!(buf.len(), 16);
    }

    #[test]
    fn test_from_sql() {
        let nulid = Nulid::new().expect("Failed to create NULID");
        let ty = Type::UUID;
        let mut buf = bytes::BytesMut::new();

        nulid.to_sql(&ty, &mut buf).expect("Failed to serialize");
        let decoded = Nulid::from_sql(&ty, &buf).expect("Failed to deserialize");

        assert_eq!(nulid, decoded);
    }

    #[test]
    fn test_roundtrip() {
        let original = Nulid::new().expect("Failed to create NULID");
        let ty = Type::UUID;
        let mut buf = bytes::BytesMut::new();

        // Encode
        original.to_sql(&ty, &mut buf).expect("Failed to serialize");

        // Decode
        let decoded = Nulid::from_sql(&ty, &buf).expect("Failed to deserialize");

        // Verify
        assert_eq!(original, decoded);
        assert_eq!(original.nanos(), decoded.nanos());
        assert_eq!(original.random(), decoded.random());
    }

    #[test]
    fn test_nil_nulid() {
        let nil = Nulid::nil();
        let ty = Type::UUID;
        let mut buf = bytes::BytesMut::new();

        nil.to_sql(&ty, &mut buf).expect("Failed to serialize");
        let decoded = Nulid::from_sql(&ty, &buf).expect("Failed to deserialize");

        assert!(decoded.is_nil());
        assert_eq!(decoded.nanos(), 0);
        assert_eq!(decoded.random(), 0);
    }

    #[test]
    fn test_preserves_timestamp_and_random() {
        let nulid = Nulid::new().expect("Failed to create NULID");
        let original_nanos = nulid.nanos();
        let original_random = nulid.random();

        let ty = Type::UUID;
        let mut buf = bytes::BytesMut::new();

        nulid.to_sql(&ty, &mut buf).expect("Failed to serialize");
        let decoded = Nulid::from_sql(&ty, &buf).expect("Failed to deserialize");

        assert_eq!(decoded.nanos(), original_nanos);
        assert_eq!(decoded.random(), original_random);
    }

    #[test]
    fn test_specific_value() {
        let nulid = Nulid::from_u128(0x0123_4567_89AB_CDEF_FEDC_BA98_7654_3210);
        let ty = Type::UUID;
        let mut buf = bytes::BytesMut::new();

        nulid.to_sql(&ty, &mut buf).expect("Failed to serialize");
        let decoded = Nulid::from_sql(&ty, &buf).expect("Failed to deserialize");

        assert_eq!(decoded.as_u128(), 0x0123_4567_89AB_CDEF_FEDC_BA98_7654_3210);
    }
}

//! The `sqlx-postgres-bytea` mapping: NULIDs are stored as 16-byte `BYTEA`
//! values, in column order — no UUID byte shuffling.

use crate::Nulid;
use sqlx_core::decode::Decode;
use sqlx_core::encode::{Encode, IsNull};
use sqlx_core::error::BoxDynError;
use sqlx_core::types::Type;
use sqlx_postgres::{PgArgumentBuffer, PgHasArrayType, PgTypeInfo, PgValueRef, Postgres};

impl Type<Postgres> for Nulid {
    fn type_info() -> PgTypeInfo {
        <Vec<u8> as Type<Postgres>>::type_info()
    }

    fn compatible(ty: &PgTypeInfo) -> bool {
        <Vec<u8> as Type<Postgres>>::compatible(ty)
    }
}

impl PgHasArrayType for Nulid {
    fn array_type_info() -> PgTypeInfo {
        <Vec<u8> as PgHasArrayType>::array_type_info()
    }

    fn array_compatible(ty: &PgTypeInfo) -> bool {
        <Vec<u8> as PgHasArrayType>::array_compatible(ty)
    }
}

impl Encode<'_, Postgres> for Nulid {
    fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> Result<IsNull, BoxDynError> {
        Encode::<Postgres>::encode(self.to_bytes(), buf)
    }
}

impl<'r> Decode<'r, Postgres> for Nulid {
    fn decode(value: PgValueRef<'r>) -> Result<Self, BoxDynError> {
        let bytes = <Vec<u8> as Decode<Postgres>>::decode(value)?;
        Ok(Self::try_from(bytes.as_slice())?)
    }
}

#[cfg(all(test, feature = "sqlx-postgres-bytea"))]
mod bytea_mode_tests {
    use super::*;

    /// The bytea mapping encodes the NULID's own 16 bytes: what the buffer
    /// receives is exactly `to_bytes()`, in column order, with no UUID byte
    /// shuffling.
    #[test]
    fn encode_writes_the_raw_16_bytes_in_order() {
        let nulid = Nulid::from_bytes([
            0x01, 0x8e, 0x7e, 0x08, 0x55, 0xc7, 0xa2, 0x93, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x42,
        ]);
        let mut buffer = PgArgumentBuffer::default();
        let null = Encode::<Postgres>::encode_by_ref(&nulid, &mut buffer)
            .expect("bytea encoding cannot fail");
        assert!(matches!(null, IsNull::No));
        assert_eq!(buffer.as_slice(), nulid.to_bytes());
    }

    /// `to_bytes()` followed by `try_from` is the identity: the same bytes a
    /// BYTEA column stores come back as the same NULID.
    #[test]
    fn to_bytes_round_trips_through_try_from() {
        let original = Nulid::new().expect("generation works");
        let decoded = Nulid::from_bytes(original.to_bytes());
        assert_eq!(decoded, original);
        assert_eq!(decoded.to_bytes(), original.to_bytes());
    }

    /// The type the impl announces is BYTEA, so sqlx picks the right column
    /// type for `CREATE TABLE AS` style flows and validates responses.
    #[test]
    fn type_info_is_bytea() {
        let expected = <Vec<u8> as Type<Postgres>>::type_info();
        assert_eq!(<Nulid as Type<Postgres>>::type_info(), expected);
        assert!(<Nulid as Type<Postgres>>::compatible(&expected));
    }
}

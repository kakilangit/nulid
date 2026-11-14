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
        let nulid = Nulid::from_bytes(&[
            0x01, 0x8e, 0x7e, 0x08, 0x55, 0xc7, 0xa2, 0x93, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x42,
        ]);
        let mut buffer = PgArgumentBuffer::default();
        let null = Encode::<Postgres>::encode_by_ref(&nulid, &mut buffer)
            .expect("bytea encoding cannot fail");
        assert_eq!(null, IsNull::No);
        assert_eq!(buffer.as_slice(), nulid.to_bytes());
    }

    /// Decode accepts exactly the bytes `encode` wrote, so a row written by
    /// this impl reads back identical, bit for bit.
    #[test]
    fn decode_reads_the_raw_16_bytes_in_order() {
        let bytes: Vec<u8> = (0..=15).collect();
        let mut buffer = PgArgumentBuffer::default();
        buffer.extend_from_slice(&bytes);
        let decoded = Nulid::decode(PgValueRef::borrowed(
            &buffer,
            <Vec<u8> as Type<Postgres>>::type_info(),
        ))
        .expect("16 bytes decode");
        assert_eq!(decoded.to_bytes(), bytes.as_slice());
    }

    /// Short and long payloads are rejected rather than silently truncated.
    #[test]
    fn decode_refuses_payloads_that_are_not_16_bytes() {
        let mut buffer = PgArgumentBuffer::default();
        buffer.extend_from_slice(&[0u8; 15]);
        assert!(
            Nulid::decode(PgValueRef::borrowed(
                &buffer,
                <Vec<u8> as Type<Postgres>>::type_info(),
            ))
            .is_err()
        );
        let mut buffer = PgArgumentBuffer::default();
        buffer.extend_from_slice(&[0u8; 17]);
        assert!(
            Nulid::decode(PgValueRef::borrowed(
                &buffer,
                <Vec<u8> as Type<Postgres>>::type_info(),
            ))
            .is_err()
        );
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

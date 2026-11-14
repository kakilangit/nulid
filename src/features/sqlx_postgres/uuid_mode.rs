use crate::Nulid;
use sqlx_core::decode::Decode;
use sqlx_core::encode::{Encode, IsNull};
use sqlx_core::error::BoxDynError;
use sqlx_core::types::Type;
use sqlx_postgres::{PgArgumentBuffer, PgHasArrayType, PgTypeInfo, PgValueRef, Postgres};
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

#[cfg(test)]
mod uuid_mode_tests {
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
        let decoded = Nulid::from_bytes(*uuid_bytes);

        assert_eq!(original, decoded);
        assert_eq!(*uuid_bytes, decoded.to_bytes());
    }
}

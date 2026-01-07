//! Protobuf integration for NULID.
//!
//! This module provides protobuf serialization support for NULID.
//! Since protobuf doesn't have a native u128 type, the 128-bit value
//! is split into two 64-bit fields (high and low bits).
//!
//! # Examples
//!
//! ```ignore
//! use nulid::Nulid;
//! use nulid::proto::nulid::Nulid;
//!
//! // Generate a NULID
//! let nulid = Nulid::new()?;
//!
//! // Convert to protobuf
//! let proto: nulid::proto::nulid::Nulid = nulid.into();
//!
//! // Convert back to NULID
//! let nulid2: Nulid = proto.into();
//!
//! assert_eq!(nulid, nulid2);
//! ```

use crate::Nulid;

impl Nulid {
    /// Converts this NULID to its protobuf representation.
    ///
    /// The 128-bit value is split into high and low 64-bit parts.
    ///
    /// # Examples
    ///
    /// ```
    /// use nulid::Nulid;
    /// use nulid::ProtoNulid;
    ///
    /// # fn main() -> nulid::Result<()> {
    /// let nulid = Nulid::new()?;
    /// let proto = nulid.to_proto();
    /// let nulid2 = Nulid::from_proto(proto);
    /// assert_eq!(nulid, nulid2);
    /// # Ok(())
    /// # }
    /// ```
    #[allow(clippy::cast_possible_truncation)]
    #[must_use]
    pub const fn to_proto(self) -> crate::proto::nulid::Nulid {
        let value = self.as_u128();
        crate::proto::nulid::Nulid {
            high: (value >> 64) as u64,
            low: value as u64,
        }
    }

    /// Creates a NULID from its protobuf representation.
    ///
    /// # Examples
    ///
    /// ```
    /// use nulid::Nulid;
    /// use nulid::ProtoNulid;
    ///
    /// let proto = ProtoNulid { high: 0x0123456789ABCDEF, low: 0xFEDCBA9876543210 };
    /// let nulid = Nulid::from_proto(proto);
    /// assert_eq!(nulid.as_u128(), 0x0123456789ABCDEF_FEDCBA9876543210);
    /// ```
    #[must_use]
    pub const fn from_proto(proto: crate::proto::nulid::Nulid) -> Self {
        Self::from_u128((proto.high as u128) << 64 | proto.low as u128)
    }
}

impl From<Nulid> for crate::proto::nulid::Nulid {
    /// Converts a NULID to its protobuf representation.
    ///
    /// # Examples
    ///
    /// ```
    /// use nulid::Nulid;
    ///
    /// # fn main() -> nulid::Result<()> {
    /// let nulid = Nulid::new()?;
    /// let proto: nulid::ProtoNulid = nulid.into();
    /// assert_eq!(proto.low as u128, nulid.as_u128() & u64::MAX as u128);
    /// # Ok(())
    /// # }
    /// ```
    fn from(nulid: Nulid) -> Self {
        nulid.to_proto()
    }
}

impl From<crate::proto::nulid::Nulid> for Nulid {
    /// Converts a protobuf representation back to a NULID.
    ///
    /// # Examples
    ///
    /// ```
    /// use nulid::Nulid;
    /// use nulid::ProtoNulid;
    ///
    /// let proto = ProtoNulid { high: 0x0123456789ABCDEF, low: 0xFEDCBA9876543210 };
    /// let nulid: Nulid = proto.into();
    /// assert_eq!(nulid.as_u128(), 0x0123456789ABCDEF_FEDCBA9876543210);
    /// ```
    fn from(proto: crate::proto::nulid::Nulid) -> Self {
        Self::from_proto(proto)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_proto_conversion() {
        let nulid = Nulid::new().expect("Failed to create NULID");
        let proto = nulid.to_proto();
        let nulid2 = Nulid::from_proto(proto);
        assert_eq!(nulid, nulid2);
    }

    #[test]
    fn test_proto_from_trait() {
        let nulid = Nulid::new().expect("Failed to create NULID");
        let proto: crate::proto::nulid::Nulid = nulid.into();
        let nulid2: Nulid = proto.into();
        assert_eq!(nulid, nulid2);
    }

    #[test]
    fn test_proto_round_trip() {
        let nulid = Nulid::new().expect("Failed to create NULID");
        let proto = crate::proto::nulid::Nulid::from(nulid);
        let nulid2 = Nulid::from(proto);
        assert_eq!(nulid, nulid2);
        assert_eq!(nulid.nanos(), nulid2.nanos());
        assert_eq!(nulid.random(), nulid2.random());
    }

    #[test]
    fn test_proto_preserves_bits() {
        let test_value = 0x0123_4567_89AB_CDEF_FEDC_BA98_7654_3210_u128;
        let nulid = Nulid::from_u128(test_value);
        let proto = nulid.to_proto();
        assert_eq!(proto.high, 0x0123_4567_89AB_CDEF);
        assert_eq!(proto.low, 0xFEDC_BA98_7654_3210);
    }

    #[test]
    fn test_proto_reconstructs_bits() {
        let proto = crate::proto::nulid::Nulid {
            high: 0x0123_4567_89AB_CDEF,
            low: 0xFEDC_BA98_7654_3210,
        };
        let nulid = Nulid::from_proto(proto);
        assert_eq!(nulid.as_u128(), 0x0123_4567_89AB_CDEF_FEDC_BA98_7654_3210);
    }

    #[test]
    fn test_proto_nil() {
        let nil = Nulid::nil();
        let proto = nil.to_proto();
        assert_eq!(proto.high, 0);
        assert_eq!(proto.low, 0);
    }

    #[test]
    fn test_proto_max() {
        let max = Nulid::max();
        let proto = max.to_proto();
        assert_eq!(proto.high, u64::MAX);
        assert_eq!(proto.low, u64::MAX);
    }

    #[test]
    fn test_proto_encoding() {
        use prost::Message;
        let nulid = Nulid::new().expect("Failed to create NULID");
        let proto = nulid.to_proto();
        let encoded = proto.encode_to_vec();
        let decoded = crate::proto::nulid::Nulid::decode(&*encoded).expect("Failed to decode");
        let nulid2 = Nulid::from_proto(decoded);
        assert_eq!(nulid, nulid2);
    }
}

//! Serde serialization support for NULID.
//!
//! This module provides `Serialize` and `Deserialize` implementations for NULID,
//! supporting both human-readable (string) and binary (bytes) formats.
//!
//! # Examples
//!
//! ```
//! use nulid::Nulid;
//! use serde_json;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let nulid = Nulid::new()?;
//!
//! // Serialize to JSON (human-readable)
//! let json = serde_json::to_string(&nulid)?;
//! println!("JSON: {}", json);
//!
//! // Deserialize from JSON
//! let nulid2: Nulid = serde_json::from_str(&json)?;
//! assert_eq!(nulid, nulid2);
//! # Ok(())
//! # }
//! ```

use crate::Nulid;
use core::str::FromStr;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

impl Serialize for Nulid {
    /// Serializes the NULID.
    ///
    /// - For human-readable formats (JSON, TOML, etc.): serializes as a string
    /// - For binary formats (`MessagePack`, Bincode, etc.): serializes as a fixed-size byte array
    fn serialize<S>(&self, serializer: S) -> core::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if serializer.is_human_readable() {
            serializer.serialize_str(&self.to_string())
        } else {
            // Serialize as a fixed-size array for efficient binary formats like bincode
            use serde::ser::SerializeTuple;
            let bytes = self.to_bytes();
            let mut tuple = serializer.serialize_tuple(16)?;
            for byte in &bytes {
                tuple.serialize_element(byte)?;
            }
            tuple.end()
        }
    }
}

impl<'de> Deserialize<'de> for Nulid {
    /// Deserializes a NULID.
    ///
    /// - For human-readable formats (JSON, TOML, etc.): expects a string
    /// - For binary formats (`MessagePack`, Bincode, etc.): expects a fixed-size byte array
    fn deserialize<D>(deserializer: D) -> core::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            let s = <&str>::deserialize(deserializer)?;
            Self::from_str(s).map_err(serde::de::Error::custom)
        } else {
            // Deserialize as a fixed-size array for efficient binary formats like bincode
            let bytes = <[u8; 16]>::deserialize(deserializer)?;
            Ok(Self::from_bytes(bytes))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serde_json_round_trip() {
        let nulid = Nulid::new().expect("Failed to create NULID");
        let json = serde_json::to_string(&nulid).expect("Failed to serialize");
        let nulid2: Nulid = serde_json::from_str(&json).expect("Failed to deserialize");
        assert_eq!(nulid, nulid2);
    }

    #[test]
    fn test_serde_json_format() {
        let nulid = Nulid::from_u128(0x0123_4567_89AB_CDEF_FEDC_BA98_7654_3210);
        let json = serde_json::to_string(&nulid).expect("Failed to serialize");
        // Should be a quoted string
        assert!(json.starts_with('"'));
        assert!(json.ends_with('"'));
    }

    #[test]
    fn test_serde_binary_round_trip() {
        let nulid = Nulid::new().expect("Failed to create NULID");
        let bytes = rmp_serde::to_vec(&nulid).expect("Failed to serialize");
        let nulid2: Nulid = rmp_serde::from_slice(&bytes).expect("Failed to deserialize");
        assert_eq!(nulid, nulid2);
    }

    #[test]
    fn test_serde_nil() {
        let nil = Nulid::nil();
        let json = serde_json::to_string(&nil).expect("Failed to serialize");
        let nil2: Nulid = serde_json::from_str(&json).expect("Failed to deserialize");
        assert!(nil2.is_nil());
    }

    #[test]
    fn test_serde_preserves_timestamp_and_random() {
        let nulid = Nulid::new().expect("Failed to create NULID");
        let json = serde_json::to_string(&nulid).expect("Failed to serialize");
        let nulid2: Nulid = serde_json::from_str(&json).expect("Failed to deserialize");

        assert_eq!(nulid.timestamp_nanos(), nulid2.timestamp_nanos());
        assert_eq!(nulid.random(), nulid2.random());
    }

    #[test]
    fn test_bincode_round_trip() {
        let nulid = Nulid::new().expect("Failed to create NULID");
        let encoded = bincode::serde::encode_to_vec(nulid, bincode::config::standard())
            .expect("Failed to serialize");
        let (decoded, _): (Nulid, usize) =
            bincode::serde::decode_from_slice(&encoded, bincode::config::standard())
                .expect("Failed to deserialize");
        assert_eq!(nulid, decoded);
    }

    #[test]
    fn test_bincode_size() {
        let nulid = Nulid::new().expect("Failed to create NULID");
        let encoded = bincode::serde::encode_to_vec(nulid, bincode::config::standard())
            .expect("Failed to serialize");
        // Bincode should encode NULID as 16 bytes (fixed-size array)
        assert_eq!(encoded.len(), 16);
    }

    #[test]
    fn test_bincode_preserves_fields() {
        let nulid = Nulid::new().expect("Failed to create NULID");
        let encoded = bincode::serde::encode_to_vec(nulid, bincode::config::standard())
            .expect("Failed to serialize");
        let (decoded, _): (Nulid, usize) =
            bincode::serde::decode_from_slice(&encoded, bincode::config::standard())
                .expect("Failed to deserialize");

        assert_eq!(nulid.timestamp_nanos(), decoded.timestamp_nanos());
        assert_eq!(nulid.random(), decoded.random());
    }

    #[test]
    fn test_bincode_deterministic() {
        let nulid = Nulid::from_u128(0x0123_4567_89AB_CDEF_FEDC_BA98_7654_3210);

        let encoded1 = bincode::serde::encode_to_vec(nulid, bincode::config::standard())
            .expect("Failed to serialize");
        let encoded2 = bincode::serde::encode_to_vec(nulid, bincode::config::standard())
            .expect("Failed to serialize");

        // Same NULID should always produce same encoding
        assert_eq!(encoded1, encoded2);
    }

    #[test]
    fn test_bincode_vec() {
        let nulids = vec![
            Nulid::new().expect("Failed to create NULID"),
            Nulid::new().expect("Failed to create NULID"),
            Nulid::new().expect("Failed to create NULID"),
        ];

        let encoded = bincode::serde::encode_to_vec(&nulids, bincode::config::standard())
            .expect("Failed to serialize");
        let (decoded, _): (Vec<Nulid>, usize) =
            bincode::serde::decode_from_slice(&encoded, bincode::config::standard())
                .expect("Failed to deserialize");

        assert_eq!(nulids, decoded);
    }
}

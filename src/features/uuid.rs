//! UUID interoperability for NULID.
//!
//! This module provides seamless conversion between NULID and UUID types,
//! enabling NULID to be used in systems that expect UUIDs.
//!
//! # Examples
//!
//! ```
//! use nulid::Nulid;
//! use uuid::Uuid;
//!
//! # fn main() -> nulid::Result<()> {
//! // Generate a NULID
//! let nulid = Nulid::new()?;
//!
//! // Convert to UUID
//! let uuid: Uuid = nulid.into();
//!
//! // Convert back to NULID
//! let nulid2: Nulid = uuid.into();
//!
//! assert_eq!(nulid, nulid2);
//! # Ok(())
//! # }
//! ```

use crate::Nulid;

impl Nulid {
    /// Converts this NULID to a UUID.
    ///
    /// The 128-bit value is preserved exactly, maintaining full compatibility
    /// with UUID-based systems.
    ///
    /// # Examples
    ///
    /// ```
    /// use nulid::Nulid;
    ///
    /// # fn main() -> nulid::Result<()> {
    /// let nulid = Nulid::new()?;
    /// let uuid = nulid.to_uuid();
    /// assert_eq!(uuid.as_u128(), nulid.as_u128());
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub const fn to_uuid(self) -> uuid::Uuid {
        uuid::Uuid::from_u128(self.as_u128())
    }

    /// Creates a NULID from a UUID.
    ///
    /// The 128-bit value is preserved exactly. Note that the UUID may not
    /// represent a valid NULID structure (timestamp + random), but the
    /// conversion is lossless.
    ///
    /// # Examples
    ///
    /// ```
    /// use nulid::Nulid;
    /// use uuid::Uuid;
    ///
    /// let uuid = Uuid::new_v4();
    /// let nulid = Nulid::from_uuid(uuid);
    /// assert_eq!(nulid.to_uuid(), uuid);
    /// ```
    #[must_use]
    pub const fn from_uuid(uuid: uuid::Uuid) -> Self {
        Self::from_u128(uuid.as_u128())
    }
}

impl From<uuid::Uuid> for Nulid {
    /// Converts a UUID to a NULID.
    ///
    /// # Examples
    ///
    /// ```
    /// use nulid::Nulid;
    /// use uuid::Uuid;
    ///
    /// let uuid = Uuid::new_v4();
    /// let nulid: Nulid = uuid.into();
    /// assert_eq!(nulid.as_u128(), uuid.as_u128());
    /// ```
    fn from(uuid: uuid::Uuid) -> Self {
        Self::from_uuid(uuid)
    }
}

impl From<Nulid> for uuid::Uuid {
    /// Converts a NULID to a UUID.
    ///
    /// # Examples
    ///
    /// ```
    /// use nulid::Nulid;
    /// use uuid::Uuid;
    ///
    /// # fn main() -> nulid::Result<()> {
    /// let nulid = Nulid::new()?;
    /// let uuid: Uuid = nulid.into();
    /// assert_eq!(uuid.as_u128(), nulid.as_u128());
    /// # Ok(())
    /// # }
    /// ```
    fn from(nulid: Nulid) -> Self {
        nulid.to_uuid()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uuid_conversion() {
        let nulid = Nulid::new().expect("Failed to create NULID");
        let uuid = nulid.to_uuid();
        assert_eq!(uuid.as_u128(), nulid.as_u128());
    }

    #[test]
    fn test_uuid_from_trait() {
        let nulid = Nulid::new().expect("Failed to create NULID");
        let uuid: uuid::Uuid = nulid.into();
        assert_eq!(uuid.as_u128(), nulid.as_u128());
    }

    #[test]
    fn test_uuid_into_trait() {
        let uuid = uuid::Uuid::new_v4();
        let nulid: Nulid = uuid.into();
        assert_eq!(nulid.as_u128(), uuid.as_u128());
    }

    #[test]
    fn test_uuid_round_trip() {
        let nulid = Nulid::new().expect("Failed to create NULID");
        let uuid = nulid.to_uuid();
        let nulid2 = Nulid::from_uuid(uuid);
        assert_eq!(nulid, nulid2);
        assert_eq!(nulid.timestamp_nanos(), nulid2.timestamp_nanos());
        assert_eq!(nulid.random(), nulid2.random());
    }

    #[test]
    fn test_nil_uuid() {
        let nil = Nulid::nil();
        let uuid = nil.to_uuid();
        assert_eq!(uuid, uuid::Uuid::nil());
    }

    #[test]
    fn test_uuid_preserves_bits() {
        // Test that all 128 bits are preserved
        let test_value = 0x0123_4567_89AB_CDEF_FEDC_BA98_7654_3210_u128;
        let nulid = Nulid::from_u128(test_value);
        let uuid = nulid.to_uuid();
        assert_eq!(uuid.as_u128(), test_value);
    }
}

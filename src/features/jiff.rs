//! Jiff integration for NULID.
//!
//! This module provides conversion between NULID and `jiff::Timestamp`.

use jiff::Timestamp;
use rand::Rng;

use crate::{Nulid, Result};

impl Nulid {
    /// Converts this NULID to a `jiff::Timestamp`.
    ///
    /// # Examples
    ///
    /// ```
    /// use nulid::Nulid;
    ///
    /// # fn main() -> nulid::Result<()> {
    /// let id = Nulid::new()?;
    /// let ts = id.jiff_timestamp()?;
    /// println!("NULID timestamp: {}", ts);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if the timestamp is out of range for jiff
    /// (which would require a timestamp beyond year 9999 or before year 1).
    #[allow(clippy::cast_possible_truncation)]
    pub fn jiff_timestamp(self) -> Result<Timestamp> {
        let nanos = self.nanos();
        let secs = (nanos / 1_000_000_000) as i64;
        let subsec_nanos = (nanos % 1_000_000_000) as i32;
        Timestamp::new(secs, subsec_nanos).map_err(|_| crate::Error::RandomError)
    }

    /// Creates a NULID from a `jiff::Timestamp` with random bits.
    ///
    /// # Examples
    ///
    /// ```
    /// use nulid::Nulid;
    /// use jiff::Timestamp;
    ///
    /// # fn main() -> nulid::Result<()> {
    /// let ts = Timestamp::from_second(1_704_067_200).unwrap();
    /// let id = Nulid::from_jiff_timestamp(ts)?;
    /// println!("NULID from Timestamp: {}", id);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if random number generation fails.
    #[allow(clippy::cast_sign_loss)]
    pub fn from_jiff_timestamp(ts: Timestamp) -> Result<Self> {
        let nanos = ts.as_nanosecond();
        let timestamp_nanos = nanos as u128;

        let mut rng = rand::rng();
        let random = rng.random::<u64>() & ((1u64 << Self::RANDOM_BITS) - 1);

        Ok(Self::from_nanos(timestamp_nanos, random))
    }
}

impl TryFrom<Timestamp> for Nulid {
    type Error = crate::Error;

    /// Creates a NULID from a `jiff::Timestamp`.
    ///
    /// # Examples
    ///
    /// ```
    /// use nulid::Nulid;
    /// use jiff::Timestamp;
    ///
    /// let ts = Timestamp::from_second(1_704_067_200).unwrap();
    /// let nulid: Nulid = ts.try_into()?;
    /// # Ok::<_, nulid::Error>(())
    /// ```
    fn try_from(ts: Timestamp) -> core::result::Result<Self, Self::Error> {
        Self::from_jiff_timestamp(ts)
    }
}

impl TryFrom<Nulid> for Timestamp {
    type Error = crate::Error;

    /// Converts a NULID to a `jiff::Timestamp`.
    ///
    /// # Examples
    ///
    /// ```
    /// use nulid::Nulid;
    ///
    /// # fn main() -> nulid::Result<()> {
    /// let nulid = Nulid::new()?;
    /// let ts: jiff::Timestamp = nulid.try_into()?;
    /// # Ok(())
    /// # }
    /// ```
    fn try_from(nulid: Nulid) -> core::result::Result<Self, Self::Error> {
        nulid.jiff_timestamp()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jiff_timestamp() {
        let timestamp_nanos = 1_704_067_200_000_000_000u128;
        let nulid = Nulid::from_nanos(timestamp_nanos, 12345);
        let ts = nulid.jiff_timestamp().unwrap();

        let expected = Timestamp::from_second(1_704_067_200).unwrap();
        assert_eq!(ts, expected);
    }

    #[test]
    fn test_jiff_timestamp_with_subsec_nanos() {
        let timestamp_nanos = 1_704_067_200_123_456_789u128;
        let nulid = Nulid::from_nanos(timestamp_nanos, 0);
        let ts = nulid.jiff_timestamp().unwrap();

        assert_eq!(ts.as_second(), 1_704_067_200);
        assert_eq!(ts.subsec_nanosecond(), 123_456_789);
    }

    #[test]
    fn test_jiff_timestamp_epoch() {
        let nulid = Nulid::from_nanos(0, 0);
        let ts = nulid.jiff_timestamp().unwrap();
        assert_eq!(ts.as_second(), 0);
    }

    #[test]
    fn test_from_jiff_timestamp() {
        let ts = Timestamp::from_second(1_704_067_200).unwrap();
        let nulid = Nulid::from_jiff_timestamp(ts).expect("Failed to create NULID");

        let timestamp_nanos = 1_704_067_200_000_000_000u128;
        assert_eq!(nulid.nanos(), timestamp_nanos);
    }

    #[test]
    fn test_from_jiff_timestamp_with_subsec() {
        let ts = Timestamp::from_second(1_704_067_200)
            .unwrap()
            .checked_add(jiff::Span::new().nanoseconds(123_456_789))
            .unwrap();
        let nulid = Nulid::from_jiff_timestamp(ts).expect("Failed to create NULID");

        let timestamp_nanos = 1_704_067_200_123_456_789u128;
        assert_eq!(nulid.nanos(), timestamp_nanos);
    }

    #[test]
    fn test_jiff_timestamp_roundtrip() {
        let original = Nulid::new().expect("Failed to create NULID");
        let ts = original.jiff_timestamp().unwrap();
        let roundtrip = Nulid::from_jiff_timestamp(ts).expect("Failed to create NULID");

        assert_eq!(original.nanos(), roundtrip.nanos());
    }

    #[test]
    #[allow(clippy::cast_possible_wrap)]
    fn test_jiff_from_trait() {
        let nulid = Nulid::new().expect("Failed to create NULID");
        let ts: Timestamp = nulid.try_into().unwrap();
        assert_eq!(ts.as_second(), nulid.seconds() as i64);
    }

    #[test]
    fn test_jiff_into_trait() {
        let ts = Timestamp::from_second(1_704_067_200).unwrap();
        let nulid: Nulid = ts.try_into().unwrap();
        assert_eq!(nulid.nanos(), 1_704_067_200_000_000_000u128);
    }
}

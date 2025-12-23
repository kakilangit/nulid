//! Chrono integration for NULID.
//!
//! This module provides conversion between NULID and `chrono::DateTime<Utc>`.

use crate::{Nulid, Result};
use chrono::{DateTime, Utc};
use rand::Rng;

impl Nulid {
    /// Converts this NULID to a `chrono::DateTime<Utc>`.
    ///
    /// # Examples
    ///
    /// ```
    /// use nulid::Nulid;
    /// use chrono::{DateTime, Utc};
    ///
    /// # fn main() -> nulid::Result<()> {
    /// let id = Nulid::new()?;
    /// let dt: DateTime<Utc> = id.chrono_datetime();
    /// println!("NULID timestamp: {}", dt);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Panics
    ///
    /// This function will never panic in practice. If the timestamp is out of range
    /// for chrono (which would require a timestamp beyond year 262,000), it falls back
    /// to the Unix epoch. The fallback itself cannot panic.
    #[must_use]
    #[allow(clippy::cast_possible_truncation)]
    pub fn chrono_datetime(self) -> DateTime<Utc> {
        let nanos = self.nanos();
        let secs = (nanos / 1_000_000_000) as i64;
        let subsec_nanos = (nanos % 1_000_000_000) as u32;
        DateTime::from_timestamp(secs, subsec_nanos).unwrap_or_else(|| {
            // This fallback should never be reached in practice, but is here for safety.
            // SAFETY: Unix epoch (0, 0) is always a valid timestamp.
            #[allow(clippy::expect_used)]
            DateTime::<Utc>::from_timestamp(0, 0).expect("epoch should be valid")
        })
    }

    /// Creates a NULID from a `chrono::DateTime<Utc>` with random bits.
    ///
    /// # Examples
    ///
    /// ```
    /// use nulid::Nulid;
    /// use chrono::{DateTime, Utc, TimeZone};
    ///
    /// # fn main() -> nulid::Result<()> {
    /// let dt = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
    /// let id = Nulid::from_chrono_datetime(dt)?;
    /// println!("NULID from DateTime: {}", id);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if random number generation fails.
    #[allow(clippy::cast_sign_loss)]
    pub fn from_chrono_datetime(dt: DateTime<Utc>) -> Result<Self> {
        let timestamp_nanos =
            dt.timestamp() as u128 * 1_000_000_000 + u128::from(dt.timestamp_subsec_nanos());

        let mut rng = rand::rng();
        let random = rng.random::<u64>() & ((1u64 << Self::RANDOM_BITS) - 1);

        Ok(Self::from_nanos(timestamp_nanos, random))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Timelike};

    #[test]
    fn test_chrono_datetime() {
        // Test with a known timestamp: 2024-01-01 00:00:00 UTC
        let timestamp_nanos = 1_704_067_200_000_000_000u128; // 2024-01-01 00:00:00 UTC
        let nulid = Nulid::from_nanos(timestamp_nanos, 12345);
        let dt = nulid.chrono_datetime();

        let expected = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
        assert_eq!(dt, expected);
    }

    #[test]
    fn test_chrono_datetime_with_subsec_nanos() {
        // Test with nanosecond precision
        let timestamp_nanos = 1_704_067_200_123_456_789u128;
        let nulid = Nulid::from_nanos(timestamp_nanos, 0);
        let dt = nulid.chrono_datetime();

        assert_eq!(dt.timestamp(), 1_704_067_200);
        assert_eq!(dt.timestamp_subsec_nanos(), 123_456_789);
    }

    #[test]
    fn test_chrono_datetime_epoch() {
        let nulid = Nulid::from_nanos(0, 0);
        let dt = nulid.chrono_datetime();
        let epoch = Utc.timestamp_opt(0, 0).unwrap();
        assert_eq!(dt, epoch);
    }

    #[test]
    fn test_chrono_datetime_current() {
        let nulid = Nulid::new().expect("Failed to create NULID");
        let dt = nulid.chrono_datetime();

        // Should be reasonably close to now
        let now = Utc::now();
        let diff = (now.timestamp() - dt.timestamp()).abs();
        assert!(diff < 2, "DateTime should be close to current time");
    }

    #[test]
    fn test_from_chrono_datetime() {
        // Test with a known DateTime
        let dt = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
        let nulid = Nulid::from_chrono_datetime(dt).expect("Failed to create NULID");

        let timestamp_nanos = 1_704_067_200_000_000_000u128;
        assert_eq!(nulid.nanos(), timestamp_nanos);
    }

    #[test]
    fn test_from_chrono_datetime_with_subsec() {
        // Test with nanosecond precision
        let dt = Utc
            .with_ymd_and_hms(2024, 1, 1, 0, 0, 0)
            .unwrap()
            .with_nanosecond(123_456_789)
            .unwrap();
        let nulid = Nulid::from_chrono_datetime(dt).expect("Failed to create NULID");

        let timestamp_nanos = 1_704_067_200_123_456_789u128;
        assert_eq!(nulid.nanos(), timestamp_nanos);
    }

    #[test]
    fn test_chrono_datetime_roundtrip() {
        // Test that converting to DateTime and back preserves the timestamp
        let original = Nulid::new().expect("Failed to create NULID");
        let dt = original.chrono_datetime();
        let roundtrip = Nulid::from_chrono_datetime(dt).expect("Failed to create NULID");

        // Timestamps should match (random parts will differ)
        assert_eq!(original.nanos(), roundtrip.nanos());
    }
}

//! Chrono integration for NULID.
//!
//! This module provides conversion between NULID and `chrono::DateTime<Utc>`.

use chrono::{DateTime, Utc};
use rand::Rng;

use crate::{Nulid, Result};

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
    /// let dt: DateTime<Utc> = id.chrono_datetime()?;
    /// println!("NULID timestamp: {}", dt);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if the timestamp is out of range for chrono
    /// (which would require a timestamp beyond year 262,000).
    #[allow(clippy::cast_possible_truncation)]
    pub fn chrono_datetime(self) -> Result<DateTime<Utc>> {
        let nanos = self.nanos();
        let secs = (nanos / 1_000_000_000) as i64;
        let subsec_nanos = (nanos % 1_000_000_000) as u32;
        DateTime::from_timestamp(secs, subsec_nanos).ok_or(crate::Error::RandomError)
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

impl TryFrom<DateTime<Utc>> for Nulid {
    type Error = crate::Error;

    /// Creates a NULID from a `chrono::DateTime<Utc>`.
    ///
    /// # Examples
    ///
    /// ```
    /// use nulid::Nulid;
    /// use chrono::{DateTime, Utc, TimeZone};
    ///
    /// let dt = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
    /// let nulid: Nulid = dt.try_into()?;
    /// # Ok::<_, nulid::Error>(())
    /// ```
    fn try_from(dt: DateTime<Utc>) -> core::result::Result<Self, Self::Error> {
        Self::from_chrono_datetime(dt)
    }
}

impl TryFrom<Nulid> for DateTime<Utc> {
    type Error = crate::Error;

    /// Converts a NULID to a `chrono::DateTime<Utc>`.
    ///
    /// # Examples
    ///
    /// ```
    /// use nulid::Nulid;
    /// use chrono::{DateTime, Utc};
    ///
    /// # fn main() -> nulid::Result<()> {
    /// let nulid = Nulid::new()?;
    /// let dt: DateTime<Utc> = nulid.try_into()?;
    /// # Ok(())
    /// # }
    /// ```
    fn try_from(nulid: Nulid) -> core::result::Result<Self, Self::Error> {
        nulid.chrono_datetime()
    }
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Timelike};

    use super::*;

    #[test]
    fn test_chrono_datetime() {
        let timestamp_nanos = 1_704_067_200_000_000_000u128;
        let nulid = Nulid::from_nanos(timestamp_nanos, 12345);
        let dt = nulid.chrono_datetime().unwrap();

        let expected = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
        assert_eq!(dt, expected);
    }

    #[test]
    fn test_chrono_datetime_with_subsec_nanos() {
        let timestamp_nanos = 1_704_067_200_123_456_789u128;
        let nulid = Nulid::from_nanos(timestamp_nanos, 0);
        let dt = nulid.chrono_datetime().unwrap();

        assert_eq!(dt.timestamp(), 1_704_067_200);
        assert_eq!(dt.timestamp_subsec_nanos(), 123_456_789);
    }

    #[test]
    fn test_chrono_datetime_epoch() {
        let nulid = Nulid::from_nanos(0, 0);
        let dt = nulid.chrono_datetime().unwrap();
        let epoch = Utc.timestamp_opt(0, 0).unwrap();
        assert_eq!(dt, epoch);
    }

    #[test]
    fn test_chrono_datetime_current() {
        let nulid = Nulid::new().expect("Failed to create NULID");
        let dt = nulid.chrono_datetime().unwrap();

        let now = Utc::now();
        let diff = (now.timestamp() - dt.timestamp()).abs();
        assert!(diff < 2, "DateTime should be close to current time");
    }

    #[test]
    fn test_from_chrono_datetime() {
        let dt = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
        let nulid = Nulid::from_chrono_datetime(dt).expect("Failed to create NULID");

        let timestamp_nanos = 1_704_067_200_000_000_000u128;
        assert_eq!(nulid.nanos(), timestamp_nanos);
    }

    #[test]
    fn test_from_chrono_datetime_with_subsec() {
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
        let original = Nulid::new().expect("Failed to create NULID");
        let dt = original.chrono_datetime().unwrap();
        let roundtrip = Nulid::from_chrono_datetime(dt).expect("Failed to create NULID");

        assert_eq!(original.nanos(), roundtrip.nanos());
    }

    #[test]
    #[allow(clippy::cast_possible_wrap)]
    fn test_chrono_from_trait() {
        let nulid = Nulid::new().expect("Failed to create NULID");
        let dt: DateTime<Utc> = nulid.try_into().unwrap();
        assert_eq!(dt.timestamp(), nulid.seconds() as i64);
    }

    #[test]
    fn test_chrono_into_trait() {
        let dt = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
        let nulid: Nulid = dt.try_into().unwrap();
        assert_eq!(nulid.nanos(), 1_704_067_200_000_000_000u128);
    }
}

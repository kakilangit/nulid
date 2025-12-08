//! Timestamp handling for NULID with 68-bit nanosecond precision.
//!
//! This module provides a timestamp type that represents time in nanoseconds
//! since the UNIX epoch, using 68 bits to match ULID's lifespan (until ~10889 AD).

use crate::{Error, Result};
use chrono::{DateTime, Utc};
use core::fmt;

/// Maximum value for a 68-bit timestamp (2^68 - 1).
/// This provides nanosecond precision until approximately year 10889 AD.
const MAX_TIMESTAMP: u128 = (1_u128 << 68) - 1;

/// A 68-bit timestamp representing nanoseconds since the UNIX epoch.
///
/// The timestamp is valid from 0 (January 1, 1970 00:00:00 UTC) to
/// 2^68 - 1 nanoseconds (approximately year 10889 AD).
///
/// Internally stored as u128 to accommodate the 68-bit value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Timestamp(u128);

impl Timestamp {
    /// Creates a new timestamp from the current system time.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The current time is before the UNIX epoch
    /// - The timestamp value exceeds 68 bits
    ///
    /// # Example
    ///
    /// ```rust
    /// use nulid::timestamp::Timestamp;
    ///
    /// let ts = Timestamp::now();
    /// assert!(ts.is_ok());
    /// ```
    pub fn now() -> Result<Self> {
        Self::from_datetime(Utc::now())
    }

    /// Creates a timestamp from nanoseconds since the UNIX epoch.
    ///
    /// # Errors
    ///
    /// Returns `Error::TimestampOverflow` if the value exceeds 68 bits.
    ///
    /// # Example
    ///
    /// ```rust
    /// use nulid::timestamp::Timestamp;
    ///
    /// let ts = Timestamp::from_nanos(1_000_000_000);
    /// assert!(ts.is_ok());
    /// assert_eq!(ts.unwrap().as_nanos(), 1_000_000_000);
    /// ```
    pub const fn from_nanos(nanos: u128) -> Result<Self> {
        if nanos > MAX_TIMESTAMP {
            return Err(Error::TimestampOverflow);
        }
        Ok(Self(nanos))
    }

    /// Creates a timestamp from a `DateTime<Utc>`.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The datetime is before the UNIX epoch
    /// - The timestamp cannot be represented in nanoseconds
    /// - The timestamp value exceeds 68 bits
    ///
    /// # Example
    ///
    /// ```rust
    /// use nulid::timestamp::Timestamp;
    /// use chrono::Utc;
    ///
    /// let dt = Utc::now();
    /// let ts = Timestamp::from_datetime(dt);
    /// assert!(ts.is_ok());
    /// ```
    pub fn from_datetime(dt: DateTime<Utc>) -> Result<Self> {
        // Use timestamp_nanos_opt() which returns Option<i64>
        let nanos_i64 = dt.timestamp_nanos_opt().ok_or(Error::TimestampOverflow)?;

        // Check if negative (before epoch)
        if nanos_i64 < 0 {
            return Err(Error::InvalidTimestamp);
        }

        // Safe to cast since we checked it's not negative
        // Convert i64 to u128 for 68-bit storage
        #[allow(clippy::cast_sign_loss)]
        let nanos = nanos_i64 as u128;

        Self::from_nanos(nanos)
    }

    /// Returns the timestamp as nanoseconds since the UNIX epoch.
    ///
    /// # Example
    ///
    /// ```rust
    /// use nulid::timestamp::Timestamp;
    ///
    /// let ts = Timestamp::from_nanos(1_234_567_890).unwrap();
    /// assert_eq!(ts.as_nanos(), 1_234_567_890);
    /// ```
    #[must_use]
    pub const fn as_nanos(&self) -> u128 {
        self.0
    }

    /// Converts the timestamp to a `DateTime<Utc>`.
    ///
    /// # Errors
    ///
    /// Returns `Error::InvalidTimestamp` if the conversion fails.
    ///
    /// # Example
    ///
    /// ```rust
    /// use nulid::timestamp::Timestamp;
    ///
    /// let ts = Timestamp::from_nanos(1_000_000_000).unwrap();
    /// let dt = ts.to_datetime().unwrap();
    /// assert_eq!(dt.timestamp(), 1);
    /// assert_eq!(dt.timestamp_subsec_nanos(), 0);
    /// ```
    pub fn to_datetime(&self) -> Result<DateTime<Utc>> {
        // Check if the value fits in i64 for chrono
        if self.0 > i64::MAX as u128 {
            return Err(Error::TimestampOverflow);
        }

        #[allow(clippy::cast_possible_wrap)]
        #[allow(clippy::cast_possible_truncation)]
        let nanos_i64 = self.0 as i64;

        let secs = nanos_i64 / 1_000_000_000;
        #[allow(clippy::cast_sign_loss)]
        let subsec_nanos = (nanos_i64 % 1_000_000_000) as u32;

        DateTime::from_timestamp(secs, subsec_nanos).ok_or(Error::InvalidTimestamp)
    }

    /// Returns the maximum valid timestamp value (2^68 - 1).
    #[must_use]
    pub const fn max_value() -> u128 {
        MAX_TIMESTAMP
    }

    /// Converts the timestamp to bytes (big-endian, 9 bytes for 68 bits).
    ///
    /// Only the lower 68 bits are used. The first byte will only use 4 bits.
    #[must_use]
    pub fn to_bytes(&self) -> [u8; 9] {
        let bytes = self.0.to_be_bytes();
        // Take the last 9 bytes (72 bits) which contain our 68-bit value
        let mut result = [0u8; 9];
        result.copy_from_slice(&bytes[7..16]);
        result
    }

    /// Creates a timestamp from bytes (big-endian, 9 bytes for 68 bits).
    ///
    /// # Errors
    ///
    /// Returns `Error::TimestampOverflow` if the value exceeds 68 bits.
    pub fn from_bytes(bytes: &[u8; 9]) -> Result<Self> {
        // Construct u128 from 9 bytes
        let mut full_bytes = [0u8; 16];
        full_bytes[7..16].copy_from_slice(bytes);
        let value = u128::from_be_bytes(full_bytes);
        Self::from_nanos(value)
    }
}

impl fmt::Display for Timestamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_from_nanos_valid() {
        let result = Timestamp::from_nanos(1_000_000_000);
        assert!(result.is_ok());
        if let Ok(ts) = result {
            assert_eq!(ts.as_nanos(), 1_000_000_000);
        }
    }

    #[test]
    fn test_from_nanos_zero() {
        let result = Timestamp::from_nanos(0);
        assert!(result.is_ok());
        if let Ok(ts) = result {
            assert_eq!(ts.as_nanos(), 0);
        }
    }

    #[test]
    fn test_from_nanos_max() {
        let result = Timestamp::from_nanos(MAX_TIMESTAMP);
        assert!(result.is_ok());
        if let Ok(ts) = result {
            assert_eq!(ts.as_nanos(), MAX_TIMESTAMP);
        }
    }

    #[test]
    fn test_from_nanos_overflow() {
        let result = Timestamp::from_nanos(MAX_TIMESTAMP + 1);
        assert_eq!(result, Err(Error::TimestampOverflow));
    }

    #[test]
    fn test_from_datetime() {
        let dt = Utc.timestamp_opt(1, 0).unwrap();
        let result = Timestamp::from_datetime(dt);
        assert!(result.is_ok());
        if let Ok(ts) = result {
            assert_eq!(ts.as_nanos(), 1_000_000_000);
        }
    }

    #[test]
    fn test_from_datetime_with_nanos() {
        let dt = Utc.timestamp_opt(1, 500_000_000).unwrap();
        let result = Timestamp::from_datetime(dt);
        assert!(result.is_ok());
        if let Ok(ts) = result {
            assert_eq!(ts.as_nanos(), 1_500_000_000);
        }
    }

    #[test]
    fn test_to_datetime() {
        let ts = Timestamp::from_nanos(1_500_000_000).unwrap();
        let result = ts.to_datetime();
        assert!(result.is_ok());
        if let Ok(dt) = result {
            assert_eq!(dt.timestamp(), 1);
            assert_eq!(dt.timestamp_subsec_nanos(), 500_000_000);
        }
    }

    #[test]
    fn test_round_trip_datetime() {
        let original = Utc.timestamp_opt(123, 456_789_000).unwrap();
        let ts = Timestamp::from_datetime(original).unwrap();
        let result = ts.to_datetime();
        assert!(result.is_ok());
        if let Ok(converted) = result {
            assert_eq!(original.timestamp(), converted.timestamp());
            assert_eq!(
                original.timestamp_subsec_nanos(),
                converted.timestamp_subsec_nanos()
            );
        }
    }

    #[test]
    fn test_now() {
        let result = Timestamp::now();
        assert!(result.is_ok());
        if let Ok(ts) = result {
            assert!(ts.as_nanos() > 0);
            assert!(ts.as_nanos() <= MAX_TIMESTAMP);
        }
    }

    #[test]
    fn test_ordering() {
        let ts1 = Timestamp::from_nanos(1000).unwrap();
        let ts2 = Timestamp::from_nanos(2000).unwrap();
        assert!(ts1 < ts2);
        assert!(ts2 > ts1);
    }

    #[test]
    fn test_equality() {
        let ts1 = Timestamp::from_nanos(1000).unwrap();
        let ts2 = Timestamp::from_nanos(1000).unwrap();
        let ts3 = Timestamp::from_nanos(2000).unwrap();
        assert_eq!(ts1, ts2);
        assert_ne!(ts1, ts3);
    }

    #[test]
    fn test_display() {
        let ts = Timestamp::from_nanos(1_234_567_890).unwrap();
        assert_eq!(ts.to_string(), "1234567890");
    }

    #[test]
    fn test_max_value() {
        assert_eq!(Timestamp::max_value(), MAX_TIMESTAMP);
        assert_eq!(Timestamp::max_value(), (1_u128 << 68) - 1);
    }

    #[test]
    fn test_clone_copy() {
        let ts1 = Timestamp::from_nanos(1000).unwrap();
        let ts2 = ts1;
        assert_eq!(ts1, ts2);
    }

    #[test]
    fn test_before_epoch() {
        let before_epoch = Utc.timestamp_opt(-1, 0).unwrap();
        let result = Timestamp::from_datetime(before_epoch);
        assert_eq!(result, Err(Error::InvalidTimestamp));
    }

    #[test]
    fn test_bytes_round_trip() {
        let ts = Timestamp::from_nanos(1_234_567_890_123_456_789).unwrap();
        let bytes = ts.to_bytes();
        let ts2 = Timestamp::from_bytes(&bytes).unwrap();
        assert_eq!(ts, ts2);
    }

    #[test]
    fn test_bytes_max_value() {
        let ts = Timestamp::from_nanos(MAX_TIMESTAMP).unwrap();
        let bytes = ts.to_bytes();
        let ts2 = Timestamp::from_bytes(&bytes).unwrap();
        assert_eq!(ts, ts2);
    }

    #[test]
    fn test_68_bit_capacity() {
        // Verify we can store the full 68-bit range
        let max_68bit = (1_u128 << 68) - 1;
        let ts = Timestamp::from_nanos(max_68bit).unwrap();
        assert_eq!(ts.as_nanos(), max_68bit);
    }
}

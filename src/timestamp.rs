//! Timestamp handling for NULID with 68-bit nanosecond precision.
//!
//! This module provides a timestamp type that represents time in nanoseconds
//! since the UNIX epoch, using 68 bits to match ULID's lifespan (until ~10889 AD).

use crate::{Error, Result};
use core::fmt;
use hifitime::{Duration, Epoch as HifiEpoch, UNIX_REF_EPOCH};

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
    /// - The current time cannot be retrieved
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
        let epoch = HifiEpoch::now().map_err(|_| Error::InvalidTimestamp)?;
        Self::from_hifitime_epoch(epoch)
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

    /// Creates a timestamp from a hifitime `Epoch`.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The epoch is before the UNIX epoch
    /// - The timestamp cannot be represented in nanoseconds
    /// - The timestamp value exceeds 68 bits
    ///
    /// # Example
    ///
    /// ```rust
    /// use nulid::timestamp::Timestamp;
    /// use hifitime::Epoch;
    ///
    /// let epoch = Epoch::now().unwrap();
    /// let ts = Timestamp::from_hifitime_epoch(epoch);
    /// assert!(ts.is_ok());
    /// ```
    pub fn from_hifitime_epoch(epoch: HifiEpoch) -> Result<Self> {
        // Get the duration since UNIX epoch by subtracting
        let duration = epoch - UNIX_REF_EPOCH;

        // Convert to nanoseconds (returns i128)
        let total_nanos = duration.total_nanoseconds();

        // Check if negative (before UNIX epoch)
        if total_nanos < 0 {
            return Err(Error::InvalidTimestamp);
        }

        // Convert to u128
        #[allow(clippy::cast_sign_loss)]
        let nanos = total_nanos as u128;

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

    /// Converts the timestamp to a hifitime `Epoch`.
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
    /// let epoch = ts.to_hifitime_epoch().unwrap();
    /// ```
    pub fn to_hifitime_epoch(&self) -> Result<HifiEpoch> {
        // Convert nanoseconds to Duration
        // hifitime uses centuries and nanoseconds internally, so we need to be careful
        #[allow(clippy::cast_possible_truncation)]
        let total_nanos_i64 = self.0 as i64;

        let duration = Duration::from_truncated_nanoseconds(total_nanos_i64);

        // Create epoch from UNIX epoch + duration
        Ok(UNIX_REF_EPOCH + duration)
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
    fn test_from_hifitime_epoch() {
        let epoch = HifiEpoch::from_unix_duration(Duration::from_seconds(1.0));
        let result = Timestamp::from_hifitime_epoch(epoch);
        assert!(result.is_ok());
        if let Ok(ts) = result {
            assert_eq!(ts.as_nanos(), 1_000_000_000);
        }
    }

    #[test]
    fn test_from_hifitime_epoch_with_nanos() {
        let duration = Duration::from_seconds(1.0) + Duration::from_milliseconds(500.0);
        let epoch = HifiEpoch::from_unix_duration(duration);
        let result = Timestamp::from_hifitime_epoch(epoch);
        assert!(result.is_ok());
        if let Ok(ts) = result {
            assert_eq!(ts.as_nanos(), 1_500_000_000);
        }
    }

    #[test]
    fn test_to_hifitime_epoch() {
        let ts = Timestamp::from_nanos(1_500_000_000).unwrap();
        let result = ts.to_hifitime_epoch();
        assert!(result.is_ok());
        if let Ok(epoch) = result {
            let duration = epoch - UNIX_REF_EPOCH;
            // Check nanoseconds
            let nanos = duration.total_nanoseconds();
            assert_eq!(nanos, 1_500_000_000);
        }
    }

    #[test]
    fn test_round_trip_hifitime_epoch() {
        let duration = Duration::from_seconds(123.0)
            + Duration::from_milliseconds(456.0)
            + Duration::from_microseconds(789.0);
        let original = HifiEpoch::from_unix_duration(duration);
        let ts = Timestamp::from_hifitime_epoch(original).unwrap();
        let result = ts.to_hifitime_epoch();
        assert!(result.is_ok());
        if let Ok(converted) = result {
            let orig_duration = original - UNIX_REF_EPOCH;
            let conv_duration = converted - UNIX_REF_EPOCH;
            // Check nanoseconds are close (within 1 nanosecond due to conversions)
            let diff =
                (orig_duration.total_nanoseconds() - conv_duration.total_nanoseconds()).abs();
            assert!(diff < 2, "Nanosecond difference too large: {diff}");
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
        let duration = Duration::from_seconds(-1.0);
        let before_epoch = HifiEpoch::from_unix_duration(duration);
        let result = Timestamp::from_hifitime_epoch(before_epoch);
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

    #[test]
    fn test_high_precision() {
        // Test that we can represent nanosecond precision accurately
        let nanos = 1_234_567_890_123_456_789_u128;
        let ts = Timestamp::from_nanos(nanos).unwrap();
        assert_eq!(ts.as_nanos(), nanos);

        let epoch = ts.to_hifitime_epoch().unwrap();
        let ts2 = Timestamp::from_hifitime_epoch(epoch).unwrap();
        // Allow for small rounding due to f64 precision in hifitime
        let diff = if ts.as_nanos() > ts2.as_nanos() {
            ts.as_nanos() - ts2.as_nanos()
        } else {
            ts2.as_nanos() - ts.as_nanos()
        };
        assert!(
            diff < 10,
            "High precision round trip failed with diff: {diff}"
        );
    }
}

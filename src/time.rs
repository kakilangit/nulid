//! Time utilities for nanosecond-precision timestamps.

use crate::{Error, Result};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Maximum valid nanosecond timestamp (2^68 - 1).
pub const MAX_TIMESTAMP_NANOS: u128 = (1u128 << 68) - 1;

/// Returns the current time as nanoseconds since Unix epoch.
///
/// # Errors
///
/// Returns an error if the system time is before Unix epoch.
///
/// # Examples
///
/// ```
/// use nulid::time::now_nanos;
///
/// # fn main() -> nulid::Result<()> {
/// let nanos = now_nanos()?;
/// assert!(nanos > 0);
/// # Ok(())
/// # }
/// ```
#[inline]
pub fn now_nanos() -> Result<u128> {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| Error::SystemTimeError)?;

    Ok(u128::from(duration.as_secs()) * 1_000_000_000 + u128::from(duration.subsec_nanos()))
}

/// Converts nanoseconds since Unix epoch to `SystemTime`.
///
/// # Examples
///
/// ```
/// use nulid::time::{now_nanos, from_nanos};
///
/// # fn main() -> nulid::Result<()> {
/// let nanos = now_nanos()?;
/// let time = from_nanos(nanos);
/// # Ok(())
/// # }
/// ```
#[must_use]
#[allow(clippy::cast_possible_truncation)]
pub fn from_nanos(timestamp_nanos: u128) -> SystemTime {
    let secs = (timestamp_nanos / 1_000_000_000) as u64;
    let subsec_nanos = (timestamp_nanos % 1_000_000_000) as u32;
    UNIX_EPOCH + Duration::new(secs, subsec_nanos)
}

/// Converts nanoseconds since Unix epoch to `Duration`.
///
/// # Examples
///
/// ```
/// use nulid::time::to_duration;
///
/// let duration = to_duration(5_000_000_000);
/// assert_eq!(duration.as_secs(), 5);
/// ```
#[must_use]
#[allow(clippy::cast_possible_truncation)]
pub const fn to_duration(timestamp_nanos: u128) -> Duration {
    let secs = (timestamp_nanos / 1_000_000_000) as u64;
    let subsec_nanos = (timestamp_nanos % 1_000_000_000) as u32;
    Duration::new(secs, subsec_nanos)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_now_nanos() {
        let nanos = now_nanos().unwrap();
        assert!(nanos > 0);
        assert!(nanos < MAX_TIMESTAMP_NANOS);
    }

    #[test]
    fn test_from_nanos() {
        let nanos = 1_234_567_890_123_456_789u128;
        let time = from_nanos(nanos);
        let duration = time.duration_since(UNIX_EPOCH).unwrap();

        let reconstructed =
            u128::from(duration.as_secs()) * 1_000_000_000 + u128::from(duration.subsec_nanos());

        assert_eq!(reconstructed, nanos);
    }

    #[test]
    fn test_to_duration() {
        let nanos = 5_123_456_789u128;
        let duration = to_duration(nanos);

        assert_eq!(duration.as_secs(), 5);
        assert_eq!(duration.subsec_nanos(), 123_456_789);
    }

    #[test]
    fn test_round_trip() {
        let nanos = now_nanos().unwrap();
        let time = from_nanos(nanos);
        let duration = time.duration_since(UNIX_EPOCH).unwrap();

        let reconstructed =
            u128::from(duration.as_secs()) * 1_000_000_000 + u128::from(duration.subsec_nanos());

        assert_eq!(reconstructed, nanos);
    }

    #[test]
    fn test_zero() {
        let time = from_nanos(0);
        assert_eq!(time, UNIX_EPOCH);

        let duration = to_duration(0);
        assert_eq!(duration.as_secs(), 0);
        assert_eq!(duration.subsec_nanos(), 0);
    }

    #[test]
    fn test_large_value() {
        let nanos = 9_999_999_999_999_999_999u128; // ~316 years
        let time = from_nanos(nanos);
        let duration = time.duration_since(UNIX_EPOCH).unwrap();

        assert!(duration.as_secs() > 0);
    }
}

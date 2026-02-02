//! Time utilities for nanosecond-precision timestamps.

use crate::{Error, Result};
use core::time::Duration;
use quanta::Clock;
use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};

/// Initialization data for the clock.
/// Stores the base wall-clock time and the corresponding quanta clock reading.
struct ClockBase {
    /// Wall-clock nanoseconds since Unix epoch at initialization
    base_wall_nanos: u128,
    /// Quanta clock reading at initialization (in nanoseconds)
    base_quanta_nanos: u64,
}

/// Global clock instances, initialized on first call to `now_nanos()`
static CLOCK: OnceLock<Clock> = OnceLock::new();
static CLOCK_BASE: OnceLock<ClockBase> = OnceLock::new();

/// Returns the current time as nanoseconds since Unix epoch.
///
/// Uses `quanta` for true nanosecond precision on all platforms:
/// - Combines wall-clock time with high-resolution monotonic counter
/// - Provides true nanosecond precision (not rounded to microseconds)
/// - Cross-platform support (macOS, Linux, Windows)
/// - Monotonically increasing for proper ordering
///
/// # How it works
///
/// 1. On first call, captures current wall-clock time and quanta counter value
/// 2. Subsequent calls add the elapsed quanta time to the base wall-clock time
/// 3. This gives true nanosecond precision even on macOS (which has microsecond wall clock)
///
/// # Note on accuracy vs precision
///
/// - **Precision**: True nanosecond granularity (values don't round to thousands)
/// - **Accuracy**: Relative to system wall-clock, may drift slightly over long periods
/// - For NULID uniqueness and ordering, precision is more important than absolute accuracy
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
pub fn now_nanos() -> Result<u128> {
    // Initialize clock on first use
    let clock = CLOCK.get_or_init(Clock::new);

    // Get or initialize the clock base
    let clock_base = CLOCK_BASE.get_or_init(|| {
        let wall_nanos = get_wall_clock_nanos().unwrap_or(0);
        let quanta_nanos = clock.raw();
        ClockBase {
            base_wall_nanos: wall_nanos,
            base_quanta_nanos: quanta_nanos,
        }
    });

    // Calculate elapsed time since base using quanta's high-resolution clock
    let current_quanta_nanos = clock.raw();
    let elapsed_nanos = current_quanta_nanos - clock_base.base_quanta_nanos;

    // Add elapsed time to base wall-clock time
    Ok(clock_base.base_wall_nanos + u128::from(elapsed_nanos))
}

/// Gets the current wall-clock time in nanoseconds since Unix epoch.
/// This is used for initialization only; subsequent calls use quanta's high-resolution timer.
fn get_wall_clock_nanos() -> Result<u128> {
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
#[allow(clippy::missing_const_for_fn)]
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

    /// Maximum valid nanosecond timestamp (2^68 - 1).
    const MAX_TIMESTAMP_NANOS: u128 = (1u128 << 68) - 1;

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

    #[test]
    fn test_nanosecond_precision() {
        // Test that we get true nanosecond precision with quanta
        let mut has_non_zero_nanos = false;
        let mut last_three_digits_seen = std::collections::HashSet::new();

        // Sample multiple times to check for nanosecond-level variation
        for _ in 0..1000 {
            let nanos = now_nanos().unwrap();
            let last_three_digits = nanos % 1000;

            last_three_digits_seen.insert(last_three_digits);

            // If we have true nanosecond precision, we should see
            // non-zero values in the last 3 digits
            if last_three_digits != 0 {
                has_non_zero_nanos = true;
            }
        }

        // With quanta, we should get true nanosecond precision on all platforms
        assert!(
            has_non_zero_nanos,
            "Expected nanosecond precision with quanta, but all samples were rounded to microseconds"
        );

        // We should see significant variety in the last 3 digits
        assert!(
            last_three_digits_seen.len() > 10,
            "Expected variety in nanosecond digits, but only saw {} unique values: {:?}",
            last_three_digits_seen.len(),
            last_three_digits_seen
        );
    }

    #[test]
    fn test_monotonic_ordering() {
        // Test that timestamps are monotonically increasing
        let mut prev_nanos = now_nanos().unwrap();

        for _ in 0..100 {
            let nanos = now_nanos().unwrap();
            assert!(
                nanos >= prev_nanos,
                "Timestamps should be monotonically increasing: {nanos} >= {prev_nanos}"
            );
            prev_nanos = nanos;
        }
    }

    #[test]
    fn test_nanosecond_storage() {
        // Test that we can store and retrieve nanosecond precision values
        let nanos1 = now_nanos().unwrap();
        let nanos2 = now_nanos().unwrap();

        // Timestamps should be monotonically increasing or equal
        assert!(nanos2 >= nanos1);

        // Verify we're getting reasonable values (not zeros)
        assert!(nanos1 > 1_000_000_000_000_000_000); // After year 2001

        // Test that nanosecond values are preserved through conversion
        let test_nanos = 1_234_567_890_123_456_789u128;
        let time = from_nanos(test_nanos);
        let duration = time.duration_since(UNIX_EPOCH).unwrap();
        let reconstructed =
            u128::from(duration.as_secs()) * 1_000_000_000 + u128::from(duration.subsec_nanos());
        assert_eq!(reconstructed, test_nanos);
    }
}

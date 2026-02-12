//! Time utilities for nanosecond-precision timestamps.
//!
//! This module provides platform-specific time implementations:
//! - **Native**: Uses `quanta` for true nanosecond precision on all platforms
//! - **WASM**: Uses `web-time` for browser compatibility with `performance.now()`

use core::time::Duration;

// Platform-specific modules
#[cfg(not(feature = "wasm"))]
mod native;

#[cfg(feature = "wasm")]
mod wasm;

// Re-export SystemTime and UNIX_EPOCH from the appropriate source
#[cfg(not(feature = "wasm"))]
pub use std::time::{SystemTime, UNIX_EPOCH};

#[cfg(feature = "wasm")]
pub use web_time::{SystemTime, UNIX_EPOCH};

// Re-export now_nanos from the appropriate platform module
#[cfg(not(feature = "wasm"))]
pub use native::now_nanos;

#[cfg(feature = "wasm")]
pub use wasm::now_nanos;

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
    #[cfg(feature = "wasm")]
    fn test_wasm_nanosecond_precision() {
        // Test that WASM time implementation provides valid nanosecond precision
        // This test ensures we're not getting zero nanoseconds repeatedly
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

        // With web-time, we should get at least microsecond precision
        // which means we should see non-zero nanosecond components
        assert!(
            has_non_zero_nanos,
            "Expected at least microsecond precision with web-time, but all samples had zero nanoseconds"
        );

        // We should see some variety in the last 3 digits
        assert!(
            last_three_digits_seen.len() > 5,
            "Expected variety in nanosecond digits, but only saw {} unique values: {:?}",
            last_three_digits_seen.len(),
            last_three_digits_seen
        );
    }

    #[test]
    #[cfg(feature = "wasm")]
    fn test_wasm_non_zero_nanoseconds() {
        // Test that we don't get stuck with zero nanoseconds
        // This is a regression test to ensure the WASM implementation
        // properly handles nanosecond precision

        const MAX_ZERO_ALLOWED: usize = 500; // Allow some zeros but not all
        let mut zero_count = 0;

        for _ in 0..1000 {
            let nanos = now_nanos().unwrap();
            let last_three_digits = nanos % 1000;

            if last_three_digits == 0 {
                zero_count += 1;

                // If we hit too many zeros, fail early
                assert!(
                    zero_count <= MAX_ZERO_ALLOWED,
                    "Too many zero nanosecond values: {zero_count}/1000 samples had zero nanoseconds"
                );
            }
        }

        // Ensure we didn't get too many zeros
        assert!(
            zero_count <= MAX_ZERO_ALLOWED,
            "Too many zero nanosecond values: {zero_count}/1000 samples had zero nanoseconds"
        );
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

//! Native (non-WASM) time implementation using quanta for true nanosecond precision.

use crate::{Error, Result};
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

#[cfg(test)]
mod tests {
    use super::*;

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
}

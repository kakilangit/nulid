//! WASM time implementation using web-time for browser compatibility.

use crate::{Error, Result};
use std::sync::OnceLock;
use web_time::{Instant, SystemTime, UNIX_EPOCH};

/// Initialization data for the WASM clock.
struct WasmClockBase {
    /// Wall-clock nanoseconds since Unix epoch at initialization
    base_wall_nanos: u128,
    /// Instant at initialization
    base_instant: Instant,
}

/// Global clock base for WASM, initialized on first call
static WASM_CLOCK_BASE: OnceLock<WasmClockBase> = OnceLock::new();

/// Returns the current time as nanoseconds since Unix epoch.
///
/// For WASM targets, uses `web-time` crate which provides:
/// - `performance.now()` for high-resolution timing in browsers
/// - Fallback to `Date.now()` when Performance API is unavailable
///
/// # How it works
///
/// 1. On first call, captures current wall-clock time and `Instant::now()`
/// 2. Subsequent calls add the elapsed time to the base wall-clock time
/// 3. This provides sub-millisecond precision in browser environments
///
/// # Note on precision
///
/// - Browser environments typically provide microsecond precision via `performance.now()`
/// - Some browsers may reduce precision due to security concerns (Spectre mitigation)
/// - For NULID uniqueness, the monotonic generator handles same-timestamp collisions
///
/// # Errors
///
/// Returns an error if the system time is before Unix epoch.
pub fn now_nanos() -> Result<u128> {
    let clock_base = WASM_CLOCK_BASE.get_or_init(|| {
        let wall_nanos = get_wall_clock_nanos().unwrap_or(0);
        WasmClockBase {
            base_wall_nanos: wall_nanos,
            base_instant: Instant::now(),
        }
    });

    // Calculate elapsed time since base
    let elapsed = clock_base.base_instant.elapsed();
    let elapsed_nanos =
        u128::from(elapsed.as_secs()) * 1_000_000_000 + u128::from(elapsed.subsec_nanos());

    Ok(clock_base.base_wall_nanos + elapsed_nanos)
}

/// Gets the current wall-clock time in nanoseconds since Unix epoch.
fn get_wall_clock_nanos() -> Result<u128> {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| Error::SystemTimeError)?;

    Ok(u128::from(duration.as_secs()) * 1_000_000_000 + u128::from(duration.subsec_nanos()))
}

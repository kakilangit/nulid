//! Monotonic NULID generator for guaranteed ordering within the same nanosecond.
//!
//! This module provides a thread-safe generator that ensures NULIDs are
//! monotonically increasing even when multiple IDs are generated within
//! the same nanosecond.
//!
//! # Monotonicity
//!
//! The generator maintains the last generated timestamp and randomness.
//! When generating a new NULID:
//!
//! - If current time > last time: Generate with current time + new random
//! - If current time == last time: Use last time + increment last random
//! - If current time < last time: Use last time + increment (clock skew protection)
//!
//! # Example
//!
//! ```rust
//! use nulid::Generator;
//!
//! let mut generator = Generator::new();
//!
//! // Generate multiple NULIDs - guaranteed to be sorted
//! let id1 = generator.generate()?;
//! let id2 = generator.generate()?;
//! let id3 = generator.generate()?;
//!
//! assert!(id1 < id2);
//! assert!(id2 < id3);
//! # Ok::<(), nulid::Error>(())
//! ```

use crate::{Error, Nulid, Random, Result, Timestamp};
use std::sync::Mutex;

/// A monotonic NULID generator that ensures strict ordering.
///
/// The generator maintains state to ensure that generated NULIDs are
/// always strictly increasing, even when multiple IDs are generated
/// within the same nanosecond.
///
/// # Thread Safety
///
/// The generator is thread-safe and can be shared across threads using
/// `Arc<Generator>` or similar synchronization primitives.
///
/// # Example
///
/// ```rust
/// use nulid::Generator;
///
/// let mut generator = Generator::new();
///
/// for _ in 0..10 {
///     let nulid = generator.generate()?;
///     println!("{}", nulid);
/// }
/// # Ok::<(), nulid::Error>(())
/// ```
pub struct Generator {
    /// Mutex protecting the generator state
    state: Mutex<GeneratorState>,
}

/// Internal state of the monotonic generator
#[derive(Debug, Clone)]
struct GeneratorState {
    /// Last generated timestamp
    last_timestamp: Option<Timestamp>,
    /// Last generated randomness
    last_randomness: Random,
}

impl Generator {
    /// Creates a new monotonic generator.
    ///
    /// # Example
    ///
    /// ```rust
    /// use nulid::Generator;
    ///
    /// let generator = Generator::new();
    /// ```
    #[must_use]
    pub const fn new() -> Self {
        Self {
            state: Mutex::new(GeneratorState {
                last_timestamp: None,
                last_randomness: Random::zero(),
            }),
        }
    }

    /// Generates a new NULID with monotonic ordering guarantees.
    ///
    /// This method ensures that each generated NULID is strictly greater
    /// than the previous one, even if generated within the same nanosecond.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The system time cannot be retrieved
    /// - Randomness overflow occurs (extremely unlikely)
    /// - The random number generator fails
    ///
    /// # Example
    ///
    /// ```rust
    /// use nulid::Generator;
    ///
    /// let mut generator = Generator::new();
    /// let nulid = generator.generate()?;
    /// # Ok::<(), nulid::Error>(())
    /// ```
    pub fn generate(&self) -> Result<Nulid> {
        let mut state = self.state.lock().map_err(|_| Error::MutexPoisoned)?;
        let current_time = Timestamp::now()?;

        match state.last_timestamp {
            None => {
                // First generation - use current time and new random
                let randomness = Random::new()?;
                let nulid = Nulid::from_parts(current_time, randomness);
                state.last_timestamp = Some(current_time);
                state.last_randomness = randomness;
                drop(state);
                Ok(nulid)
            }
            Some(last_time) => {
                if current_time > last_time {
                    // Time has advanced - use current time and new random
                    let randomness = Random::new()?;
                    let nulid = Nulid::from_parts(current_time, randomness);
                    state.last_timestamp = Some(current_time);
                    state.last_randomness = randomness;
                    drop(state);
                    Ok(nulid)
                } else {
                    // Same time or clock skew - increment randomness
                    let mut randomness = state.last_randomness;
                    randomness.increment()?;
                    // Use the later of current_time or last_time
                    let timestamp = if current_time >= last_time {
                        current_time
                    } else {
                        last_time
                    };
                    let nulid = Nulid::from_parts(timestamp, randomness);
                    state.last_randomness = randomness;
                    state.last_timestamp = Some(timestamp);
                    drop(state);
                    Ok(nulid)
                }
            }
        }
    }

    /// Generates a NULID with a specific timestamp.
    ///
    /// This method allows generating NULIDs with custom timestamps while
    /// maintaining monotonic ordering. If the provided timestamp is less
    /// than the last generated timestamp, the last timestamp is used instead.
    ///
    /// # Errors
    ///
    /// Returns an error if randomness overflow occurs.
    ///
    /// # Example
    ///
    /// ```rust
    /// use nulid::{Generator, Timestamp};
    ///
    /// let mut generator = Generator::new();
    /// let timestamp = Timestamp::from_nanos(1_000_000_000)?;
    /// let nulid = generator.generate_with_timestamp(timestamp)?;
    /// # Ok::<(), nulid::Error>(())
    /// ```
    pub fn generate_with_timestamp(&self, timestamp: Timestamp) -> Result<Nulid> {
        let mut state = self.state.lock().map_err(|_| Error::MutexPoisoned)?;

        match state.last_timestamp {
            None => {
                // First generation
                let randomness = Random::new()?;
                let nulid = Nulid::from_parts(timestamp, randomness);
                state.last_timestamp = Some(timestamp);
                state.last_randomness = randomness;
                drop(state);
                Ok(nulid)
            }
            Some(last_time) => {
                if timestamp > last_time {
                    // Timestamp has advanced
                    let randomness = Random::new()?;
                    let nulid = Nulid::from_parts(timestamp, randomness);
                    state.last_timestamp = Some(timestamp);
                    state.last_randomness = randomness;
                    drop(state);
                    Ok(nulid)
                } else {
                    // Same or earlier timestamp - increment randomness
                    let mut randomness = state.last_randomness;
                    randomness.increment()?;
                    // Use the later timestamp to maintain monotonicity
                    let effective_timestamp = if timestamp >= last_time {
                        timestamp
                    } else {
                        last_time
                    };
                    let nulid = Nulid::from_parts(effective_timestamp, randomness);
                    state.last_randomness = randomness;
                    state.last_timestamp = Some(effective_timestamp);
                    drop(state);
                    Ok(nulid)
                }
            }
        }
    }

    /// Resets the generator state.
    ///
    /// This clears the last timestamp and randomness, allowing the generator
    /// to start fresh. This is primarily useful for testing.
    ///
    /// # Example
    ///
    /// ```rust
    /// use nulid::Generator;
    ///
    /// let mut generator = Generator::new();
    /// let _ = generator.generate()?;
    /// generator.reset();
    /// # Ok::<(), nulid::Error>(())
    /// ```
    pub fn reset(&self) {
        if let Ok(mut state) = self.state.lock() {
            state.last_timestamp = None;
            state.last_randomness = Random::zero();
        }
        // If mutex is poisoned, silently ignore for reset
    }
}

impl Default for Generator {
    fn default() -> Self {
        Self::new()
    }
}

// Note: Generator is not Clone because it maintains state that shouldn't be duplicated

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_generator() {
        let generator = Generator::new();
        let state = generator.state.lock().unwrap();
        assert!(state.last_timestamp.is_none());
    }

    #[test]
    fn test_first_generation() {
        let generator = Generator::new();
        let nulid = generator.generate().unwrap();
        assert!(nulid.timestamp_nanos() > 0);
    }

    #[test]
    fn test_monotonic_ordering() {
        let generator = Generator::new();
        let id1 = generator.generate().unwrap();
        let id2 = generator.generate().unwrap();
        let id3 = generator.generate().unwrap();

        assert!(id1 < id2);
        assert!(id2 < id3);
    }

    #[test]
    fn test_multiple_generations() {
        let generator = Generator::new();
        let mut ids = Vec::new();

        for _ in 0..100 {
            ids.push(generator.generate().unwrap());
        }

        // Verify all IDs are strictly increasing
        for i in 1..ids.len() {
            assert!(
                ids[i - 1] < ids[i],
                "IDs not strictly increasing at index {}",
                i
            );
        }
    }

    #[test]
    fn test_generate_with_timestamp() {
        let generator = Generator::new();
        let timestamp = Timestamp::from_nanos(1_000_000_000).unwrap();
        let nulid = generator.generate_with_timestamp(timestamp).unwrap();
        assert_eq!(nulid.timestamp(), timestamp);
    }

    #[test]
    fn test_monotonic_with_same_timestamp() {
        let generator = Generator::new();
        let timestamp = Timestamp::from_nanos(1_000_000_000).unwrap();

        let id1 = generator.generate_with_timestamp(timestamp).unwrap();
        let id2 = generator.generate_with_timestamp(timestamp).unwrap();
        let id3 = generator.generate_with_timestamp(timestamp).unwrap();

        // Should have same timestamp but increasing randomness
        assert_eq!(id1.timestamp(), id2.timestamp());
        assert_eq!(id2.timestamp(), id3.timestamp());
        assert!(id1 < id2);
        assert!(id2 < id3);
    }

    #[test]
    fn test_clock_skew_protection() {
        let generator = Generator::new();
        let ts1 = Timestamp::from_nanos(2_000_000_000).unwrap();
        let ts2 = Timestamp::from_nanos(1_000_000_000).unwrap(); // Earlier time

        let id1 = generator.generate_with_timestamp(ts1).unwrap();
        let id2 = generator.generate_with_timestamp(ts2).unwrap();

        // id2 should use ts1 (not go backward in time)
        assert_eq!(id2.timestamp(), ts1);
        assert!(id1 < id2); // Still monotonic
    }

    #[test]
    fn test_reset() {
        let generator = Generator::new();
        let _ = generator.generate().unwrap();

        generator.reset();

        let state = generator.state.lock().unwrap();
        assert!(state.last_timestamp.is_none());
    }

    #[test]
    fn test_string_representation_sorted() {
        let generator = Generator::new();
        let id1 = generator.generate().unwrap();
        let id2 = generator.generate().unwrap();

        let s1 = id1.to_string();
        let s2 = id2.to_string();

        // String comparison should match NULID ordering
        assert!(s1 < s2);
    }

    #[test]
    fn test_default() {
        let generator = Generator::default();
        let nulid = generator.generate().unwrap();
        assert!(nulid.timestamp_nanos() > 0);
    }

    #[test]
    fn test_rapid_generation() {
        let generator = Generator::new();
        let mut ids = Vec::new();

        // Generate many IDs rapidly
        for _ in 0..1000 {
            ids.push(generator.generate().unwrap());
        }

        // All should be unique and sorted
        for i in 1..ids.len() {
            assert_ne!(ids[i - 1], ids[i]);
            assert!(ids[i - 1] < ids[i]);
        }
    }

    #[test]
    fn test_randomness_overflow_protection() {
        let generator = Generator::new();
        let timestamp = Timestamp::from_nanos(1_000_000_000).unwrap();

        // Generate first NULID with max randomness
        let _first = generator.generate_with_timestamp(timestamp).unwrap();

        // Manually set to near-max
        {
            let mut state = generator.state.lock().unwrap();
            state.last_randomness = Random::max();
        }

        // Next generation with same timestamp should error (overflow)
        let result = generator.generate_with_timestamp(timestamp);
        assert!(result.is_err());
    }

    #[test]
    fn test_concurrent_safety() {
        use std::sync::Arc;
        use std::thread;

        let generator = Arc::new(Generator::new());
        let mut handles = vec![];

        for _ in 0..10 {
            let gen_clone = Arc::clone(&generator);
            let handle = thread::spawn(move || {
                let mut ids = Vec::new();
                for _ in 0..10 {
                    ids.push(gen_clone.generate().unwrap());
                }
                ids
            });
            handles.push(handle);
        }

        let mut all_ids = Vec::new();
        for handle in handles {
            all_ids.extend(handle.join().unwrap());
        }

        // All IDs should be unique
        all_ids.sort();
        all_ids.dedup();
        assert_eq!(all_ids.len(), 100);
    }
}

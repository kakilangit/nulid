//! Monotonic NULID generator for guaranteed ordering.
//!
//! This module provides a thread-safe generator that ensures NULIDs are
//! monotonically increasing even when multiple IDs are generated within
//! the same nanosecond.

use crate::{Error, Nulid, Result};
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
/// # Examples
///
/// ```
/// use nulid::Generator;
///
/// # fn main() -> nulid::Result<()> {
/// let generator = Generator::new();
///
/// // Generate multiple NULIDs - guaranteed to be sorted
/// let id1 = generator.generate()?;
/// let id2 = generator.generate()?;
/// let id3 = generator.generate()?;
///
/// assert!(id1 < id2);
/// assert!(id2 < id3);
/// # Ok(())
/// # }
/// ```
pub struct Generator {
    /// Mutex protecting the last generated NULID
    state: Mutex<Option<Nulid>>,
}

impl Generator {
    /// Creates a new monotonic generator.
    ///
    /// # Examples
    ///
    /// ```
    /// use nulid::Generator;
    ///
    /// let generator = Generator::new();
    /// ```
    #[must_use]
    pub const fn new() -> Self {
        Self {
            state: Mutex::new(None),
        }
    }

    /// Generates a new NULID with monotonic ordering guarantees.
    ///
    /// This method ensures that each generated NULID is strictly greater
    /// than the previous one, even if generated within the same nanosecond.
    ///
    /// # Monotonicity Strategy
    ///
    /// - If current timestamp > last timestamp: Use current time + new random
    /// - If current timestamp <= last timestamp: Increment last NULID by 1
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The system time cannot be retrieved
    /// - The random number generator fails
    /// - NULID overflow occurs (last NULID is at maximum value)
    ///
    /// # Examples
    ///
    /// ```
    /// use nulid::Generator;
    ///
    /// # fn main() -> nulid::Result<()> {
    /// let generator = Generator::new();
    /// let id = generator.generate()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn generate(&self) -> Result<Nulid> {
        let mut state = self.state.lock().map_err(|_| Error::MutexPoisoned)?;

        let new_id = Nulid::new()?;

        match *state {
            None => {
                // First generation
                *state = Some(new_id);
                Ok(new_id)
            }
            Some(last_id) => {
                if new_id > last_id {
                    // Time has advanced, use new timestamp
                    *state = Some(new_id);
                    Ok(new_id)
                } else {
                    // Same nanosecond or clock skew - increment last ID
                    let incremented = last_id.increment().ok_or(Error::Overflow)?;
                    *state = Some(incremented);
                    Ok(incremented)
                }
            }
        }
    }

    /// Resets the generator state.
    ///
    /// This clears the last generated NULID, allowing the generator
    /// to start fresh. This is primarily useful for testing.
    ///
    /// # Examples
    ///
    /// ```
    /// use nulid::Generator;
    ///
    /// # fn main() -> nulid::Result<()> {
    /// let generator = Generator::new();
    /// let _ = generator.generate()?;
    /// generator.reset();
    /// # Ok(())
    /// # }
    /// ```
    pub fn reset(&self) {
        if let Ok(mut state) = self.state.lock() {
            *state = None;
        }
    }

    /// Returns the last generated NULID, if any.
    ///
    /// # Examples
    ///
    /// ```
    /// use nulid::Generator;
    ///
    /// # fn main() -> nulid::Result<()> {
    /// let generator = Generator::new();
    /// assert!(generator.last().is_none());
    ///
    /// let id = generator.generate()?;
    /// assert_eq!(generator.last(), Some(id));
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn last(&self) -> Option<Nulid> {
        self.state.lock().ok().and_then(|state| *state)
    }
}

impl Default for Generator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_generator() {
        let generator = Generator::new();
        assert!(generator.last().is_none());
    }

    #[test]
    fn test_first_generation() {
        let generator = Generator::new();
        let id = generator.generate().unwrap();
        assert!(id.timestamp_nanos() > 0);
        assert_eq!(generator.last(), Some(id));
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
                "IDs not strictly increasing at index {i}"
            );
        }
    }

    #[test]
    fn test_rapid_generation() {
        let generator = Generator::new();
        let mut ids = Vec::new();

        // Generate many IDs rapidly (likely within same nanosecond)
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
    fn test_reset() {
        let generator = Generator::new();
        let _ = generator.generate().unwrap();
        assert!(generator.last().is_some());

        generator.reset();
        assert!(generator.last().is_none());
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
        let id = generator.generate().unwrap();
        assert!(id.timestamp_nanos() > 0);
    }

    #[test]
    fn test_last_tracking() {
        let generator = Generator::new();
        assert!(generator.last().is_none());

        let id1 = generator.generate().unwrap();
        assert_eq!(generator.last(), Some(id1));

        let id2 = generator.generate().unwrap();
        assert_eq!(generator.last(), Some(id2));
        assert_ne!(generator.last(), Some(id1));
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
        let original_len = all_ids.len();
        all_ids.dedup();
        assert_eq!(all_ids.len(), original_len);
    }

    #[test]
    fn test_increment_same_timestamp() {
        let generator = Generator::new();

        // Create a NULID with a specific timestamp
        let id1 = Nulid::from_timestamp_nanos(1_000_000_000, 100);

        // Manually set it as last
        *generator.state.lock().unwrap() = Some(id1);

        // Generate with the same timestamp (simulating same nanosecond)
        let id2 = Nulid::from_timestamp_nanos(1_000_000_000, 50); // Lower random

        // Manually create scenario where new_id <= last_id
        *generator.state.lock().unwrap() = Some(id1);

        // The actual generate() call will create a new ID, but if time hasn't advanced
        // it should increment the last one
        let id3 = generator.generate().unwrap();

        // id3 should be greater than id1
        assert!(id3 > id1);
    }
}

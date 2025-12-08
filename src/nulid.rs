//! Core NULID implementation combining timestamp and randomness.
//!
//! This module provides the main NULID type which combines a 68-bit timestamp
//! with 80-bit randomness to create a 148-bit unique identifier.

use crate::{Random, Result, Timestamp};
use core::fmt;

/// A NULID (Nanosecond-Precision Universally Lexicographically Sortable Identifier).
///
/// NULIDs are 148-bit identifiers composed of:
/// - 68-bit timestamp (nanoseconds since UNIX epoch)
/// - 80-bit cryptographically secure randomness
///
/// NULIDs are lexicographically sortable and provide nanosecond precision
/// for time-based ordering, valid until approximately year 10889 AD.
///
/// # Example
///
/// ```rust
/// use nulid::Nulid;
///
/// // Generate a new NULID
/// let nulid = Nulid::new();
/// assert!(nulid.is_ok());
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Nulid {
    timestamp: Timestamp,
    randomness: Random,
}

impl Nulid {
    /// Creates a new NULID with the current timestamp and random data.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The system time cannot be retrieved
    /// - The random number generator fails
    ///
    /// # Example
    ///
    /// ```rust
    /// use nulid::Nulid;
    ///
    /// let nulid = Nulid::new()?;
    /// # Ok::<(), nulid::Error>(())
    /// ```
    pub fn new() -> Result<Self> {
        let timestamp = Timestamp::now()?;
        let randomness = Random::new()?;
        Ok(Self {
            timestamp,
            randomness,
        })
    }

    /// Creates a NULID with a specific timestamp and random data.
    ///
    /// # Example
    ///
    /// ```rust
    /// use nulid::{Nulid, Timestamp, Random};
    ///
    /// let timestamp = Timestamp::from_nanos(1_000_000_000)?;
    /// let randomness = Random::new()?;
    /// let nulid = Nulid::from_parts(timestamp, randomness);
    /// # Ok::<(), nulid::Error>(())
    /// ```
    #[must_use]
    pub const fn from_parts(timestamp: Timestamp, randomness: Random) -> Self {
        Self {
            timestamp,
            randomness,
        }
    }

    /// Creates a NULID from raw bytes (19 bytes: 9 for timestamp, 10 for randomness).
    ///
    /// # Errors
    ///
    /// Returns an error if the timestamp portion exceeds 68 bits.
    ///
    /// # Example
    ///
    /// ```rust
    /// use nulid::Nulid;
    ///
    /// let bytes = [0u8; 19];
    /// let nulid = Nulid::from_bytes(&bytes)?;
    /// # Ok::<(), nulid::Error>(())
    /// ```
    pub fn from_bytes(bytes: &[u8; 19]) -> Result<Self> {
        // First 9 bytes are timestamp
        let mut timestamp_bytes = [0u8; 9];
        timestamp_bytes.copy_from_slice(&bytes[0..9]);
        let timestamp = Timestamp::from_bytes(&timestamp_bytes)?;

        // Next 10 bytes are randomness
        let mut randomness_bytes = [0u8; 10];
        randomness_bytes.copy_from_slice(&bytes[9..19]);
        let randomness = Random::from_bytes(randomness_bytes);

        Ok(Self {
            timestamp,
            randomness,
        })
    }

    /// Converts the NULID to raw bytes (19 bytes).
    ///
    /// # Example
    ///
    /// ```rust
    /// use nulid::Nulid;
    ///
    /// let nulid = Nulid::new()?;
    /// let bytes = nulid.to_bytes();
    /// assert_eq!(bytes.len(), 19);
    /// # Ok::<(), nulid::Error>(())
    /// ```
    #[must_use]
    pub fn to_bytes(&self) -> [u8; 19] {
        let mut bytes = [0u8; 19];
        let timestamp_bytes = self.timestamp.to_bytes();
        let randomness_bytes = self.randomness.as_bytes();

        bytes[0..9].copy_from_slice(&timestamp_bytes);
        bytes[9..19].copy_from_slice(randomness_bytes);

        bytes
    }

    /// Returns the timestamp component of the NULID.
    ///
    /// # Example
    ///
    /// ```rust
    /// use nulid::Nulid;
    ///
    /// let nulid = Nulid::new()?;
    /// let timestamp = nulid.timestamp();
    /// # Ok::<(), nulid::Error>(())
    /// ```
    #[must_use]
    pub const fn timestamp(&self) -> Timestamp {
        self.timestamp
    }

    /// Returns the randomness component of the NULID.
    ///
    /// # Example
    ///
    /// ```rust
    /// use nulid::Nulid;
    ///
    /// let nulid = Nulid::new()?;
    /// let randomness = nulid.randomness();
    /// # Ok::<(), nulid::Error>(())
    /// ```
    #[must_use]
    pub const fn randomness(&self) -> Random {
        self.randomness
    }

    /// Returns the timestamp as nanoseconds since UNIX epoch.
    ///
    /// # Example
    ///
    /// ```rust
    /// use nulid::Nulid;
    ///
    /// let nulid = Nulid::new()?;
    /// let nanos = nulid.timestamp_nanos();
    /// assert!(nanos > 0);
    /// # Ok::<(), nulid::Error>(())
    /// ```
    #[must_use]
    pub const fn timestamp_nanos(&self) -> u128 {
        self.timestamp.as_nanos()
    }

    /// Increments the randomness component for monotonic generation.
    ///
    /// This is used when generating multiple NULIDs within the same nanosecond.
    ///
    /// # Errors
    ///
    /// Returns `Error::RandomnessOverflow` if the randomness is already at maximum.
    ///
    /// # Example
    ///
    /// ```rust
    /// use nulid::Nulid;
    ///
    /// let mut nulid = Nulid::new()?;
    /// nulid.increment_randomness()?;
    /// # Ok::<(), nulid::Error>(())
    /// ```
    pub fn increment_randomness(&mut self) -> Result<()> {
        self.randomness.increment()
    }
}

impl PartialOrd for Nulid {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Nulid {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        // First compare timestamps
        match self.timestamp.cmp(&other.timestamp) {
            core::cmp::Ordering::Equal => {
                // If timestamps are equal, compare randomness
                self.randomness.cmp(&other.randomness)
            }
            other_ordering => other_ordering,
        }
    }
}

impl fmt::Display for Nulid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // For now, display as hex until we implement Base32
        for byte in self.timestamp.to_bytes() {
            write!(f, "{byte:02x}")?;
        }
        for byte in self.randomness.as_bytes() {
            write!(f, "{byte:02x}")?;
        }
        Ok(())
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::Error;

    #[test]
    fn test_new() {
        let nulid = Nulid::new().unwrap();
        assert!(nulid.timestamp_nanos() > 0);
    }

    #[test]
    fn test_from_parts() {
        let timestamp = Timestamp::from_nanos(1_000_000_000).unwrap();
        let randomness = Random::from_bytes([1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
        let nulid = Nulid::from_parts(timestamp, randomness);

        assert_eq!(nulid.timestamp(), timestamp);
        assert_eq!(nulid.randomness(), randomness);
    }

    #[test]
    fn test_bytes_round_trip() {
        let nulid = Nulid::new().unwrap();
        let bytes = nulid.to_bytes();
        let nulid2 = Nulid::from_bytes(&bytes).unwrap();
        assert_eq!(nulid, nulid2);
    }

    #[test]
    fn test_timestamp_extraction() {
        let timestamp = Timestamp::from_nanos(1_234_567_890).unwrap();
        let randomness = Random::from_bytes([1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
        let nulid = Nulid::from_parts(timestamp, randomness);

        assert_eq!(nulid.timestamp_nanos(), 1_234_567_890);
    }

    #[test]
    fn test_ordering() {
        let ts1 = Timestamp::from_nanos(1000).unwrap();
        let ts2 = Timestamp::from_nanos(2000).unwrap();
        let rand = Random::from_bytes([1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);

        let nulid1 = Nulid::from_parts(ts1, rand);
        let nulid2 = Nulid::from_parts(ts2, rand);

        assert!(nulid1 < nulid2);
        assert!(nulid2 > nulid1);
    }

    #[test]
    fn test_ordering_same_timestamp() {
        let ts = Timestamp::from_nanos(1000).unwrap();
        let rand1 = Random::from_bytes([1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
        let rand2 = Random::from_bytes([1, 2, 3, 4, 5, 6, 7, 8, 9, 11]);

        let nulid1 = Nulid::from_parts(ts, rand1);
        let nulid2 = Nulid::from_parts(ts, rand2);

        assert!(nulid1 < nulid2);
    }

    #[test]
    fn test_equality() {
        let timestamp = Timestamp::from_nanos(1_000_000_000).unwrap();
        let randomness = Random::from_bytes([1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);

        let nulid1 = Nulid::from_parts(timestamp, randomness);
        let nulid2 = Nulid::from_parts(timestamp, randomness);

        assert_eq!(nulid1, nulid2);
    }

    #[test]
    fn test_clone_copy() {
        let nulid1 = Nulid::new().unwrap();
        let nulid2 = nulid1;
        assert_eq!(nulid1, nulid2);
    }

    #[test]
    fn test_increment_randomness() {
        let timestamp = Timestamp::from_nanos(1000).unwrap();
        let randomness = Random::from_bytes([0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        let mut nulid = Nulid::from_parts(timestamp, randomness);

        assert!(nulid.increment_randomness().is_ok());
        assert_eq!(nulid.randomness().as_bytes()[9], 1);
    }

    #[test]
    fn test_increment_randomness_overflow() {
        let timestamp = Timestamp::from_nanos(1000).unwrap();
        let randomness = Random::max();
        let mut nulid = Nulid::from_parts(timestamp, randomness);

        let result = nulid.increment_randomness();
        assert_eq!(result, Err(Error::RandomnessOverflow));
    }

    #[test]
    fn test_display() {
        let timestamp = Timestamp::from_nanos(1_000_000_000).unwrap();
        let randomness =
            Random::from_bytes([0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef, 0x00, 0xff]);
        let nulid = Nulid::from_parts(timestamp, randomness);

        let display = format!("{nulid}");
        // Should be hex representation of timestamp + randomness
        assert!(!display.is_empty());
        assert_eq!(display.len(), 38); // 9 bytes timestamp + 10 bytes randomness = 19 bytes * 2 hex chars
    }

    #[test]
    fn test_sorting_multiple() {
        let ts1 = Timestamp::from_nanos(1000).unwrap();
        let ts2 = Timestamp::from_nanos(2000).unwrap();
        let ts3 = Timestamp::from_nanos(3000).unwrap();

        let rand1 = Random::from_bytes([1, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        let rand2 = Random::from_bytes([2, 0, 0, 0, 0, 0, 0, 0, 0, 0]);

        let nulid1 = Nulid::from_parts(ts1, rand1);
        let nulid2 = Nulid::from_parts(ts2, rand2);
        let nulid3 = Nulid::from_parts(ts3, rand1);

        let mut vec = [nulid3, nulid1, nulid2];
        vec.sort();

        assert_eq!(vec[0], nulid1);
        assert_eq!(vec[1], nulid2);
        assert_eq!(vec[2], nulid3);
    }

    #[test]
    fn test_bytes_length() {
        let nulid = Nulid::new().unwrap();
        let bytes = nulid.to_bytes();
        assert_eq!(bytes.len(), 19);
    }

    #[test]
    fn test_components_independent() {
        let ts = Timestamp::from_nanos(12345).unwrap();
        let rand1 = Random::from_bytes([1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
        let rand2 = Random::from_bytes([10, 9, 8, 7, 6, 5, 4, 3, 2, 1]);

        let nulid1 = Nulid::from_parts(ts, rand1);
        let nulid2 = Nulid::from_parts(ts, rand2);

        // Same timestamp but different randomness should be different
        assert_ne!(nulid1, nulid2);
        // But timestamps should be the same
        assert_eq!(nulid1.timestamp(), nulid2.timestamp());
    }
}

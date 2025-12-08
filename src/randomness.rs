//! Randomness handling for NULID with 80-bit cryptographic randomness.
//!
//! This module provides a randomness type that represents 80 bits of
//! cryptographically secure random data, matching ULID's collision resistance.

use crate::{Error, Result};
use core::fmt;

/// An 80-bit (10 byte) cryptographically secure random value.
///
/// Provides the same collision resistance as ULID with 2^80 unique values,
/// which equals approximately 1.21e+24 unique combinations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Random([u8; 10]);

impl Random {
    /// Generates a new cryptographically secure random value.
    ///
    /// Uses the system's cryptographically secure random number generator
    /// via the `getrandom` crate.
    ///
    /// # Errors
    ///
    /// Returns an error if the system's random number generator fails.
    ///
    /// # Example
    ///
    /// ```rust
    /// use nulid::randomness::Random;
    ///
    /// let random = Random::new();
    /// assert!(random.is_ok());
    /// ```
    pub fn new() -> Result<Self> {
        let mut bytes = [0u8; 10];
        getrandom::fill(&mut bytes).map_err(|_| Error::InvalidTimestamp)?;
        Ok(Self(bytes))
    }

    /// Creates a random value from a byte array.
    ///
    /// # Example
    ///
    /// ```rust
    /// use nulid::randomness::Random;
    ///
    /// let bytes = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    /// let random = Random::from_bytes(bytes);
    /// assert_eq!(random.as_bytes(), &bytes);
    /// ```
    #[must_use]
    pub const fn from_bytes(bytes: [u8; 10]) -> Self {
        Self(bytes)
    }

    /// Returns the random value as a byte array.
    ///
    /// # Example
    ///
    /// ```rust
    /// use nulid::randomness::Random;
    ///
    /// let bytes = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    /// let random = Random::from_bytes(bytes);
    /// assert_eq!(random.as_bytes(), &bytes);
    /// ```
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; 10] {
        &self.0
    }

    /// Converts the random value into a byte array.
    ///
    /// # Example
    ///
    /// ```rust
    /// use nulid::randomness::Random;
    ///
    /// let bytes = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    /// let random = Random::from_bytes(bytes);
    /// assert_eq!(random.into_bytes(), bytes);
    /// ```
    #[must_use]
    pub const fn into_bytes(self) -> [u8; 10] {
        self.0
    }

    /// Increments the random value by 1 for monotonic generation.
    ///
    /// This is used when generating multiple NULIDs within the same nanosecond
    /// to ensure monotonic ordering.
    ///
    /// # Errors
    ///
    /// Returns `Error::RandomnessOverflow` if incrementing would overflow
    /// (i.e., the current value is the maximum possible: all bits set to 1).
    ///
    /// # Example
    ///
    /// ```rust
    /// use nulid::randomness::Random;
    ///
    /// let mut random = Random::from_bytes([0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
    /// assert!(random.increment().is_ok());
    /// assert_eq!(random.as_bytes()[9], 1);
    /// ```
    pub fn increment(&mut self) -> Result<()> {
        // Increment from least significant byte to most significant byte
        for byte in self.0.iter_mut().rev() {
            if *byte == 255 {
                *byte = 0;
                // Continue to next byte (carry)
            } else {
                *byte += 1;
                return Ok(());
            }
        }

        // If we got here, all bytes were 255 and overflowed
        Err(Error::RandomnessOverflow)
    }

    /// Checks if this random value is at its maximum (all bits set to 1).
    ///
    /// This is useful for detecting potential overflow before incrementing.
    ///
    /// # Example
    ///
    /// ```rust
    /// use nulid::randomness::Random;
    ///
    /// let max = Random::from_bytes([255, 255, 255, 255, 255, 255, 255, 255, 255, 255]);
    /// assert!(max.is_max());
    ///
    /// let not_max = Random::from_bytes([0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
    /// assert!(!not_max.is_max());
    /// ```
    #[must_use]
    pub fn is_max(&self) -> bool {
        self.0.iter().all(|&byte| byte == 255)
    }

    /// Returns a zero-initialized random value (all bits set to 0).
    ///
    /// This is primarily useful for testing.
    ///
    /// # Example
    ///
    /// ```rust
    /// use nulid::randomness::Random;
    ///
    /// let zero = Random::zero();
    /// assert_eq!(zero.as_bytes(), &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
    /// ```
    #[must_use]
    pub const fn zero() -> Self {
        Self([0u8; 10])
    }

    /// Returns the maximum random value (all bits set to 1).
    ///
    /// # Example
    ///
    /// ```rust
    /// use nulid::randomness::Random;
    ///
    /// let max = Random::max();
    /// assert_eq!(max.as_bytes(), &[255, 255, 255, 255, 255, 255, 255, 255, 255, 255]);
    /// ```
    #[must_use]
    pub const fn max() -> Self {
        Self([255u8; 10])
    }
}

impl Default for Random {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| Self::zero())
    }
}

impl fmt::Display for Random {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Display as hex string
        for byte in &self.0 {
            write!(f, "{byte:02x}")?;
        }
        Ok(())
    }
}

impl fmt::LowerHex for Random {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in &self.0 {
            write!(f, "{byte:02x}")?;
        }
        Ok(())
    }
}

impl fmt::UpperHex for Random {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in &self.0 {
            write!(f, "{byte:02X}")?;
        }
        Ok(())
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let r1 = Random::new().unwrap();
        let r2 = Random::new().unwrap();
        // Two random values should be different (with extremely high probability)
        assert_ne!(r1, r2);
    }

    #[test]
    fn test_from_bytes() {
        let bytes = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let random = Random::from_bytes(bytes);
        assert_eq!(random.as_bytes(), &bytes);
    }

    #[test]
    fn test_into_bytes() {
        let bytes = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let random = Random::from_bytes(bytes);
        assert_eq!(random.into_bytes(), bytes);
    }

    #[test]
    fn test_increment() {
        let mut random = Random::from_bytes([0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        assert!(random.increment().is_ok());
        assert_eq!(random.as_bytes()[9], 1);

        // Test carry
        let mut random = Random::from_bytes([0, 0, 0, 0, 0, 0, 0, 0, 0, 255]);
        assert!(random.increment().is_ok());
        assert_eq!(random.as_bytes()[9], 0);
        assert_eq!(random.as_bytes()[8], 1);
    }

    #[test]
    fn test_increment_multiple_carries() {
        let mut random = Random::from_bytes([0, 0, 0, 0, 0, 0, 0, 0, 255, 255]);
        assert!(random.increment().is_ok());
        assert_eq!(random.as_bytes()[9], 0);
        assert_eq!(random.as_bytes()[8], 0);
        assert_eq!(random.as_bytes()[7], 1);
    }

    #[test]
    fn test_increment_overflow() {
        let mut random = Random::from_bytes([255, 255, 255, 255, 255, 255, 255, 255, 255, 255]);
        let result = random.increment();
        assert_eq!(result, Err(Error::RandomnessOverflow));
    }

    #[test]
    fn test_is_max() {
        let max = Random::from_bytes([255, 255, 255, 255, 255, 255, 255, 255, 255, 255]);
        assert!(max.is_max());

        let not_max = Random::from_bytes([255, 255, 255, 255, 255, 255, 255, 255, 255, 254]);
        assert!(!not_max.is_max());

        let zero = Random::zero();
        assert!(!zero.is_max());
    }

    #[test]
    fn test_zero() {
        let zero = Random::zero();
        assert_eq!(zero.as_bytes(), &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
    }

    #[test]
    fn test_max() {
        let max = Random::max();
        assert_eq!(
            max.as_bytes(),
            &[255, 255, 255, 255, 255, 255, 255, 255, 255, 255]
        );
        assert!(max.is_max());
    }

    #[test]
    fn test_ordering() {
        let r1 = Random::from_bytes([0, 0, 0, 0, 0, 0, 0, 0, 0, 1]);
        let r2 = Random::from_bytes([0, 0, 0, 0, 0, 0, 0, 0, 0, 2]);
        assert!(r1 < r2);
        assert!(r2 > r1);
    }

    #[test]
    fn test_equality() {
        let r1 = Random::from_bytes([1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
        let r2 = Random::from_bytes([1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
        let r3 = Random::from_bytes([1, 2, 3, 4, 5, 6, 7, 8, 9, 11]);
        assert_eq!(r1, r2);
        assert_ne!(r1, r3);
    }

    #[test]
    fn test_clone_copy() {
        let r1 = Random::from_bytes([1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
        let r2 = r1;
        assert_eq!(r1, r2);
    }

    #[test]
    fn test_display() {
        let random =
            Random::from_bytes([0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef, 0x00, 0xff]);
        assert_eq!(random.to_string(), "0123456789abcdef00ff");
    }

    #[test]
    fn test_lower_hex() {
        let random =
            Random::from_bytes([0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef, 0x00, 0xff]);
        assert_eq!(format!("{random:x}"), "0123456789abcdef00ff");
    }

    #[test]
    fn test_upper_hex() {
        let random =
            Random::from_bytes([0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef, 0x00, 0xff]);
        assert_eq!(format!("{random:X}"), "0123456789ABCDEF00FF");
    }

    #[test]
    fn test_default() {
        let random = Random::default();
        // Default should either be a new random value or zero (if RNG fails)
        // We can't test the exact value, but we can test it doesn't panic
        let _ = random.as_bytes();
    }

    #[test]
    fn test_increment_sequence() {
        let mut random = Random::from_bytes([0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);

        for i in 1..=100 {
            assert!(random.increment().is_ok());
            assert_eq!(random.as_bytes()[9], i);
        }
    }

    #[test]
    fn test_all_bytes_used() {
        // Verify we're using all 10 bytes (80 bits)
        let r1 = Random::from_bytes([1, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        let r2 = Random::from_bytes([0, 0, 0, 0, 0, 0, 0, 0, 0, 1]);
        assert_ne!(r1, r2);
        assert!(r1 > r2); // First byte is MSB in comparison
    }
}

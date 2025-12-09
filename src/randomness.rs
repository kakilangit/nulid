//! Cryptographically secure randomness for NULID.
//!
//! This module provides 60-bit secure random values for the NULID randomness component.

use crate::{Error, Result};

/// Generates a cryptographically secure 60-bit random value.
///
/// Uses the system's cryptographically secure random number generator
/// via the `getrandom` crate.
///
/// # Errors
///
/// Returns an error if the system's random number generator fails.
///
/// # Examples
///
/// ```
/// use nulid::randomness::secure_random;
///
/// # fn main() -> nulid::Result<()> {
/// let random = secure_random()?;
/// assert!(random > 0);
/// # Ok(())
/// # }
/// ```
pub fn secure_random() -> Result<u64> {
    let mut bytes = [0u8; 8];
    getrandom::fill(&mut bytes).map_err(|_| Error::RandomError)?;

    let value = u64::from_be_bytes(bytes);

    // Mask to 60 bits (keep only lower 60 bits)
    Ok(value & ((1u64 << 60) - 1))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_secure_random() {
        let r1 = secure_random().unwrap();
        let r2 = secure_random().unwrap();

        // Two random values should be different (with extremely high probability)
        assert_ne!(r1, r2);
    }

    #[test]
    fn test_secure_random_within_bounds() {
        for _ in 0..100 {
            let random = secure_random().unwrap();
            // Must fit in 60 bits
            assert!(random < (1u64 << 60));
        }
    }

    #[test]
    fn test_secure_random_not_zero() {
        // With 60 bits, getting zero is extremely unlikely over 100 iterations
        let mut found_non_zero = false;
        for _ in 0..100 {
            let random = secure_random().unwrap();
            if random != 0 {
                found_non_zero = true;
                break;
            }
        }
        assert!(found_non_zero);
    }

    #[test]
    fn test_secure_random_distribution() {
        // Test that we're using multiple bits
        let mut values = Vec::new();
        for _ in 0..10 {
            values.push(secure_random().unwrap());
        }

        // All values should be unique (with very high probability)
        values.sort_unstable();
        values.dedup();
        assert_eq!(values.len(), 10);
    }
}

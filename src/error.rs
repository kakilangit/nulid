//! Error types for NULID operations.

use core::fmt;

/// Errors that can occur during NULID operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// The provided string has an invalid length.
    InvalidLength {
        /// The expected length.
        expected: usize,
        /// The actual length found.
        found: usize,
    },

    /// The string contains an invalid character.
    InvalidCharacter {
        /// The invalid character.
        character: char,
        /// The position of the character.
        position: usize,
    },

    /// The timestamp value is invalid or out of range.
    InvalidTimestamp,

    /// The timestamp exceeds the maximum allowed value (2^68 - 1).
    TimestampOverflow,

    /// The randomness component has overflowed during monotonic increment.
    RandomnessOverflow,

    /// General decoding error with a description.
    DecodeError(String),

    /// The data buffer is too small.
    BufferTooSmall,

    /// The internal mutex was poisoned (another thread panicked while holding the lock).
    MutexPoisoned,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidLength { expected, found } => {
                write!(
                    f,
                    "Invalid length: expected {expected} characters, found {found}"
                )
            }
            Self::InvalidCharacter {
                character,
                position,
            } => {
                write!(f, "Invalid character '{character}' at position {position}")
            }
            Self::InvalidTimestamp => {
                write!(f, "Invalid timestamp value")
            }
            Self::TimestampOverflow => {
                write!(f, "Timestamp overflow: value exceeds 68-bit maximum")
            }
            Self::RandomnessOverflow => {
                write!(f, "Randomness overflow: cannot increment further")
            }
            Self::DecodeError(msg) => {
                write!(f, "Decode error: {msg}")
            }
            Self::BufferTooSmall => {
                write!(f, "Buffer too small for NULID data")
            }
            Self::MutexPoisoned => {
                write!(f, "Internal mutex poisoned (thread panic)")
            }
        }
    }
}

impl std::error::Error for Error {}

/// A specialized `Result` type for NULID operations.
pub type Result<T> = core::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = Error::InvalidLength {
            expected: 30,
            found: 20,
        };
        assert_eq!(
            err.to_string(),
            "Invalid length: expected 30 characters, found 20"
        );

        let err = Error::InvalidCharacter {
            character: 'I',
            position: 5,
        };
        assert_eq!(err.to_string(), "Invalid character 'I' at position 5");

        let err = Error::TimestampOverflow;
        assert_eq!(
            err.to_string(),
            "Timestamp overflow: value exceeds 68-bit maximum"
        );

        let err = Error::RandomnessOverflow;
        assert_eq!(
            err.to_string(),
            "Randomness overflow: cannot increment further"
        );
    }

    #[test]
    fn test_error_equality() {
        let err1 = Error::InvalidTimestamp;
        let err2 = Error::InvalidTimestamp;
        assert_eq!(err1, err2);

        let err3 = Error::TimestampOverflow;
        assert_ne!(err1, err3);
    }

    #[test]
    fn test_error_clone() {
        let err = Error::InvalidCharacter {
            character: 'X',
            position: 10,
        };
        let cloned = err.clone();
        assert_eq!(err, cloned);
    }
}

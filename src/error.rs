//! Error types for NULID operations.

use std::fmt;

/// Errors that can occur during NULID operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// Failed to generate random data.
    RandomError,

    /// Invalid character in Base32 string.
    InvalidChar(char, usize),

    /// Invalid string length for NULID encoding.
    InvalidLength {
        /// Expected length.
        expected: usize,
        /// Actual length found.
        found: usize,
    },

    /// System time is before Unix epoch.
    SystemTimeError,

    /// Overflow occurred during NULID increment.
    Overflow,

    /// Mutex was poisoned (another thread panicked while holding the lock).
    MutexPoisoned,

    /// UTF-8 encoding error (should never occur with valid ALPHABET).
    EncodingError,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RandomError => write!(f, "Failed to generate random data"),
            Self::InvalidChar(ch, pos) => {
                write!(f, "Invalid character '{ch}' at position {pos}")
            }
            Self::InvalidLength { expected, found } => {
                write!(
                    f,
                    "Invalid length: expected {expected} characters, found {found}"
                )
            }
            Self::SystemTimeError => write!(f, "System time is before Unix epoch"),
            Self::Overflow => write!(f, "Overflow occurred during NULID increment"),
            Self::MutexPoisoned => write!(f, "Mutex poisoned (thread panic)"),
            Self::EncodingError => write!(f, "UTF-8 encoding error"),
        }
    }
}

impl std::error::Error for Error {}

/// A specialized `Result` type for NULID operations.
pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        assert_eq!(
            Error::RandomError.to_string(),
            "Failed to generate random data"
        );

        assert_eq!(
            Error::InvalidChar('I', 5).to_string(),
            "Invalid character 'I' at position 5"
        );

        assert_eq!(
            Error::InvalidLength {
                expected: 26,
                found: 20
            }
            .to_string(),
            "Invalid length: expected 26 characters, found 20"
        );

        assert_eq!(
            Error::SystemTimeError.to_string(),
            "System time is before Unix epoch"
        );

        assert_eq!(
            Error::Overflow.to_string(),
            "Overflow occurred during NULID increment"
        );

        assert_eq!(
            Error::MutexPoisoned.to_string(),
            "Mutex poisoned (thread panic)"
        );
    }

    #[test]
    fn test_error_equality() {
        assert_eq!(Error::RandomError, Error::RandomError);
        assert_ne!(Error::RandomError, Error::SystemTimeError);

        assert_eq!(Error::InvalidChar('X', 1), Error::InvalidChar('X', 1));
        assert_ne!(Error::InvalidChar('X', 1), Error::InvalidChar('Y', 1));
        assert_ne!(Error::InvalidChar('X', 1), Error::InvalidChar('X', 2));
    }

    #[test]
    fn test_error_clone() {
        let err = Error::InvalidChar('A', 10);
        #[allow(clippy::redundant_clone)]
        let cloned = err.clone();
        assert_eq!(err, cloned);
    }

    #[test]
    fn test_error_debug() {
        let err = Error::InvalidLength {
            expected: 26,
            found: 10,
        };
        let debug_str = format!("{err:?}");
        assert!(debug_str.contains("InvalidLength"));
    }
}

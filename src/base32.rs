//! Base32 encoding and decoding using Crockford's alphabet.
//!
//! This module provides functions to encode and decode NULID values using
//! Crockford's Base32 alphabet, which is designed to be URL-safe and
//! human-readable while avoiding ambiguous characters.
//!
//! # Alphabet
//!
//! The Crockford Base32 alphabet uses: `0123456789ABCDEFGHJKMNPQRSTVWXYZ`
//! (excluding I, L, O, U to avoid confusion with 1, 1, 0, V respectively)
//!
//! # Encoding Format
//!
//! A NULID is encoded as a 30-character string:
//! - First 14 characters: 68-bit timestamp (with 2 bits padding)
//! - Last 16 characters: 80-bit randomness (exact fit)
//!
//! The encoding preserves lexicographic ordering, making NULID strings
//! naturally sortable by time.
//!
//! # Timestamp Precision
//!
//! The timestamp uses the full 70 bits available in 14 Base32 characters,
//! providing nanosecond precision until approximately year 45526 AD.

use crate::error::{Error, Result};

/// Crockford's Base32 alphabet (32 characters, 5 bits each)
const ALPHABET: &[u8; 32] = b"0123456789ABCDEFGHJKMNPQRSTVWXYZ";

/// Length of a NULID string representation
pub const NULID_STRING_LENGTH: usize = 30;

/// Number of characters used to encode the timestamp
const TIMESTAMP_CHARS: usize = 14;

/// Encodes a NULID (70-bit timestamp + 80-bit randomness) into a 30-character Base32 string.
///
/// # Arguments
///
/// * `timestamp_bits` - The 70-bit timestamp as a u128
/// * `randomness` - The 80-bit randomness as a 10-byte array
///
/// # Returns
///
/// A 30-character string using Crockford's Base32 alphabet
///
/// # Example
///
/// ```
/// use nulid::base32::encode;
///
/// let timestamp = 0x1234567890ABCD;
/// let randomness = [0x01, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD, 0xEF, 0x01, 0x23];
/// let encoded = encode(timestamp, &randomness);
/// assert_eq!(encoded.len(), 30);
/// ```
#[must_use]
pub fn encode(timestamp_bits: u128, randomness: &[u8; 10]) -> String {
    let mut result = String::with_capacity(NULID_STRING_LENGTH);

    // Encode timestamp (70 bits -> 14 characters)
    // We need to extract 5 bits at a time from the 70-bit timestamp
    encode_timestamp(&mut result, timestamp_bits);

    // Encode randomness (80 bits -> 16 characters)
    encode_randomness(&mut result, randomness);

    result
}

/// Encodes the timestamp portion (70 bits -> 14 characters)
fn encode_timestamp(result: &mut String, timestamp_bits: u128) {
    // Encode 14 characters (70 bits exactly: 14 × 5 = 70)
    // Start from the most significant bits
    for i in (0..TIMESTAMP_CHARS).rev() {
        let shift = i * 5;
        let index = ((timestamp_bits >> shift) & 0x1F) as usize;
        result.push(ALPHABET[index] as char);
    }
}

/// Encodes the randomness portion (80 bits -> 16 characters)
fn encode_randomness(result: &mut String, randomness: &[u8; 10]) {
    // Convert 10 bytes into 16 base32 characters
    // Each group of 5 bytes (40 bits) becomes 8 characters

    // Process first 5 bytes (40 bits -> 8 characters)
    encode_5_bytes(result, &randomness[0..5]);

    // Process last 5 bytes (40 bits -> 8 characters)
    encode_5_bytes(result, &randomness[5..10]);
}

/// Encodes 5 bytes (40 bits) into 8 Base32 characters
fn encode_5_bytes(result: &mut String, bytes: &[u8]) {
    // Combine 5 bytes into a u64 for easier bit manipulation
    let value = (u64::from(bytes[0]) << 32)
        | (u64::from(bytes[1]) << 24)
        | (u64::from(bytes[2]) << 16)
        | (u64::from(bytes[3]) << 8)
        | u64::from(bytes[4]);

    // Extract 8 groups of 5 bits each
    for i in (0..8).rev() {
        let shift = i * 5;
        let index = ((value >> shift) & 0x1F) as usize;
        result.push(ALPHABET[index] as char);
    }
}

/// Decodes a 30-character Base32 string into a NULID (70-bit timestamp + 80-bit randomness).
///
/// # Arguments
///
/// * `s` - A 30-character string using Crockford's Base32 alphabet
///
/// # Returns
///
/// A tuple of (`timestamp_bits`, randomness) or an error if decoding fails
///
/// # Errors
///
/// Returns `Error::InvalidLength` if the string is not 30 characters.
/// Returns `Error::InvalidCharacter` if the string contains invalid characters.
/// Returns `Error::TimestampOverflow` if the timestamp exceeds 70 bits.
///
/// # Example
///
/// ```
/// use nulid::base32::{encode, decode};
///
/// let timestamp = 0x1234567890ABCD;
/// let randomness = [0x01, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD, 0xEF, 0x01, 0x23];
/// let encoded = encode(timestamp, &randomness);
/// let (decoded_ts, decoded_rand) = decode(&encoded).unwrap();
/// assert_eq!(decoded_ts, timestamp);
/// assert_eq!(decoded_rand, randomness);
/// ```
pub fn decode(s: &str) -> Result<(u128, [u8; 10])> {
    // Validate length
    if s.len() != NULID_STRING_LENGTH {
        return Err(Error::InvalidLength {
            expected: NULID_STRING_LENGTH,
            found: s.len(),
        });
    }

    // Decode timestamp (14 characters -> 70 bits)
    let timestamp_str = &s[..TIMESTAMP_CHARS];
    let timestamp_bits = decode_timestamp(timestamp_str)?;

    // Validate that timestamp doesn't exceed 70 bits (2^70 - 1)
    if timestamp_bits > 0x3F_FFFF_FFFF_FFFF_FFFF {
        return Err(Error::TimestampOverflow);
    }

    // Decode randomness (16 characters -> 80 bits)
    let randomness_str = &s[TIMESTAMP_CHARS..];
    let randomness = decode_randomness(randomness_str)?;

    Ok((timestamp_bits, randomness))
}

/// Decodes the timestamp portion (14 characters -> 70 bits)
fn decode_timestamp(s: &str) -> Result<u128> {
    let mut result: u128 = 0;

    for (i, ch) in s.chars().enumerate() {
        let value = decode_char(ch, i)?;
        result = (result << 5) | u128::from(value);
    }

    Ok(result)
}

/// Decodes the randomness portion (16 characters -> 80 bits)
fn decode_randomness(s: &str) -> Result<[u8; 10]> {
    let mut result = [0u8; 10];

    // Decode first 8 characters -> first 5 bytes
    decode_8_chars_to_5_bytes(&s[..8], &mut result[0..5])?;

    // Decode last 8 characters -> last 5 bytes
    decode_8_chars_to_5_bytes(&s[8..], &mut result[5..10])?;

    Ok(result)
}

/// Decodes 8 Base32 characters into 5 bytes (40 bits)
fn decode_8_chars_to_5_bytes(s: &str, output: &mut [u8]) -> Result<()> {
    let mut value: u64 = 0;

    for (i, ch) in s.chars().enumerate() {
        let decoded = decode_char(ch, i)?;
        value = (value << 5) | u64::from(decoded);
    }

    // Extract 5 bytes from the 40-bit value
    output[0] = ((value >> 32) & 0xFF) as u8;
    output[1] = ((value >> 24) & 0xFF) as u8;
    output[2] = ((value >> 16) & 0xFF) as u8;
    output[3] = ((value >> 8) & 0xFF) as u8;
    output[4] = (value & 0xFF) as u8;

    Ok(())
}

/// Decodes a single Base32 character to its 5-bit value
const fn decode_char(ch: char, position: usize) -> Result<u8> {
    let value = match ch {
        '0' => 0,
        '1' => 1,
        '2' => 2,
        '3' => 3,
        '4' => 4,
        '5' => 5,
        '6' => 6,
        '7' => 7,
        '8' => 8,
        '9' => 9,
        'A' | 'a' => 10,
        'B' | 'b' => 11,
        'C' | 'c' => 12,
        'D' | 'd' => 13,
        'E' | 'e' => 14,
        'F' | 'f' => 15,
        'G' | 'g' => 16,
        'H' | 'h' => 17,
        'J' | 'j' => 18,
        'K' | 'k' => 19,
        'M' | 'm' => 20,
        'N' | 'n' => 21,
        'P' | 'p' => 22,
        'Q' | 'q' => 23,
        'R' | 'r' => 24,
        'S' | 's' => 25,
        'T' | 't' => 26,
        'V' | 'v' => 27,
        'W' | 'w' => 28,
        'X' | 'x' => 29,
        'Y' | 'y' => 30,
        'Z' | 'z' => 31,
        _ => {
            return Err(Error::InvalidCharacter {
                character: ch,
                position,
            });
        }
    };

    Ok(value)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_decode_zero() {
        let timestamp = 0u128;
        let randomness = [0u8; 10];

        let encoded = encode(timestamp, &randomness);
        assert_eq!(encoded.len(), NULID_STRING_LENGTH);
        assert_eq!(encoded, "000000000000000000000000000000");

        let (decoded_ts, decoded_rand) = decode(&encoded).unwrap();
        assert_eq!(decoded_ts, timestamp);
        assert_eq!(decoded_rand, randomness);
    }

    #[test]
    fn test_encode_decode_max_timestamp() {
        let timestamp = 0x3F_FFFF_FFFF_FFFF_FFFF; // Max 70-bit value (2^70 - 1)
        let randomness = [0u8; 10];

        let encoded = encode(timestamp, &randomness);
        let (decoded_ts, decoded_rand) = decode(&encoded).unwrap();
        assert_eq!(decoded_ts, timestamp);
        assert_eq!(decoded_rand, randomness);
    }

    #[test]
    fn test_encode_decode_max_randomness() {
        let timestamp = 0u128;
        let randomness = [0xFF; 10];

        let encoded = encode(timestamp, &randomness);
        let (decoded_ts, decoded_rand) = decode(&encoded).unwrap();
        assert_eq!(decoded_ts, timestamp);
        assert_eq!(decoded_rand, randomness);
    }

    #[test]
    fn test_encode_decode_mixed_values() {
        let timestamp = 0x0012_3456_7890_ABCD;
        let randomness = [0x01, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD, 0xEF, 0x01, 0x23];

        let encoded = encode(timestamp, &randomness);
        let (decoded_ts, decoded_rand) = decode(&encoded).unwrap();
        assert_eq!(decoded_ts, timestamp);
        assert_eq!(decoded_rand, randomness);
    }

    #[test]
    fn test_decode_case_insensitive() {
        let timestamp = 0x0012_3456_7890_ABCD;
        let randomness = [0x01, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD, 0xEF, 0x01, 0x23];

        let encoded = encode(timestamp, &randomness);
        let lowercase = encoded.to_lowercase();

        let (decoded_ts, decoded_rand) = decode(&lowercase).unwrap();
        assert_eq!(decoded_ts, timestamp);
        assert_eq!(decoded_rand, randomness);
    }

    #[test]
    fn test_decode_invalid_length_short() {
        let result = decode("123");
        assert!(matches!(result, Err(Error::InvalidLength { .. })));
    }

    #[test]
    fn test_decode_invalid_length_long() {
        let result = decode("0123456789ABCDEFGHJKMNPQRSTVWXYZ");
        assert!(matches!(result, Err(Error::InvalidLength { .. })));
    }

    #[test]
    fn test_decode_invalid_character() {
        let invalid = "00000000000000000000000000000I"; // 'I' is not in Crockford alphabet
        let result = decode(invalid);
        assert!(matches!(result, Err(Error::InvalidCharacter { .. })));
    }

    #[test]
    fn test_decode_invalid_character_o() {
        let invalid = "00000000000000000000000000000O"; // 'O' is not in Crockford alphabet
        let result = decode(invalid);
        assert!(matches!(result, Err(Error::InvalidCharacter { .. })));
    }

    #[test]
    fn test_lexicographic_ordering() {
        // Earlier timestamps should produce lexicographically smaller strings
        let ts1 = 1000u128;
        let ts2 = 2000u128;
        let randomness = [0u8; 10];

        let encoded1 = encode(ts1, &randomness);
        let encoded2 = encode(ts2, &randomness);

        assert!(encoded1 < encoded2);
    }

    #[test]
    fn test_alphabet_characters() {
        // Verify the alphabet contains the expected characters
        let alphabet_str = std::str::from_utf8(ALPHABET).unwrap();
        assert_eq!(alphabet_str, "0123456789ABCDEFGHJKMNPQRSTVWXYZ");
        assert_eq!(ALPHABET.len(), 32);
    }

    #[test]
    fn test_encode_all_alphabet_chars() {
        // Ensure all alphabet characters can appear in encoded output
        let timestamp = u128::MAX >> (128 - 68);
        let randomness = [0xFF; 10];
        let encoded = encode(timestamp, &randomness);

        // The encoded string should only contain valid Crockford characters
        for ch in encoded.chars() {
            assert!(ALPHABET.contains(&(ch as u8)));
        }
    }

    #[test]
    fn test_decode_all_valid_chars() {
        // Test that all valid Crockford characters can be decoded
        let valid_chars = "0123456789ABCDEFGHJKMNPQRSTVWXYZ";

        for (i, ch) in valid_chars.chars().enumerate() {
            let result = decode_char(ch, 0);
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), u8::try_from(i).unwrap());
        }
    }

    #[test]
    fn test_roundtrip_various_values() {
        let test_cases = vec![
            (0u128, [0u8; 10]),
            (1u128, [1u8, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
            (0xFFFF, [0xFF, 0xFF, 0, 0, 0, 0, 0, 0, 0, 0]),
            (
                0x0123_4567_89AB_CDEF,
                [0xFE, 0xDC, 0xBA, 0x98, 0x76, 0x54, 0x32, 0x10, 0xFF, 0x00],
            ),
        ];

        for (timestamp, randomness) in test_cases {
            let encoded = encode(timestamp, &randomness);
            let (decoded_ts, decoded_rand) = decode(&encoded).unwrap();
            assert_eq!(
                decoded_ts, timestamp,
                "Timestamp mismatch for {timestamp:X}"
            );
            assert_eq!(
                decoded_rand, randomness,
                "Randomness mismatch for {timestamp:X}"
            );
        }
    }
}

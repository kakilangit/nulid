//! Base32 encoding and decoding for 128-bit NULID using Crockford's alphabet.
//!
//! This module provides functions to encode and decode 128-bit NULID values using
//! Crockford's Base32 alphabet, which is designed to be URL-safe and human-readable
//! while avoiding ambiguous characters.
//!
//! # Alphabet
//!
//! The Crockford Base32 alphabet uses: `0123456789ABCDEFGHJKMNPQRSTVWXYZ`
//! (excluding I, L, O, U to avoid confusion with 1, 1, 0, V respectively)
//!
//! # Encoding Format
//!
//! A 128-bit NULID is encoded as a 26-character string:
//! - 128 bits / 5 bits per character = 25.6 characters → 26 characters (130 bits capacity)
//! - 2 bits are unused (padding in the most significant position)
//!
//! The encoding preserves lexicographic ordering, making NULID strings naturally
//! sortable by their timestamp component.

use crate::{Error, Result};

/// Crockford's Base32 alphabet (32 characters, 5 bits each)
const ALPHABET: &[u8; 32] = b"0123456789ABCDEFGHJKMNPQRSTVWXYZ";

/// Length of a NULID string representation (26 characters)
pub const NULID_STRING_LENGTH: usize = 26;

/// Lookup table for decoding Base32 characters
/// Invalid characters are marked with 0xFF
const DECODE_TABLE: [u8; 256] = {
    let mut table = [0xFF; 256];
    table[b'0' as usize] = 0;
    table[b'1' as usize] = 1;
    table[b'2' as usize] = 2;
    table[b'3' as usize] = 3;
    table[b'4' as usize] = 4;
    table[b'5' as usize] = 5;
    table[b'6' as usize] = 6;
    table[b'7' as usize] = 7;
    table[b'8' as usize] = 8;
    table[b'9' as usize] = 9;
    table[b'A' as usize] = 10;
    table[b'a' as usize] = 10;
    table[b'B' as usize] = 11;
    table[b'b' as usize] = 11;
    table[b'C' as usize] = 12;
    table[b'c' as usize] = 12;
    table[b'D' as usize] = 13;
    table[b'd' as usize] = 13;
    table[b'E' as usize] = 14;
    table[b'e' as usize] = 14;
    table[b'F' as usize] = 15;
    table[b'f' as usize] = 15;
    table[b'G' as usize] = 16;
    table[b'g' as usize] = 16;
    table[b'H' as usize] = 17;
    table[b'h' as usize] = 17;
    table[b'J' as usize] = 18;
    table[b'j' as usize] = 18;
    table[b'K' as usize] = 19;
    table[b'k' as usize] = 19;
    table[b'M' as usize] = 20;
    table[b'm' as usize] = 20;
    table[b'N' as usize] = 21;
    table[b'n' as usize] = 21;
    table[b'P' as usize] = 22;
    table[b'p' as usize] = 22;
    table[b'Q' as usize] = 23;
    table[b'q' as usize] = 23;
    table[b'R' as usize] = 24;
    table[b'r' as usize] = 24;
    table[b'S' as usize] = 25;
    table[b's' as usize] = 25;
    table[b'T' as usize] = 26;
    table[b't' as usize] = 26;
    table[b'V' as usize] = 27;
    table[b'v' as usize] = 27;
    table[b'W' as usize] = 28;
    table[b'w' as usize] = 28;
    table[b'X' as usize] = 29;
    table[b'x' as usize] = 29;
    table[b'Y' as usize] = 30;
    table[b'y' as usize] = 30;
    table[b'Z' as usize] = 31;
    table[b'z' as usize] = 31;
    table
};

/// Encodes a 128-bit value into a 26-character Base32 string.
///
/// The encoding is written directly into the provided buffer for zero-allocation encoding.
///
/// # Arguments
///
/// * `value` - The 128-bit value to encode
/// * `buf` - A 26-byte buffer to write the encoded string into
///
/// # Returns
///
/// A string slice pointing to the encoded data in the buffer
///
/// # Errors
///
/// Returns `Error::EncodingError` if UTF-8 validation fails. In practice, this should
/// never occur since the ALPHABET contains only valid ASCII characters.
///
/// # Examples
///
/// ```
/// use nulid::base32::encode_u128;
///
/// # fn main() -> nulid::Result<()> {
/// let value = 0x0123_4567_89AB_CDEF_FEDC_BA98_7654_3210u128;
/// let mut buf = [0u8; 26];
/// let s = encode_u128(value, &mut buf)?;
/// assert_eq!(s.len(), 26);
/// # Ok(())
/// # }
/// ```
#[inline]
pub fn encode_u128(mut value: u128, buf: &mut [u8; 26]) -> Result<&str> {
    buf[25] = ALPHABET[(value & 0x1F) as usize];
    value >>= 5;
    buf[24] = ALPHABET[(value & 0x1F) as usize];
    value >>= 5;
    buf[23] = ALPHABET[(value & 0x1F) as usize];
    value >>= 5;
    buf[22] = ALPHABET[(value & 0x1F) as usize];
    value >>= 5;
    buf[21] = ALPHABET[(value & 0x1F) as usize];
    value >>= 5;
    buf[20] = ALPHABET[(value & 0x1F) as usize];
    value >>= 5;
    buf[19] = ALPHABET[(value & 0x1F) as usize];
    value >>= 5;
    buf[18] = ALPHABET[(value & 0x1F) as usize];
    value >>= 5;
    buf[17] = ALPHABET[(value & 0x1F) as usize];
    value >>= 5;
    buf[16] = ALPHABET[(value & 0x1F) as usize];
    value >>= 5;
    buf[15] = ALPHABET[(value & 0x1F) as usize];
    value >>= 5;
    buf[14] = ALPHABET[(value & 0x1F) as usize];
    value >>= 5;
    buf[13] = ALPHABET[(value & 0x1F) as usize];
    value >>= 5;
    buf[12] = ALPHABET[(value & 0x1F) as usize];
    value >>= 5;
    buf[11] = ALPHABET[(value & 0x1F) as usize];
    value >>= 5;
    buf[10] = ALPHABET[(value & 0x1F) as usize];
    value >>= 5;
    buf[9] = ALPHABET[(value & 0x1F) as usize];
    value >>= 5;
    buf[8] = ALPHABET[(value & 0x1F) as usize];
    value >>= 5;
    buf[7] = ALPHABET[(value & 0x1F) as usize];
    value >>= 5;
    buf[6] = ALPHABET[(value & 0x1F) as usize];
    value >>= 5;
    buf[5] = ALPHABET[(value & 0x1F) as usize];
    value >>= 5;
    buf[4] = ALPHABET[(value & 0x1F) as usize];
    value >>= 5;
    buf[3] = ALPHABET[(value & 0x1F) as usize];
    value >>= 5;
    buf[2] = ALPHABET[(value & 0x1F) as usize];
    value >>= 5;
    buf[1] = ALPHABET[(value & 0x1F) as usize];
    value >>= 5;
    buf[0] = ALPHABET[(value & 0x1F) as usize];

    // Safety: ALPHABET contains only ASCII characters (0-9, A-Z), so this conversion
    // should never fail. We include a debug assertion to catch any potential issues
    // during development.
    core::str::from_utf8(buf).map_err(|utf8_err| {
        // This should be unreachable since ALPHABET is guaranteed to be valid ASCII
        debug_assert!(
            false,
            "UTF-8 conversion failed unexpectedly. This indicates a bug in the encoding logic. Error: {utf8_err}"
        );
        Error::EncodingError
    })
}

/// Decodes a 26-character Base32 string into a 128-bit value.
///
/// # Arguments
///
/// * `s` - A 26-character string using Crockford's Base32 alphabet (case-insensitive)
///
/// # Returns
///
/// The decoded 128-bit value
///
/// # Errors
///
/// Returns `Error::InvalidLength` if the string is not 26 characters.
/// Returns `Error::InvalidChar` if the string contains invalid characters.
///
/// # Examples
///
/// ```
/// use nulid::base32::{encode_u128, decode_u128};
///
/// # fn main() -> nulid::Result<()> {
/// let value = 0x0123_4567_89AB_CDEF_FEDC_BA98_7654_3210u128;
/// let mut buf = [0u8; 26];
/// let encoded = encode_u128(value, &mut buf)?;
/// let decoded = decode_u128(encoded)?;
/// assert_eq!(decoded, value);
/// # Ok(())
/// # }
/// ```
#[inline]
pub fn decode_u128(s: &str) -> Result<u128> {
    // Validate length
    if s.len() != NULID_STRING_LENGTH {
        return Err(Error::InvalidLength {
            expected: NULID_STRING_LENGTH,
            found: s.len(),
        });
    }

    let mut result: u128 = 0;

    for (i, byte) in s.bytes().enumerate() {
        let value = DECODE_TABLE[byte as usize];
        if value == 0xFF {
            return Err(Error::InvalidChar(byte as char, i));
        }
        result = (result << 5) | u128::from(value);
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_decode_zero() {
        let value = 0u128;
        let mut buf = [0u8; 26];
        let encoded = encode_u128(value, &mut buf).unwrap();

        assert_eq!(encoded.len(), NULID_STRING_LENGTH);
        assert_eq!(encoded, "00000000000000000000000000");

        let decoded = decode_u128(encoded).unwrap();
        assert_eq!(decoded, value);
    }

    #[test]
    fn test_encode_decode_max() {
        let value = u128::MAX;
        let mut buf = [0u8; 26];
        let encoded = encode_u128(value, &mut buf).unwrap();

        let decoded = decode_u128(encoded).unwrap();
        assert_eq!(decoded, value);
    }

    #[test]
    fn test_encode_decode_various() {
        let test_cases = vec![
            0u128,
            1u128,
            255u128,
            65535u128,
            0xFFFF_FFFF_u128,
            0xFFFF_FFFF_FFFF_FFFF_u128,
            0x0123_4567_89AB_CDEF_FEDC_BA98_7654_3210_u128,
            u128::MAX,
        ];

        for value in test_cases {
            let mut buf = [0u8; 26];
            let encoded = encode_u128(value, &mut buf).unwrap();
            let decoded = decode_u128(encoded).unwrap();
            assert_eq!(decoded, value, "Mismatch for {value:X}");
        }
    }

    #[test]
    fn test_decode_case_insensitive() {
        let value = 0x0123_4567_89AB_CDEF_FEDC_BA98_7654_3210_u128;
        let mut buf = [0u8; 26];
        let encoded = encode_u128(value, &mut buf).unwrap();

        let lowercase = encoded.to_lowercase();
        let decoded = decode_u128(&lowercase).unwrap();
        assert_eq!(decoded, value);
    }

    #[test]
    fn test_decode_invalid_length_short() {
        let result = decode_u128("123");
        assert!(matches!(result, Err(Error::InvalidLength { .. })));
    }

    #[test]
    fn test_decode_invalid_length_long() {
        let result = decode_u128("012345678901234567890123456");
        assert!(matches!(result, Err(Error::InvalidLength { .. })));
    }

    #[test]
    fn test_decode_invalid_char_i() {
        let invalid = "0000000000000000000000000I"; // 'I' is not in Crockford alphabet
        let result = decode_u128(invalid);
        assert!(matches!(result, Err(Error::InvalidChar('I', 25))));
    }

    #[test]
    fn test_decode_invalid_char_o() {
        let invalid = "0000000000000000000000000O"; // 'O' is not in Crockford alphabet
        let result = decode_u128(invalid);
        assert!(matches!(result, Err(Error::InvalidChar('O', 25))));
    }

    #[test]
    fn test_decode_invalid_char_l() {
        let invalid = "0000000000000000000000000L"; // 'L' is not in Crockford alphabet
        let result = decode_u128(invalid);
        assert!(matches!(result, Err(Error::InvalidChar('L', 25))));
    }

    #[test]
    fn test_decode_invalid_char_u() {
        let invalid = "0000000000000000000000000U"; // 'U' is not in Crockford alphabet
        let result = decode_u128(invalid);
        assert!(matches!(result, Err(Error::InvalidChar('U', 25))));
    }

    #[test]
    fn test_lexicographic_ordering() {
        // Earlier values should produce lexicographically smaller strings
        let val1 = 1000u128;
        let val2 = 2000u128;

        let mut buf1 = [0u8; 26];
        let mut buf2 = [0u8; 26];
        let encoded1 = encode_u128(val1, &mut buf1).unwrap();
        let encoded2 = encode_u128(val2, &mut buf2).unwrap();

        assert!(encoded1 < encoded2);
    }

    #[test]
    fn test_alphabet_valid() {
        // Verify that ALPHABET contains only valid ASCII/UTF-8 characters
        // This test ensures the debug_assert in encode_u128 should never trigger
        for &byte in ALPHABET {
            assert!(
                byte.is_ascii(),
                "ALPHABET contains non-ASCII byte: {byte:#x}"
            );
        }

        // Verify the entire alphabet can be converted to a valid UTF-8 string
        let alphabet_str = core::str::from_utf8(ALPHABET).unwrap();
        assert_eq!(alphabet_str, "0123456789ABCDEFGHJKMNPQRSTVWXYZ");
        assert_eq!(ALPHABET.len(), 32);

        // Verify no ambiguous characters
        assert!(!alphabet_str.contains('I'));
        assert!(!alphabet_str.contains('L'));
        assert!(!alphabet_str.contains('O'));
        assert!(!alphabet_str.contains('U'));
    }

    #[test]
    fn test_encode_only_valid_chars() {
        let value = u128::MAX;
        let mut buf = [0u8; 26];
        let encoded = encode_u128(value, &mut buf).unwrap();

        for ch in encoded.chars() {
            assert!(
                ALPHABET.contains(&(ch as u8)),
                "Invalid character '{ch}' in encoded output"
            );
        }
    }

    #[test]
    fn test_decode_all_valid_chars() {
        let valid_chars = "0123456789ABCDEFGHJKMNPQRSTVWXYZ";

        for ch in valid_chars.chars() {
            let byte = ch as u8;
            let value = DECODE_TABLE[byte as usize];
            assert_ne!(value, 0xFF, "Character '{ch}' not in decode table");
            assert!(value < 32, "Decoded value for '{ch}' out of range");
        }
    }

    #[test]
    fn test_roundtrip_sequential() {
        for i in 0..100 {
            let value = u128::try_from(i).unwrap();
            let mut buf = [0u8; 26];
            let encoded = encode_u128(value, &mut buf).unwrap();
            let decoded = decode_u128(encoded).unwrap();
            assert_eq!(decoded, value);
        }
    }

    #[test]
    fn test_ordering_preserved() {
        // Test that numeric ordering is preserved in string encoding
        let values = vec![0u128, 100, 1000, 10000, 100_000, 1_000_000];
        let mut encoded_strs = Vec::new();

        for &value in &values {
            let mut buf = [0u8; 26];
            let encoded = encode_u128(value, &mut buf).unwrap();
            encoded_strs.push(encoded.to_string());
        }

        // Check that string ordering matches numeric ordering
        for i in 1..encoded_strs.len() {
            assert!(
                encoded_strs[i - 1] < encoded_strs[i],
                "{} should be < {}",
                encoded_strs[i - 1],
                encoded_strs[i]
            );
        }
    }

    #[test]
    fn test_128_bit_boundary() {
        // Test values near the 128-bit boundary
        let test_cases = vec![u128::MAX - 1, u128::MAX, 1u128 << 127, (1u128 << 127) - 1];

        for value in test_cases {
            let mut buf = [0u8; 26];
            let encoded = encode_u128(value, &mut buf).unwrap();
            let decoded = decode_u128(encoded).unwrap();
            assert_eq!(decoded, value);
        }
    }

    #[test]
    fn test_encode_string_length() {
        let mut buf = [0u8; 26];
        let encoded = encode_u128(12345, &mut buf).unwrap();
        assert_eq!(encoded.len(), 26);
        assert_eq!(encoded.chars().count(), 26);
    }

    #[test]
    fn test_decode_mixed_case() {
        let value = 0x0123_4567_89AB_CDEF_u128;
        let mut buf = [0u8; 26];
        let encoded = encode_u128(value, &mut buf).unwrap();

        // Create mixed case version
        let mixed: String = encoded
            .chars()
            .enumerate()
            .map(|(i, c)| {
                if i % 2 == 0 {
                    c.to_lowercase().next().unwrap()
                } else {
                    c
                }
            })
            .collect();

        let decoded = decode_u128(&mixed).unwrap();
        assert_eq!(decoded, value);
    }

    #[test]
    fn test_all_alphabet_chars_decodable() {
        for &ch in ALPHABET {
            let char_value = ch as char;
            let s = format!("{char_value:0<26}");

            // Should not panic and should decode to some value
            let _ = decode_u128(&s).unwrap();
        }
    }
}

//! Core NULID type with 128-bit layout (68-bit timestamp + 60-bit random).

use crate::{Error, Result};
use rand::Rng;
use std::cmp::Ordering;
use std::fmt;
use std::str::FromStr;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// A NULID (Nanosecond-Precision Universally Lexicographically Sortable Identifier).
///
/// NULID is a 128-bit identifier with:
/// - 68 bits for nanoseconds since Unix epoch (~9,356 years lifespan)
/// - 60 bits for cryptographically secure randomness
///
/// The layout ensures lexicographic sortability with nanosecond precision.
///
/// # Examples
///
/// ```
/// use nulid::Nulid;
///
/// # fn main() -> nulid::Result<()> {
/// // Generate a new NULID
/// let id = Nulid::new()?;
///
/// // Extract components
/// let timestamp = id.nanos();
/// let random = id.random();
///
/// // Convert to string
/// let s = id.to_string();
///
/// // Parse from string
/// let parsed: Nulid = s.parse()?;
/// assert_eq!(id, parsed);
/// # Ok(())
/// # }
/// ```
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(
    feature = "rkyv",
    derive(::rkyv::Archive, ::rkyv::Serialize, ::rkyv::Deserialize)
)]
#[repr(transparent)]
pub struct Nulid(u128);

impl Nulid {
    /// Number of bits used for the timestamp (nanoseconds).
    pub const TIMESTAMP_BITS: u32 = 68;

    /// Number of bits used for randomness.
    pub const RANDOM_BITS: u32 = 60;

    /// Bit shift for timestamp (equals `RANDOM_BITS`).
    const TIMESTAMP_SHIFT: u32 = Self::RANDOM_BITS;

    /// Mask for extracting the random bits (lower 60 bits).
    const RANDOM_MASK: u128 = (1u128 << Self::RANDOM_BITS) - 1;

    /// Mask for the timestamp (68 bits).
    const TIMESTAMP_MASK: u128 = (1u128 << Self::TIMESTAMP_BITS) - 1;

    /// The minimum NULID value (all zeros).
    pub const MIN: Self = Self(0);

    /// The maximum NULID value (all ones).
    pub const MAX: Self = Self(u128::MAX);

    /// A zero NULID (same as MIN).
    pub const ZERO: Self = Self::MIN;

    /// Creates a nil (zero) NULID.
    ///
    /// # Examples
    ///
    /// ```
    /// use nulid::Nulid;
    ///
    /// let nil = Nulid::nil();
    /// assert!(nil.is_nil());
    /// assert_eq!(nil.nanos(), 0);
    /// assert_eq!(nil.random(), 0);
    /// ```
    #[must_use]
    pub const fn nil() -> Self {
        Self::ZERO
    }

    /// Returns `true` if this NULID is nil (all zeros).
    ///
    /// # Examples
    ///
    /// ```
    /// use nulid::Nulid;
    ///
    /// assert!(Nulid::nil().is_nil());
    /// assert!(Nulid::from_u128(0).is_nil());
    /// ```
    #[must_use]
    pub const fn is_nil(self) -> bool {
        self.0 == 0
    }

    /// Returns the minimum possible NULID value (all zeros).
    ///
    /// # Examples
    ///
    /// ```
    /// use nulid::Nulid;
    ///
    /// let min = Nulid::min();
    /// assert_eq!(min, Nulid::MIN);
    /// assert!(min.is_nil());
    /// ```
    #[must_use]
    pub const fn min() -> Self {
        Self::MIN
    }

    /// Returns the maximum possible NULID value (all ones).
    ///
    /// # Examples
    ///
    /// ```
    /// use nulid::Nulid;
    ///
    /// let max = Nulid::max();
    /// assert_eq!(max, Nulid::MAX);
    /// assert_eq!(max.as_u128(), u128::MAX);
    /// ```
    #[must_use]
    pub const fn max() -> Self {
        Self::MAX
    }

    /// Generates a new NULID with the current timestamp and random bits.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The system time is before Unix epoch
    /// - Random number generation fails
    ///
    /// # Examples
    ///
    /// ```
    /// use nulid::Nulid;
    ///
    /// # fn main() -> nulid::Result<()> {
    /// let id = Nulid::new()?;
    /// assert!(id.nanos() > 0);
    /// assert!(id.random() > 0);
    /// # Ok(())
    /// # }
    /// ```
    pub fn new() -> Result<Self> {
        Self::now()
    }

    /// Generates a new NULID with the current timestamp and random bits.
    ///
    /// This is an alias for [`new()`](Self::new).
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The system time is before Unix epoch
    /// - Random number generation fails
    pub fn now() -> Result<Self> {
        let timestamp_nanos = crate::time::now_nanos()?;
        // Generate 60-bit cryptographically secure random value using rand's thread-local RNG
        let random = rand::rng().random::<u64>() & ((1u64 << Self::RANDOM_BITS) - 1);
        Ok(Self::from_nanos(timestamp_nanos, random))
    }

    /// Creates a NULID from a `SystemTime` with random bits.
    ///
    /// # Examples
    ///
    /// ```
    /// use nulid::Nulid;
    /// use std::time::SystemTime;
    ///
    /// # fn main() -> nulid::Result<()> {
    /// let time = SystemTime::now();
    /// let id = Nulid::from_datetime(time)?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The time is before Unix epoch
    /// - Random number generation fails
    pub fn from_datetime(time: SystemTime) -> Result<Self> {
        let duration = time
            .duration_since(UNIX_EPOCH)
            .map_err(|_| Error::SystemTimeError)?;

        let timestamp_nanos =
            u128::from(duration.as_secs()) * 1_000_000_000 + u128::from(duration.subsec_nanos());

        // Generate 60-bit cryptographically secure random value using rand's thread-local RNG
        let random = rand::rng().random::<u64>() & ((1u64 << Self::RANDOM_BITS) - 1);
        Ok(Self::from_nanos(timestamp_nanos, random))
    }

    /// Creates a NULID from a timestamp (nanoseconds) and random value.
    ///
    /// The timestamp is masked to 68 bits and the random value is masked to 60 bits.
    ///
    /// # Examples
    ///
    /// ```
    /// use nulid::Nulid;
    ///
    /// let id = Nulid::from_nanos(1_000_000_000_000, 12345);
    /// assert_eq!(id.nanos(), 1_000_000_000_000);
    /// assert_eq!(id.random(), 12345);
    /// ```
    #[must_use]
    pub const fn from_nanos(timestamp_nanos: u128, random: u64) -> Self {
        let ts = timestamp_nanos & Self::TIMESTAMP_MASK;
        let rand = (random as u128) & Self::RANDOM_MASK;
        let value = (ts << Self::TIMESTAMP_SHIFT) | rand;
        Self(value)
    }

    /// Creates a NULID from a raw `u128` value.
    ///
    /// # Examples
    ///
    /// ```
    /// use nulid::Nulid;
    ///
    /// let id = Nulid::from_u128(0x0123_4567_89AB_CDEF_FEDC_BA98_7654_3210);
    /// assert_eq!(id.as_u128(), 0x0123_4567_89AB_CDEF_FEDC_BA98_7654_3210);
    /// ```
    #[must_use]
    pub const fn from_u128(value: u128) -> Self {
        Self(value)
    }

    /// Creates a NULID from a 16-byte array (big-endian).
    ///
    /// # Examples
    ///
    /// ```
    /// use nulid::Nulid;
    ///
    /// let bytes = [0u8; 16];
    /// let id = Nulid::from_bytes(bytes);
    /// assert_eq!(id.to_bytes(), bytes);
    /// ```
    #[must_use]
    pub const fn from_bytes(bytes: [u8; 16]) -> Self {
        Self(u128::from_be_bytes(bytes))
    }

    /// Extracts the timestamp in nanoseconds since Unix epoch.
    ///
    /// # Examples
    ///
    /// ```
    /// use nulid::Nulid;
    ///
    /// let id = Nulid::from_nanos(1_234_567_890_123_456_789, 0);
    /// assert_eq!(id.nanos(), 1_234_567_890_123_456_789);
    /// ```
    #[must_use]
    pub const fn nanos(self) -> u128 {
        self.0 >> Self::TIMESTAMP_SHIFT
    }

    /// Extracts the timestamp in microseconds since Unix epoch.
    ///
    /// # Examples
    ///
    /// ```
    /// use nulid::Nulid;
    ///
    /// let id = Nulid::from_nanos(1_234_567_890_000, 0);
    /// assert_eq!(id.micros(), 1_234_567_890);
    /// ```
    #[must_use]
    pub const fn micros(self) -> u128 {
        self.nanos() / 1_000
    }

    /// Extracts the timestamp in milliseconds since Unix epoch.
    ///
    /// # Examples
    ///
    /// ```
    /// use nulid::Nulid;
    ///
    /// let id = Nulid::from_nanos(1_234_567_000_000, 0);
    /// assert_eq!(id.millis(), 1_234_567);
    /// ```
    #[must_use]
    pub const fn millis(self) -> u128 {
        self.nanos() / 1_000_000
    }

    /// Extracts the random component (60 bits) from this NULID.
    ///
    /// # Examples
    ///
    /// ```
    /// use nulid::Nulid;
    ///
    /// let id = Nulid::from_nanos(0, 123456);
    /// assert_eq!(id.random(), 123456);
    /// ```
    #[must_use]
    pub const fn random(self) -> u64 {
        (self.0 & Self::RANDOM_MASK) as u64
    }

    /// Extracts both timestamp and random components.
    ///
    /// # Returns
    ///
    /// A tuple of `(timestamp_nanos, random)`.
    ///
    /// # Examples
    ///
    /// ```
    /// use nulid::Nulid;
    ///
    /// let id = Nulid::from_nanos(1_000_000_000, 12345);
    /// let (ts, rand) = id.parts();
    /// assert_eq!(ts, 1_000_000_000);
    /// assert_eq!(rand, 12345);
    /// ```
    #[must_use]
    pub const fn parts(self) -> (u128, u64) {
        (self.nanos(), self.random())
    }

    /// Extracts the seconds component from the timestamp.
    ///
    /// This method divides the nanosecond timestamp by 1 billion to get seconds.
    /// The cast to `u64` is always safe because NULID uses a 68-bit timestamp,
    /// which allows a maximum of ~295 billion seconds (~9,353 years, valid until
    /// year ~11,323 AD), well within `u64::MAX` (~18.4 quintillion seconds).
    ///
    /// # Examples
    ///
    /// ```
    /// use nulid::Nulid;
    ///
    /// let id = Nulid::from_nanos(1_234_567_890_123_456_789, 0);
    /// assert_eq!(id.seconds(), 1_234_567_890);
    /// ```
    #[must_use]
    #[allow(clippy::cast_possible_truncation)]
    pub const fn seconds(self) -> u64 {
        let seconds = self.nanos() / 1_000_000_000;

        // Safety: 68-bit timestamp in nanoseconds divided by 1 billion
        // yields at most ~295 billion, which fits comfortably in u64.
        debug_assert!(seconds <= u64::MAX as u128);

        seconds as u64
    }

    /// Extracts the subsecond nanoseconds from the timestamp.
    ///
    /// This method uses the modulo operation with 1 billion to extract the
    /// nanosecond component. The result is guaranteed to be less than
    /// 1,000,000,000 and always fits in `u32`.
    ///
    /// # Examples
    ///
    /// ```
    /// use nulid::Nulid;
    ///
    /// let id = Nulid::from_nanos(1_234_567_890_123_456_789, 0);
    /// assert_eq!(id.subsec_nanos(), 123_456_789);
    /// ```
    #[must_use]
    #[allow(clippy::cast_possible_truncation)]
    pub const fn subsec_nanos(self) -> u32 {
        let subsec = self.nanos() % 1_000_000_000;

        // Safety: Modulo 1 billion guarantees the result is always less than 1 billion,
        // which fits comfortably in u32 (max ~4.29 billion).
        debug_assert!(subsec < 1_000_000_000);

        subsec as u32
    }

    /// Returns the raw `u128` value of this NULID.
    ///
    /// # Examples
    ///
    /// ```
    /// use nulid::Nulid;
    ///
    /// let value = 0x0123_4567_89AB_CDEF_FEDC_BA98_7654_3210u128;
    /// let id = Nulid::from_u128(value);
    /// assert_eq!(id.as_u128(), value);
    /// ```
    #[must_use]
    pub const fn as_u128(self) -> u128 {
        self.0
    }

    /// Converts this NULID to a 16-byte array (big-endian).
    ///
    /// # Examples
    ///
    /// ```
    /// use nulid::Nulid;
    ///
    /// let id = Nulid::from_u128(0x0123_4567_89AB_CDEF_FEDC_BA98_7654_3210);
    /// let bytes = id.to_bytes();
    /// assert_eq!(Nulid::from_bytes(bytes), id);
    /// ```
    #[must_use]
    pub const fn to_bytes(self) -> [u8; 16] {
        self.0.to_be_bytes()
    }

    /// Converts this NULID to a `SystemTime`.
    ///
    /// # Examples
    ///
    /// ```
    /// use nulid::Nulid;
    /// use std::time::{SystemTime, UNIX_EPOCH};
    ///
    /// # fn main() -> nulid::Result<()> {
    /// let id = Nulid::new()?;
    /// let time = id.datetime();
    /// assert!(time > UNIX_EPOCH);
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    #[allow(clippy::cast_possible_truncation)]
    #[allow(clippy::missing_const_for_fn)]
    pub fn datetime(self) -> SystemTime {
        let nanos = self.nanos();
        let secs = (nanos / 1_000_000_000) as u64;
        let subsec_nanos = (nanos % 1_000_000_000) as u32;
        UNIX_EPOCH + Duration::new(secs, subsec_nanos)
    }

    /// Returns the duration since Unix epoch.
    ///
    /// # Examples
    ///
    /// ```
    /// use nulid::Nulid;
    ///
    /// let id = Nulid::from_nanos(5_000_000_000, 0);
    /// let duration = id.duration_since_epoch();
    /// assert_eq!(duration.as_secs(), 5);
    /// ```
    #[must_use]
    #[allow(clippy::cast_possible_truncation)]
    pub const fn duration_since_epoch(self) -> Duration {
        let nanos = self.nanos();
        let secs = (nanos / 1_000_000_000) as u64;
        let subsec_nanos = (nanos % 1_000_000_000) as u32;
        Duration::new(secs, subsec_nanos)
    }

    /// Increments this NULID by 1, returning `None` on overflow.
    ///
    /// This is useful for monotonic generation when multiple IDs are generated
    /// within the same nanosecond.
    ///
    /// # Examples
    ///
    /// ```
    /// use nulid::Nulid;
    ///
    /// let id1 = Nulid::from_u128(100);
    /// let id2 = id1.increment().unwrap();
    /// assert_eq!(id2.as_u128(), 101);
    ///
    /// let max = Nulid::MAX;
    /// assert!(max.increment().is_none());
    /// ```
    #[must_use]
    pub const fn increment(self) -> Option<Self> {
        match self.0.checked_add(1) {
            Some(value) => Some(Self(value)),
            None => None,
        }
    }

    /// Encodes this NULID to Base32 (Crockford) into the provided buffer.
    ///
    /// Returns a string slice pointing to the encoded data in the buffer.
    ///
    /// # Errors
    ///
    /// Returns an error if UTF-8 encoding fails (should never occur with valid ALPHABET).
    ///
    /// # Examples
    ///
    /// ```
    /// use nulid::Nulid;
    ///
    /// # fn main() -> nulid::Result<()> {
    /// let id = Nulid::new()?;
    /// let mut buf = [0u8; 26];
    /// let s = id.encode(&mut buf)?;
    /// assert_eq!(s.len(), 26);
    /// # Ok(())
    /// # }
    /// ```
    pub fn encode(self, buf: &mut [u8; 26]) -> Result<&str> {
        crate::base32::encode_u128(self.0, buf)
    }
}

impl fmt::Debug for Nulid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut buf = [0u8; 26];
        let s = self.encode(&mut buf).map_err(|_| fmt::Error)?;
        f.debug_tuple("Nulid").field(&s).finish()
    }
}

impl fmt::Display for Nulid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut buf = [0u8; 26];
        let s = self.encode(&mut buf).map_err(|_| fmt::Error)?;
        f.write_str(s)
    }
}

impl FromStr for Nulid {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let value = crate::base32::decode_u128(s)?;
        Ok(Self::from_u128(value))
    }
}

impl Ord for Nulid {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }
}

impl PartialOrd for Nulid {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Default for Nulid {
    fn default() -> Self {
        Self::ZERO
    }
}

impl From<u128> for Nulid {
    fn from(value: u128) -> Self {
        Self::from_u128(value)
    }
}

impl From<Nulid> for u128 {
    fn from(nulid: Nulid) -> Self {
        nulid.as_u128()
    }
}

impl From<[u8; 16]> for Nulid {
    fn from(bytes: [u8; 16]) -> Self {
        Self::from_bytes(bytes)
    }
}

impl From<Nulid> for [u8; 16] {
    fn from(nulid: Nulid) -> Self {
        nulid.to_bytes()
    }
}

impl AsRef<u128> for Nulid {
    fn as_ref(&self) -> &u128 {
        &self.0
    }
}

impl TryFrom<&[u8]> for Nulid {
    type Error = Error;

    fn try_from(bytes: &[u8]) -> Result<Self> {
        if bytes.len() != 16 {
            return Err(Error::InvalidLength {
                expected: 16,
                found: bytes.len(),
            });
        }
        let mut arr = [0u8; 16];
        arr.copy_from_slice(bytes);
        Ok(Self::from_bytes(arr))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nil() {
        let nil = Nulid::nil();
        assert!(nil.is_nil());
        assert_eq!(nil.nanos(), 0);
        assert_eq!(nil.random(), 0);
        assert_eq!(nil.as_u128(), 0);
    }

    #[test]
    fn test_from_nanos() {
        let timestamp = 1_234_567_890_123_456_789u128;
        let random = 987_654_321u64;
        let id = Nulid::from_nanos(timestamp, random);

        assert_eq!(id.nanos(), timestamp);
        assert_eq!(id.random(), random);
    }

    #[test]
    fn test_nanos() {
        let timestamp = 1_234_567_890_123_456_789u128;
        let id = Nulid::from_nanos(timestamp, 0);
        assert_eq!(id.nanos(), timestamp);
    }

    #[test]
    fn test_micros() {
        // Test basic conversion
        let nanos = 1_234_567_890_000u128; // 1,234,567,890 microseconds
        let id = Nulid::from_nanos(nanos, 0);
        assert_eq!(id.micros(), 1_234_567_890);

        // Test rounding (should truncate)
        let nanos_with_remainder = 1_234_567_890_999u128;
        let id2 = Nulid::from_nanos(nanos_with_remainder, 0);
        assert_eq!(id2.micros(), 1_234_567_890);

        // Test zero
        let id_zero = Nulid::from_nanos(0, 0);
        assert_eq!(id_zero.micros(), 0);
    }

    #[test]
    fn test_millis() {
        // Test basic conversion
        let nanos = 1_234_567_000_000u128; // 1,234,567 milliseconds
        let id = Nulid::from_nanos(nanos, 0);
        assert_eq!(id.millis(), 1_234_567);

        // Test rounding (should truncate)
        let nanos_with_remainder = 1_234_567_999_999u128;
        let id2 = Nulid::from_nanos(nanos_with_remainder, 0);
        assert_eq!(id2.millis(), 1_234_567);

        // Test zero
        let id_zero = Nulid::from_nanos(0, 0);
        assert_eq!(id_zero.millis(), 0);
    }

    #[test]
    fn test_timestamp_conversions() {
        // Test that nanos, micros, and millis are consistent
        let nanos = 1_234_567_890_123_456_789u128;
        let id = Nulid::from_nanos(nanos, 0);

        let micros_from_nanos = id.nanos() / 1_000;
        let millis_from_nanos = id.nanos() / 1_000_000;

        assert_eq!(id.micros(), micros_from_nanos);
        assert_eq!(id.millis(), millis_from_nanos);
    }

    #[test]
    fn test_parts() {
        let timestamp = 5_000_000_000u128;
        let random = 12345u64;
        let id = Nulid::from_nanos(timestamp, random);

        let (ts, rand) = id.parts();
        assert_eq!(ts, timestamp);
        assert_eq!(rand, random);
    }

    #[test]
    fn test_seconds_and_subsec_nanos() {
        let timestamp = 1_234_567_890_123_456_789u128;
        let id = Nulid::from_nanos(timestamp, 0);

        assert_eq!(id.seconds(), 1_234_567_890);
        assert_eq!(id.subsec_nanos(), 123_456_789);
    }

    #[test]
    fn test_seconds_maximum_timestamp() {
        // Test with maximum 68-bit timestamp value
        let max_68bit = (1u128 << 68) - 1; // 295_147_905_179_352_825_855 nanoseconds
        let id = Nulid::from_nanos(max_68bit, 0);

        // Should safely convert to seconds without overflow
        let seconds = id.seconds();
        assert_eq!(seconds, 295_147_905_179); // ~9,353 years from epoch

        // Verify the cast is safe: result fits comfortably in u64
        assert!(seconds < u64::MAX);

        // Subsec nanos should work correctly too
        let subsec = id.subsec_nanos();
        assert_eq!(subsec, 352_825_855);

        // Verify timestamp is preserved
        assert_eq!(id.nanos(), max_68bit);
    }

    #[test]
    fn test_subsec_nanos_invariants() {
        // Test that subsec_nanos() always returns a value < 1 billion

        // Test with various timestamps
        let test_cases = [
            0u128,
            999_999_999,               // Just under 1 second
            1_000_000_000,             // Exactly 1 second
            1_000_000_001,             // Just over 1 second
            1_234_567_890_123_456_789, // Regular timestamp
            999_999_999_999_999_999,   // Large timestamp
            (1u128 << 68) - 1,         // Maximum 68-bit value
        ];

        for timestamp in test_cases {
            let id = Nulid::from_nanos(timestamp, 0);
            let subsec = id.subsec_nanos();

            // Verify invariant: subsec_nanos is always < 1 billion
            assert!(
                subsec < 1_000_000_000,
                "subsec_nanos() returned {subsec}, which is >= 1 billion for timestamp {timestamp}"
            );

            // Verify it matches the expected modulo result
            let expected = (timestamp % 1_000_000_000) as u32;
            assert_eq!(
                subsec, expected,
                "subsec_nanos() mismatch for timestamp {timestamp}"
            );
        }
    }

    #[test]
    fn test_seconds_and_subsec_nanos_reconstruction() {
        // Verify that seconds and subsec_nanos can reconstruct the original timestamp
        let test_timestamps = [
            1_234_567_890_123_456_789u128,
            5_000_000_000_000_000_000,
            999_999_999,
            1_000_000_000_000_000_000,
        ];

        for original_ts in test_timestamps {
            let id = Nulid::from_nanos(original_ts, 0);
            let seconds = id.seconds();
            let subsec = id.subsec_nanos();

            // Reconstruct timestamp
            let reconstructed = u128::from(seconds) * 1_000_000_000 + u128::from(subsec);

            assert_eq!(
                reconstructed, original_ts,
                "Failed to reconstruct timestamp {original_ts} from seconds {seconds} and subsec_nanos {subsec}"
            );
        }
    }

    #[test]
    fn test_from_to_bytes() {
        let id = Nulid::from_u128(0x0123_4567_89AB_CDEF_FEDC_BA98_7654_3210);
        let bytes = id.to_bytes();
        let id2 = Nulid::from_bytes(bytes);
        assert_eq!(id, id2);
    }

    #[test]
    fn test_ordering() {
        let id1 = Nulid::from_u128(100);
        let id2 = Nulid::from_u128(200);
        let id3 = Nulid::from_u128(200);

        assert!(id1 < id2);
        assert!(id2 > id1);
        assert_eq!(id2, id3);
    }

    #[test]
    fn test_increment() {
        let id = Nulid::from_u128(100);
        let next = id.increment().unwrap();
        assert_eq!(next.as_u128(), 101);

        let max = Nulid::MAX;
        assert!(max.increment().is_none());
    }

    #[test]
    fn test_timestamp_ordering() {
        let id1 = Nulid::from_nanos(1000, 500);
        let id2 = Nulid::from_nanos(2000, 100);

        // Earlier timestamp should be less, regardless of random value
        assert!(id1 < id2);
    }

    #[test]
    fn test_random_ordering_same_timestamp() {
        let id1 = Nulid::from_nanos(1000, 100);
        let id2 = Nulid::from_nanos(1000, 200);

        // Same timestamp, random value determines order
        assert!(id1 < id2);
    }

    #[test]
    fn test_constants() {
        assert_eq!(Nulid::MIN.as_u128(), 0);
        assert_eq!(Nulid::MAX.as_u128(), u128::MAX);
        assert_eq!(Nulid::ZERO.as_u128(), 0);
    }

    #[test]
    fn test_bit_masks() {
        // Verify timestamp mask is 68 bits
        assert_eq!(Nulid::TIMESTAMP_MASK, (1u128 << 68) - 1);

        // Verify random mask is 60 bits
        assert_eq!(Nulid::RANDOM_MASK, (1u128 << 60) - 1);
    }

    #[test]
    fn test_masking() {
        // Test that values are properly masked
        let large_ts = u128::MAX;
        let large_rand = u64::MAX;

        let id = Nulid::from_nanos(large_ts, large_rand);

        // Values should be masked to their bit limits
        assert!(id.nanos() <= Nulid::TIMESTAMP_MASK);
        assert!(id.random() < (1u64 << Nulid::RANDOM_BITS));
    }

    #[test]
    fn test_from_u128() {
        let value = 0x0123_4567_89AB_CDEF_FEDC_BA98_7654_3210u128;
        let id: Nulid = value.into();
        assert_eq!(id.as_u128(), value);
    }

    #[test]
    fn test_into_u128() {
        let id = Nulid::from_u128(0x0123_4567_89AB_CDEF_FEDC_BA98_7654_3210);
        let value: u128 = id.into();
        assert_eq!(value, 0x0123_4567_89AB_CDEF_FEDC_BA98_7654_3210);
    }

    #[test]
    fn test_from_bytes_trait() {
        let bytes = [
            0x01, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD, 0xEF, 0xFE, 0xDC, 0xBA, 0x98, 0x76, 0x54,
            0x32, 0x10,
        ];
        let id: Nulid = bytes.into();
        assert_eq!(id.to_bytes(), bytes);
    }

    #[test]
    fn test_into_bytes() {
        let id = Nulid::from_u128(0x0123_4567_89AB_CDEF_FEDC_BA98_7654_3210);
        let bytes: [u8; 16] = id.into();
        assert_eq!(bytes, id.to_bytes());
    }

    #[test]
    fn test_as_ref_u128() {
        let id = Nulid::from_u128(12345);
        let value_ref: &u128 = id.as_ref();
        assert_eq!(*value_ref, 12345);
    }

    #[test]
    fn test_try_from_slice_valid() {
        let bytes = [
            0x01, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD, 0xEF, 0xFE, 0xDC, 0xBA, 0x98, 0x76, 0x54,
            0x32, 0x10,
        ];
        let slice: &[u8] = &bytes;
        let id = Nulid::try_from(slice).unwrap();
        assert_eq!(id.to_bytes(), bytes);
    }

    #[test]
    fn test_try_from_slice_invalid_length() {
        let bytes = [0u8; 15]; // Wrong length
        let slice: &[u8] = &bytes;
        let result = Nulid::try_from(slice);
        assert!(result.is_err());
        match result {
            Err(Error::InvalidLength { expected, found }) => {
                assert_eq!(expected, 16);
                assert_eq!(found, 15);
            }
            _ => panic!("Expected InvalidLength error"),
        }
    }

    #[test]
    fn test_try_from_slice_too_long() {
        let bytes = [0u8; 20]; // Too long
        let slice: &[u8] = &bytes;
        let result = Nulid::try_from(slice);
        assert!(result.is_err());
        match result {
            Err(Error::InvalidLength { expected, found }) => {
                assert_eq!(expected, 16);
                assert_eq!(found, 20);
            }
            _ => panic!("Expected InvalidLength error"),
        }
    }

    #[test]
    fn test_try_from_empty_slice() {
        let bytes: &[u8] = &[];
        let result = Nulid::try_from(bytes);
        assert!(result.is_err());
        match result {
            Err(Error::InvalidLength { expected, found }) => {
                assert_eq!(expected, 16);
                assert_eq!(found, 0);
            }
            _ => panic!("Expected InvalidLength error"),
        }
    }
}

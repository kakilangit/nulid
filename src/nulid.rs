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
/// let timestamp = id.timestamp_nanos();
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
    /// assert_eq!(nil.timestamp_nanos(), 0);
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
    /// assert!(id.timestamp_nanos() > 0);
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
        Ok(Self::from_timestamp_nanos(timestamp_nanos, random))
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
        Ok(Self::from_timestamp_nanos(timestamp_nanos, random))
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
    /// let id = Nulid::from_timestamp_nanos(1_000_000_000_000, 12345);
    /// assert_eq!(id.timestamp_nanos(), 1_000_000_000_000);
    /// assert_eq!(id.random(), 12345);
    /// ```
    #[must_use]
    pub const fn from_timestamp_nanos(timestamp_nanos: u128, random: u64) -> Self {
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
    /// let id = Nulid::from_u128(0x0123456789ABCDEF_FEDCBA9876543210);
    /// assert_eq!(id.as_u128(), 0x0123456789ABCDEF_FEDCBA9876543210);
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

    /// Extracts the timestamp (nanoseconds since Unix epoch) from this NULID.
    ///
    /// # Examples
    ///
    /// ```
    /// use nulid::Nulid;
    ///
    /// let id = Nulid::from_timestamp_nanos(1_234_567_890_123_456_789, 0);
    /// assert_eq!(id.timestamp_nanos(), 1_234_567_890_123_456_789);
    /// ```
    #[must_use]
    pub const fn timestamp_nanos(self) -> u128 {
        self.0 >> Self::TIMESTAMP_SHIFT
    }

    /// Extracts the random component (60 bits) from this NULID.
    ///
    /// # Examples
    ///
    /// ```
    /// use nulid::Nulid;
    ///
    /// let id = Nulid::from_timestamp_nanos(0, 123456);
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
    /// let id = Nulid::from_timestamp_nanos(1_000_000_000, 12345);
    /// let (ts, rand) = id.parts();
    /// assert_eq!(ts, 1_000_000_000);
    /// assert_eq!(rand, 12345);
    /// ```
    #[must_use]
    pub const fn parts(self) -> (u128, u64) {
        (self.timestamp_nanos(), self.random())
    }

    /// Extracts the seconds component from the timestamp.
    ///
    /// This is a convenience method that divides the nanosecond timestamp by 1 billion.
    ///
    /// # Examples
    ///
    /// ```
    /// use nulid::Nulid;
    ///
    /// let id = Nulid::from_timestamp_nanos(1_234_567_890_123_456_789, 0);
    /// assert_eq!(id.seconds(), 1_234_567_890);
    /// ```
    #[must_use]
    #[allow(clippy::cast_possible_truncation)]
    pub const fn seconds(self) -> u64 {
        (self.timestamp_nanos() / 1_000_000_000) as u64
    }

    /// Extracts the subsecond nanoseconds from the timestamp.
    ///
    /// # Examples
    ///
    /// ```
    /// use nulid::Nulid;
    ///
    /// let id = Nulid::from_timestamp_nanos(1_234_567_890_123_456_789, 0);
    /// assert_eq!(id.subsec_nanos(), 123_456_789);
    /// ```
    #[must_use]
    pub const fn subsec_nanos(self) -> u32 {
        (self.timestamp_nanos() % 1_000_000_000) as u32
    }

    /// Returns the raw `u128` value of this NULID.
    ///
    /// # Examples
    ///
    /// ```
    /// use nulid::Nulid;
    ///
    /// let value = 0x0123456789ABCDEF_FEDCBA9876543210u128;
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
    /// let id = Nulid::from_u128(0x0123456789ABCDEF_FEDCBA9876543210);
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
    pub fn datetime(self) -> SystemTime {
        let nanos = self.timestamp_nanos();
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
    /// let id = Nulid::from_timestamp_nanos(5_000_000_000, 0);
    /// let duration = id.duration_since_epoch();
    /// assert_eq!(duration.as_secs(), 5);
    /// ```
    #[must_use]
    #[allow(clippy::cast_possible_truncation)]
    pub const fn duration_since_epoch(self) -> Duration {
        let nanos = self.timestamp_nanos();
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

    /// Converts this NULID to a UUID.
    ///
    /// Requires the `uuid` feature flag.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[cfg(feature = "uuid")]
    /// # {
    /// use nulid::Nulid;
    ///
    /// # fn main() -> nulid::Result<()> {
    /// let nulid = Nulid::new()?;
    /// let uuid = nulid.to_uuid();
    /// assert_eq!(uuid.as_u128(), nulid.as_u128());
    /// # Ok(())
    /// # }
    /// # }
    /// ```
    #[cfg(feature = "uuid")]
    #[must_use]
    pub const fn to_uuid(self) -> uuid::Uuid {
        uuid::Uuid::from_u128(self.0)
    }

    /// Creates a NULID from a UUID.
    ///
    /// Requires the `uuid` feature flag.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[cfg(feature = "uuid")]
    /// # {
    /// use nulid::Nulid;
    /// use uuid::Uuid;
    ///
    /// let uuid = Uuid::new_v4();
    /// let nulid = Nulid::from_uuid(uuid);
    /// assert_eq!(nulid.to_uuid(), uuid);
    /// # }
    /// ```
    #[cfg(feature = "uuid")]
    #[must_use]
    pub const fn from_uuid(uuid: uuid::Uuid) -> Self {
        Self(uuid.as_u128())
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

#[cfg(feature = "uuid")]
impl From<uuid::Uuid> for Nulid {
    fn from(uuid: uuid::Uuid) -> Self {
        Self::from_uuid(uuid)
    }
}

#[cfg(feature = "uuid")]
impl From<Nulid> for uuid::Uuid {
    fn from(nulid: Nulid) -> Self {
        nulid.to_uuid()
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for Nulid {
    fn serialize<S>(&self, serializer: S) -> core::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        if serializer.is_human_readable() {
            serializer.serialize_str(&self.to_string())
        } else {
            serializer.serialize_bytes(&self.to_bytes())
        }
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Nulid {
    fn deserialize<D>(deserializer: D) -> core::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            let s = <&str>::deserialize(deserializer)?;
            Self::from_str(s).map_err(serde::de::Error::custom)
        } else {
            let bytes = <[u8; 16]>::deserialize(deserializer)?;
            Ok(Self::from_bytes(bytes))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nil() {
        let nil = Nulid::nil();
        assert!(nil.is_nil());
        assert_eq!(nil.timestamp_nanos(), 0);
        assert_eq!(nil.random(), 0);
        assert_eq!(nil.as_u128(), 0);
    }

    #[test]
    fn test_from_timestamp_nanos() {
        let timestamp = 1_234_567_890_123_456_789u128;
        let random = 987_654_321u64;
        let id = Nulid::from_timestamp_nanos(timestamp, random);

        assert_eq!(id.timestamp_nanos(), timestamp);
        assert_eq!(id.random(), random);
    }

    #[test]
    fn test_parts() {
        let timestamp = 5_000_000_000u128;
        let random = 12345u64;
        let id = Nulid::from_timestamp_nanos(timestamp, random);

        let (ts, rand) = id.parts();
        assert_eq!(ts, timestamp);
        assert_eq!(rand, random);
    }

    #[test]
    fn test_seconds_and_subsec_nanos() {
        let timestamp = 1_234_567_890_123_456_789u128;
        let id = Nulid::from_timestamp_nanos(timestamp, 0);

        assert_eq!(id.seconds(), 1_234_567_890);
        assert_eq!(id.subsec_nanos(), 123_456_789);
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
        let id1 = Nulid::from_timestamp_nanos(1000, 500);
        let id2 = Nulid::from_timestamp_nanos(2000, 100);

        // Earlier timestamp should be less, regardless of random value
        assert!(id1 < id2);
    }

    #[test]
    fn test_random_ordering_same_timestamp() {
        let id1 = Nulid::from_timestamp_nanos(1000, 100);
        let id2 = Nulid::from_timestamp_nanos(1000, 200);

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

        let id = Nulid::from_timestamp_nanos(large_ts, large_rand);

        // Values should be masked to their bit limits
        assert!(id.timestamp_nanos() <= Nulid::TIMESTAMP_MASK);
        assert!(id.random() < (1u64 << Nulid::RANDOM_BITS));
    }

    #[cfg(feature = "uuid")]
    #[test]
    fn test_uuid_conversion() {
        let nulid = Nulid::from_u128(0x0123_4567_89AB_CDEF_FEDC_BA98_7654_3210);
        let uuid = nulid.to_uuid();

        assert_eq!(uuid.as_u128(), nulid.as_u128());

        let nulid2 = Nulid::from_uuid(uuid);
        assert_eq!(nulid, nulid2);
    }

    #[cfg(feature = "uuid")]
    #[test]
    fn test_uuid_from_trait() {
        let uuid = uuid::Uuid::from_u128(0x0123_4567_89AB_CDEF_FEDC_BA98_7654_3210);
        let nulid: Nulid = uuid.into();

        assert_eq!(nulid.as_u128(), uuid.as_u128());
    }

    #[cfg(feature = "uuid")]
    #[test]
    fn test_uuid_into_trait() {
        let nulid = Nulid::from_u128(0x0123_4567_89AB_CDEF_FEDC_BA98_7654_3210);
        let uuid: uuid::Uuid = nulid.into();

        assert_eq!(uuid.as_u128(), nulid.as_u128());
    }

    #[cfg(feature = "uuid")]
    #[test]
    fn test_uuid_round_trip() {
        let original = Nulid::new().unwrap();
        let uuid = original.to_uuid();
        let round_trip = Nulid::from_uuid(uuid);

        assert_eq!(original, round_trip);
        assert_eq!(original.as_u128(), uuid.as_u128());
    }
}

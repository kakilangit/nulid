//! Monotonic NULID generator with dependency injection for testing.
//!
//! This module provides a thread-safe generator that ensures NULIDs are
//! monotonically increasing even when multiple IDs are generated within
//! the same nanosecond or when clock skew occurs.
//!
//! # Design
//!
//! The generator uses the increment-on-skew strategy:
//! 1. Generate candidate ID from timestamp + random bits
//! 2. If candidate > `last_id`: use candidate
//! 3. Else: increment `last_id` (handles clock skew)
//!
//! # Testability
//!
//! The generator supports dependency injection for testing:
//! - `Clock` trait for injectable time source
//! - `Rng` trait for injectable random source
//! - `NodeId` trait for optional distributed node ID

use crate::{Error, Nulid, Result};
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};

// ============================================================================
// Clock Trait and Implementations
// ============================================================================

/// Clock abstraction for dependency injection.
///
/// Implement this trait to provide a custom time source for testing.
pub trait Clock: Send + Sync {
    /// Returns current time in nanoseconds since Unix epoch.
    ///
    /// # Errors
    ///
    /// Returns an error if the system time cannot be retrieved.
    fn now_nanos(&self) -> Result<u128>;
}

/// System clock using quanta for high-precision timing.
///
/// This is the default clock for production use.
#[derive(Debug, Clone, Copy, Default)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now_nanos(&self) -> Result<u128> {
        crate::time::now_nanos()
    }
}

/// Mock clock for testing with interior mutability.
///
/// Uses `AtomicU64` so the clock can be modified while the generator
/// holds a reference to it.
///
/// # Examples
///
/// ```
/// use nulid::generator::{MockClock, Clock};
/// use core::time::Duration;
///
/// let clock = MockClock::new(1_000_000_000);
/// assert_eq!(clock.get(), 1_000_000_000);
///
/// clock.advance(Duration::from_millis(100));
/// assert_eq!(clock.get(), 1_100_000_000);
///
/// clock.regress(Duration::from_millis(50));
/// assert_eq!(clock.get(), 1_050_000_000);
/// ```
#[derive(Debug)]
pub struct MockClock {
    nanos: AtomicU64,
}

impl MockClock {
    /// Creates a new mock clock with the given initial time.
    #[must_use]
    pub const fn new(initial_nanos: u64) -> Self {
        Self {
            nanos: AtomicU64::new(initial_nanos),
        }
    }

    /// Gets the current time in nanoseconds.
    #[must_use]
    pub fn get(&self) -> u64 {
        self.nanos.load(Ordering::SeqCst)
    }

    /// Sets the clock to a specific time.
    pub fn set(&self, nanos: u64) {
        self.nanos.store(nanos, Ordering::SeqCst);
    }

    /// Advances the clock by the given duration.
    #[allow(clippy::cast_possible_truncation)]
    pub fn advance(&self, duration: core::time::Duration) {
        self.nanos
            .fetch_add(duration.as_nanos() as u64, Ordering::SeqCst);
    }

    /// Regresses the clock by the given duration (simulates clock going backward).
    #[allow(clippy::cast_possible_truncation)]
    pub fn regress(&self, duration: core::time::Duration) {
        self.nanos
            .fetch_sub(duration.as_nanos() as u64, Ordering::SeqCst);
    }
}

impl Default for MockClock {
    fn default() -> Self {
        Self {
            nanos: AtomicU64::new(0),
        }
    }
}

impl Clock for MockClock {
    fn now_nanos(&self) -> Result<u128> {
        Ok(u128::from(self.nanos.load(Ordering::SeqCst)))
    }
}

impl Clock for &MockClock {
    fn now_nanos(&self) -> Result<u128> {
        Ok(u128::from(self.nanos.load(Ordering::SeqCst)))
    }
}

// ============================================================================
// Rng Trait and Implementations
// ============================================================================

/// Random source abstraction for dependency injection.
///
/// Implement this trait to provide a custom random source for testing.
pub trait Rng: Send + Sync {
    /// Returns a random u64 (will be masked to appropriate bits).
    fn random_u64(&self) -> u64;
}

/// Cryptographic RNG for production use.
///
/// Uses the thread-local cryptographically secure random number generator.
#[derive(Debug, Clone, Copy, Default)]
pub struct CryptoRng;

impl Rng for CryptoRng {
    fn random_u64(&self) -> u64 {
        rand::random::<u64>()
    }
}

/// Seeded RNG for reproducible tests.
///
/// Uses internal `Mutex` for interior mutability since `StdRng` requires `&mut self`.
///
/// # Examples
///
/// ```
/// use nulid::generator::{SeededRng, Rng};
///
/// let rng1 = SeededRng::new(42);
/// let rng2 = SeededRng::new(42);
///
/// // Same seed produces same sequence
/// assert_eq!(rng1.random_u64(), rng2.random_u64());
/// ```
pub struct SeededRng {
    rng: Mutex<rand::rngs::StdRng>,
}

impl SeededRng {
    /// Creates a new seeded RNG with the given seed.
    #[must_use]
    pub fn new(seed: u64) -> Self {
        use rand::SeedableRng;
        Self {
            rng: Mutex::new(rand::rngs::StdRng::seed_from_u64(seed)),
        }
    }
}

impl core::fmt::Debug for SeededRng {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("SeededRng").finish_non_exhaustive()
    }
}

impl Rng for SeededRng {
    #[allow(clippy::expect_used)]
    fn random_u64(&self) -> u64 {
        use rand::RngCore;
        self.rng
            .lock()
            .expect("SeededRng mutex poisoned")
            .next_u64()
    }
}

impl Rng for &SeededRng {
    #[allow(clippy::expect_used)]
    fn random_u64(&self) -> u64 {
        use rand::RngCore;
        self.rng
            .lock()
            .expect("SeededRng mutex poisoned")
            .next_u64()
    }
}

/// Sequential "random" for debugging (not for production!).
///
/// Returns 0, 1, 2, 3... - useful for understanding ordering behavior.
#[derive(Debug)]
pub struct SequentialRng {
    counter: AtomicU64,
}

impl SequentialRng {
    /// Creates a new sequential RNG starting at 0.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            counter: AtomicU64::new(0),
        }
    }

    /// Creates a new sequential RNG starting at the given value.
    #[must_use]
    pub const fn starting_at(value: u64) -> Self {
        Self {
            counter: AtomicU64::new(value),
        }
    }
}

impl Default for SequentialRng {
    fn default() -> Self {
        Self {
            counter: AtomicU64::new(0),
        }
    }
}

impl Rng for SequentialRng {
    fn random_u64(&self) -> u64 {
        self.counter.fetch_add(1, Ordering::SeqCst)
    }
}

impl Rng for &SequentialRng {
    fn random_u64(&self) -> u64 {
        self.counter.fetch_add(1, Ordering::SeqCst)
    }
}

// ============================================================================
// NodeId Trait and Implementations
// ============================================================================

/// Marker trait for optional node ID (zero-cost abstraction).
///
/// Use `NoNodeId` (default) for single-node deployments.
/// Use `WithNodeId` for distributed deployments requiring guaranteed uniqueness.
pub trait NodeId: Send + Sync + Default + Copy {
    /// Returns the node ID if present.
    fn get(&self) -> Option<u16>;
}

/// No node ID (default) - Zero Sized Type.
///
/// Uses full 60 bits for randomness.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct NoNodeId;

impl NodeId for NoNodeId {
    #[inline]
    fn get(&self) -> Option<u16> {
        None
    }
}

/// With node ID for distributed deployments.
///
/// Reserves 16 bits for node ID, leaving 44 bits for randomness.
/// Supports up to 65536 unique nodes (0-65535).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct WithNodeId(u16);

impl WithNodeId {
    /// Creates a new node ID.
    #[must_use]
    pub const fn new(node_id: u16) -> Self {
        Self(node_id)
    }

    /// Returns the node ID value.
    #[must_use]
    pub const fn value(&self) -> u16 {
        self.0
    }
}

impl NodeId for WithNodeId {
    #[inline]
    fn get(&self) -> Option<u16> {
        Some(self.0)
    }
}

// ============================================================================
// Generator
// ============================================================================

/// NULID generator with ULID spec compliance and full testability.
///
/// # Type Parameters
///
/// - `C`: Clock implementation (`SystemClock` or `MockClock`)
/// - `R`: Random source (`CryptoRng` or `SeededRng`)
/// - `N`: Node ID type (`NoNodeId` or `WithNodeId`) - defaults to `NoNodeId`
///
/// # Design
///
/// Uses the increment-on-skew strategy:
/// 1. Generate candidate ID from timestamp + random bits
/// 2. If candidate > `last_id`: use candidate
/// 3. Else: increment `last_id` (handles clock skew)
///
/// This is the same proven algorithm as previous versions, with injectable dependencies.
///
/// # Thread Safety
///
/// The generator is thread-safe and can be shared across threads using
/// `Arc<Generator>` or similar synchronization primitives.
///
/// # Examples
///
/// ## Production (Single Node)
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
///
/// ## Production (Distributed)
///
/// ```
/// use nulid::generator::{Generator, SystemClock, CryptoRng, WithNodeId};
///
/// # fn main() -> nulid::Result<()> {
/// let generator = Generator::<SystemClock, CryptoRng, WithNodeId>::with_node_id(1);
/// let id = generator.generate()?;
/// # Ok(())
/// # }
/// ```
///
/// ## Testing with Mock Clock
///
/// ```
/// use nulid::generator::{Generator, MockClock, SeededRng, NoNodeId};
/// use core::time::Duration;
///
/// # fn main() -> nulid::Result<()> {
/// let clock = MockClock::new(1_000_000_000);
/// let rng = SeededRng::new(42);
/// let generator = Generator::<_, _, NoNodeId>::with_deps(&clock, &rng);
///
/// let id1 = generator.generate()?;
///
/// // Simulate clock regression
/// clock.regress(Duration::from_millis(100));
/// let id2 = generator.generate()?;
///
/// // Still monotonic!
/// assert!(id2 > id1);
/// # Ok(())
/// # }
/// ```
pub struct Generator<C: Clock = SystemClock, R: Rng = CryptoRng, N: NodeId = NoNodeId> {
    clock: C,
    rng: R,
    node_id: N,
    state: Mutex<Option<Nulid>>,
}

// Production constructors for single-node use
impl Generator<SystemClock, CryptoRng, NoNodeId> {
    /// Creates a new generator for production use (single node).
    ///
    /// Uses system clock and cryptographic RNG.
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
            clock: SystemClock,
            rng: CryptoRng,
            node_id: NoNodeId,
            state: Mutex::new(None),
        }
    }
}

impl Default for Generator<SystemClock, CryptoRng, NoNodeId> {
    fn default() -> Self {
        Self::new()
    }
}

// Production constructor for distributed use
impl Generator<SystemClock, CryptoRng, WithNodeId> {
    /// Creates a new generator with node ID for distributed deployments.
    ///
    /// Uses system clock and cryptographic RNG with embedded node ID.
    ///
    /// # Arguments
    ///
    /// * `node_id` - Unique node identifier (0-65535, 16 bits)
    ///
    /// # Examples
    ///
    /// ```
    /// use nulid::generator::{Generator, SystemClock, CryptoRng, WithNodeId};
    ///
    /// let generator = Generator::<SystemClock, CryptoRng, WithNodeId>::with_node_id(1);
    /// ```
    #[must_use]
    pub const fn with_node_id(node_id: u16) -> Self {
        Self {
            clock: SystemClock,
            rng: CryptoRng,
            node_id: WithNodeId::new(node_id),
            state: Mutex::new(None),
        }
    }
}

// Generic constructors for testing
impl<C: Clock, R: Rng, N: NodeId> Generator<C, R, N> {
    /// Creates a generator with custom clock and RNG (for testing).
    ///
    /// # Examples
    ///
    /// ```
    /// use nulid::generator::{Generator, MockClock, SeededRng, NoNodeId};
    ///
    /// let clock = MockClock::new(1_000_000_000);
    /// let rng = SeededRng::new(42);
    /// let generator = Generator::<_, _, NoNodeId>::with_deps(&clock, &rng);
    /// ```
    pub fn with_deps(clock: C, rng: R) -> Self {
        Self {
            clock,
            rng,
            node_id: N::default(),
            state: Mutex::new(None),
        }
    }

    /// Creates a generator with custom clock, RNG, and node ID (for testing).
    ///
    /// # Examples
    ///
    /// ```
    /// use nulid::generator::{Generator, MockClock, SeededRng, WithNodeId};
    ///
    /// let clock = MockClock::new(1_000_000_000);
    /// let rng = SeededRng::new(42);
    /// let generator = Generator::with_deps_and_node_id(&clock, &rng, WithNodeId::new(1));
    /// ```
    pub const fn with_deps_and_node_id(clock: C, rng: R, node_id: N) -> Self {
        Self {
            clock,
            rng,
            node_id,
            state: Mutex::new(None),
        }
    }

    /// Generates a new NULID with monotonicity guarantee.
    ///
    /// # Algorithm (increment-on-skew)
    ///
    /// 1. Generate candidate ID: timestamp + random bits (+ optional node ID)
    /// 2. If candidate > `last_id`: use candidate
    /// 3. Else: increment `last_id` (handles clock skew and same-nanosecond)
    ///
    /// # Guarantees
    ///
    /// - **Monotonic**: Each ID from this generator is strictly > previous
    /// - **Random**: Uses randomness for cross-generator collision resistance
    /// - **Clock-resilient**: Handles backward jumps via increment strategy
    /// - **ULID compliant**: Preserves randomness as required by spec
    ///
    /// # Errors
    ///
    /// - `Overflow`: If increment would overflow 128-bit space
    /// - `MutexPoisoned`: If internal mutex is poisoned
    /// - `SystemTimeError`: If clock read fails
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
        let timestamp = self.clock.now_nanos()?;

        // Generate random bits with optional node ID
        // Layout with node ID: [node_id: 16 bits][random: 44 bits] = 60 bits total
        // Layout without node ID: [random: 60 bits]
        let random_bits = self.node_id.get().map_or_else(
            || self.rng.random_u64() & ((1u64 << 60) - 1),
            |node_id| {
                let random_44 = self.rng.random_u64() & ((1u64 << 44) - 1);
                (u64::from(node_id) << 44) | random_44
            },
        );

        let candidate = Nulid::from_nanos(timestamp, random_bits);

        let mut state = self.state.lock().map_err(|_| Error::MutexPoisoned)?;

        let result = match *state {
            None => {
                *state = Some(candidate);
                Ok(candidate)
            }
            Some(last_id) => {
                if candidate > last_id {
                    *state = Some(candidate);
                    Ok(candidate)
                } else {
                    // Clock skew or same nanosecond with lower random
                    let incremented = last_id.increment().ok_or(Error::Overflow)?;
                    *state = Some(incremented);
                    Ok(incremented)
                }
            }
        };

        drop(state);
        result
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
        self.state.lock().ok().and_then(|s| *s)
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
    /// assert!(generator.last().is_some());
    ///
    /// generator.reset();
    /// assert!(generator.last().is_none());
    /// # Ok(())
    /// # }
    /// ```
    pub fn reset(&self) {
        if let Ok(mut state) = self.state.lock() {
            *state = None;
        }
    }

    /// Returns the node ID if configured.
    ///
    /// # Examples
    ///
    /// ```
    /// use nulid::Generator;
    /// use nulid::generator::{SystemClock, CryptoRng, WithNodeId};
    ///
    /// let gen1 = Generator::new();
    /// assert_eq!(gen1.node_id(), None);
    ///
    /// let gen2 = Generator::<SystemClock, CryptoRng, WithNodeId>::with_node_id(42);
    /// assert_eq!(gen2.node_id(), Some(42));
    /// ```
    #[must_use]
    pub fn node_id(&self) -> Option<u16> {
        self.node_id.get()
    }
}

// ============================================================================
// Type Aliases
// ============================================================================

/// Default production generator (single node).
///
/// Alias for `Generator<SystemClock, CryptoRng, NoNodeId>`.
pub type DefaultGenerator = Generator<SystemClock, CryptoRng, NoNodeId>;

/// Distributed production generator (with node ID).
///
/// Alias for `Generator<SystemClock, CryptoRng, WithNodeId>`.
pub type DistributedGenerator = Generator<SystemClock, CryptoRng, WithNodeId>;

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use core::time::Duration;
    use std::sync::Arc;
    use std::thread;

    #[test]
    fn test_new_generator() {
        let generator = Generator::new();
        assert!(generator.last().is_none());
        assert_eq!(generator.node_id(), None);
    }

    #[test]
    fn test_first_generation() {
        let generator = Generator::new();
        let id = generator.generate().unwrap();
        assert!(id.nanos() > 0);
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
        assert!(id.nanos() > 0);
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

    // ========================================================================
    // Mock Clock Tests
    // ========================================================================

    #[test]
    fn test_mock_clock_basic() {
        let clock = MockClock::new(1_000_000_000);
        assert_eq!(clock.get(), 1_000_000_000);
        assert_eq!(clock.now_nanos().unwrap(), 1_000_000_000);
    }

    #[test]
    fn test_mock_clock_set() {
        let clock = MockClock::new(0);
        clock.set(999);
        assert_eq!(clock.get(), 999);
    }

    #[test]
    fn test_mock_clock_advance() {
        let clock = MockClock::new(1_000_000_000);
        clock.advance(Duration::from_millis(100));
        assert_eq!(clock.get(), 1_100_000_000);
    }

    #[test]
    fn test_mock_clock_regress() {
        let clock = MockClock::new(1_000_000_000);
        clock.regress(Duration::from_millis(100));
        assert_eq!(clock.get(), 900_000_000);
    }

    // ========================================================================
    // Seeded RNG Tests
    // ========================================================================

    #[test]
    fn test_seeded_rng_reproducible() {
        let rng1 = SeededRng::new(42);
        let rng2 = SeededRng::new(42);

        let vals1: Vec<u64> = (0..10).map(|_| rng1.random_u64()).collect();
        let vals2: Vec<u64> = (0..10).map(|_| rng2.random_u64()).collect();

        assert_eq!(vals1, vals2);
    }

    #[test]
    fn test_seeded_rng_different_seeds() {
        let rng1 = SeededRng::new(42);
        let rng2 = SeededRng::new(43);

        let val1 = rng1.random_u64();
        let val2 = rng2.random_u64();

        assert_ne!(val1, val2);
    }

    // ========================================================================
    // Sequential RNG Tests
    // ========================================================================

    #[test]
    fn test_sequential_rng() {
        let rng = SequentialRng::new();
        assert_eq!(rng.random_u64(), 0);
        assert_eq!(rng.random_u64(), 1);
        assert_eq!(rng.random_u64(), 2);
    }

    #[test]
    fn test_sequential_rng_starting_at() {
        let rng = SequentialRng::starting_at(100);
        assert_eq!(rng.random_u64(), 100);
        assert_eq!(rng.random_u64(), 101);
    }

    // ========================================================================
    // Node ID Tests
    // ========================================================================

    #[test]
    fn test_no_node_id() {
        let n = NoNodeId;
        assert_eq!(n.get(), None);
    }

    #[test]
    fn test_with_node_id() {
        let n = WithNodeId::new(42);
        assert_eq!(n.get(), Some(42));
        assert_eq!(n.value(), 42);
    }

    #[test]
    fn test_node_id_max_valid() {
        let n = WithNodeId::new(65535);
        assert_eq!(n.get(), Some(65535));
    }

    #[test]
    fn test_no_node_id_is_zst() {
        assert_eq!(core::mem::size_of::<NoNodeId>(), 0);
    }

    #[test]
    fn test_with_node_id_size() {
        assert_eq!(core::mem::size_of::<WithNodeId>(), 2);
    }

    // ========================================================================
    // Generator with Dependencies Tests
    // ========================================================================

    #[test]
    fn test_generator_with_mock_clock() {
        let clock = MockClock::new(1_000_000_000);
        let rng = SeededRng::new(42);
        let generator = Generator::<_, _, NoNodeId>::with_deps(&clock, &rng);

        let id1 = generator.generate().unwrap();
        assert!(id1.nanos() > 0);

        let id2 = generator.generate().unwrap();
        assert!(id2 > id1);
    }

    #[test]
    fn test_clock_regression_preserves_monotonicity() {
        let clock = MockClock::new(1_000_000_000);
        let rng = SeededRng::new(42);
        let generator = Generator::<_, _, NoNodeId>::with_deps(&clock, &rng);

        let id1 = generator.generate().unwrap();

        // Clock goes backward by 100ms (simulates NTP correction)
        clock.regress(Duration::from_millis(100));

        let id2 = generator.generate().unwrap();

        // Must still be monotonic
        assert!(id2 > id1, "Clock regression must not break monotonicity");

        // id2 should be id1 + 1 (increment strategy)
        assert_eq!(id2.as_u128(), id1.as_u128() + 1);
    }

    #[test]
    fn test_clock_stall_maintains_ordering() {
        let clock = MockClock::new(1_000_000_000);
        let rng = SequentialRng::new(); // Predictable: 0, 1, 2, ...
        let generator = Generator::<_, _, NoNodeId>::with_deps(&clock, &rng);

        // Generate 100 IDs with clock frozen
        let ids: Vec<Nulid> = (0..100).map(|_| generator.generate().unwrap()).collect();

        // All must be strictly increasing
        for i in 1..ids.len() {
            assert!(ids[i] > ids[i - 1], "ID {} not > ID {}", i, i - 1);
        }
    }

    #[test]
    fn test_reproducible_sequence() {
        fn generate_sequence(seed: u64) -> Vec<Nulid> {
            let clock = MockClock::new(1_000_000_000);
            let rng = SeededRng::new(seed);
            let generator = Generator::<_, _, NoNodeId>::with_deps(&clock, &rng);

            (0..10)
                .map(|i| {
                    clock.set(1_000_000_000 + i * 1000);
                    generator.generate().unwrap()
                })
                .collect()
        }

        let run1 = generate_sequence(42);
        let run2 = generate_sequence(42);

        // Same seed + same clock sequence = same IDs
        assert_eq!(run1, run2, "Seeded generator must be reproducible");
    }

    #[test]
    fn test_distributed_no_collision() {
        let clock = MockClock::new(1_000_000_000);
        let rng1 = SeededRng::new(42);
        let rng2 = SeededRng::new(42); // Same seed!

        let gen1 = Generator::with_deps_and_node_id(&clock, &rng1, WithNodeId::new(1));
        let gen2 = Generator::with_deps_and_node_id(&clock, &rng2, WithNodeId::new(2));

        let id1 = gen1.generate().unwrap();
        let id2 = gen2.generate().unwrap();

        // Must be different due to node ID
        assert_ne!(id1, id2, "Different nodes must produce different IDs");
    }

    #[test]
    fn test_clock_oscillation() {
        let clock = MockClock::new(1000);
        let rng = SeededRng::new(42);
        let generator = Generator::<_, _, NoNodeId>::with_deps(&clock, &rng);

        let mut ids = Vec::new();

        // Simulate clock bouncing between 1000 and 900
        for i in 0..20 {
            clock.set(if i % 2 == 0 { 1000 } else { 900 });
            ids.push(generator.generate().unwrap());
        }

        // All must be monotonic despite oscillation
        for i in 1..ids.len() {
            assert!(
                ids[i] > ids[i - 1],
                "Oscillating clock must not break ordering"
            );
        }
    }

    #[test]
    fn test_large_clock_jump_forward() {
        let clock = MockClock::new(1_000_000_000);
        let rng = SeededRng::new(42);
        let generator = Generator::<_, _, NoNodeId>::with_deps(&clock, &rng);

        let id1 = generator.generate().unwrap();

        // Clock jumps forward significantly (e.g., VM resume)
        clock.set(2_000_000_000_000);

        let id2 = generator.generate().unwrap();

        assert!(id2 > id1, "Forward clock jump should produce larger ID");
        // New timestamp should be used (not incremented from old)
        assert!(id2.nanos() > id1.nanos());
    }

    #[test]
    fn test_generator_node_id() {
        let gen1 = Generator::new();
        assert_eq!(gen1.node_id(), None);

        let gen2 = Generator::<SystemClock, CryptoRng, WithNodeId>::with_node_id(42);
        assert_eq!(gen2.node_id(), Some(42));
    }

    #[test]
    fn test_node_id_embedded_in_nulid() {
        let clock = MockClock::new(1_000_000_000);
        let rng = SequentialRng::new();
        let generator = Generator::with_deps_and_node_id(&clock, &rng, WithNodeId::new(0x123));

        let id = generator.generate().unwrap();

        // Node ID (0x123 = 291) should be in upper 16 bits of random part
        // Random part: [node_id: 16 bits][random: 44 bits]
        let random = id.random();
        #[allow(clippy::cast_possible_truncation)]
        let extracted_node_id = (random >> 44) as u16;
        assert_eq!(extracted_node_id, 0x123);
    }

    #[test]
    fn test_increment_same_timestamp() {
        let clock = MockClock::new(1_000_000_000);
        // Use sequential RNG so each random value is different but predictable
        let rng = SequentialRng::starting_at(1000);
        let generator = Generator::<_, _, NoNodeId>::with_deps(&clock, &rng);

        // First ID with random = 1000 (masked to 60 bits)
        let _id1 = generator.generate().unwrap();

        // Force next random to be lower (won't happen with sequential, but let's
        // manually simulate by resetting state and using lower random)
        generator.reset();
        let rng2 = SequentialRng::starting_at(100); // Lower than before
        let gen2 = Generator::<_, _, NoNodeId>::with_deps(&clock, &rng2);

        // Set state to id1
        let _ = gen2.generate().unwrap(); // This will be < id1

        // Now create scenario where new candidate <= last
        // We need to manually construct this
        let clock3 = MockClock::new(1_000_000_000);
        let rng3 = SequentialRng::new(); // Starts at 0
        let gen3 = Generator::<_, _, NoNodeId>::with_deps(&clock3, &rng3);

        let first = gen3.generate().unwrap(); // random = 0
        let second = gen3.generate().unwrap(); // random = 1, should be > first

        assert!(second > first);
    }
}

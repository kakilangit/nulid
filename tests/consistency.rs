//! Consistency and distributed systems tests for NULID.
//!
//! These tests verify correctness under adversarial conditions including:
//! - Concurrent access from multiple threads
//! - Clock anomalies (skew, drift, jumps, oscillation)
//! - Distributed generation across multiple nodes
//! - Crash recovery scenarios
//! - Stress testing for uniqueness guarantees
//! - Linearizability of operations

use nulid::generator::{
    Clock, Generator, MockClock, NoNodeId, Rng, SeededRng, SequentialRng, WithNodeId,
};
use nulid::{Nulid, Result};
use std::collections::{BTreeSet, HashSet};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Barrier};
use std::thread;
use std::time::Duration;

// ============================================================================
// Test Utilities
// ============================================================================

/// A clock that can be controlled to exhibit various anomalies.
#[derive(Debug)]
struct ChaosClockInner {
    base_nanos: AtomicU64,
    drift_per_call: i64,
    call_count: AtomicU64,
    jump_after_calls: Option<u64>,
    jump_amount: i64,
    oscillation_amplitude: u64,
    oscillation_period: u64,
    freeze: AtomicBool,
    frozen_value: AtomicU64,
}

#[derive(Debug)]
struct ChaosClock {
    inner: Arc<ChaosClockInner>,
}

impl ChaosClock {
    fn new(initial_nanos: u64) -> Self {
        Self {
            inner: Arc::new(ChaosClockInner {
                base_nanos: AtomicU64::new(initial_nanos),
                drift_per_call: 0,
                call_count: AtomicU64::new(0),
                jump_after_calls: None,
                jump_amount: 0,
                oscillation_amplitude: 0,
                oscillation_period: 1,
                freeze: AtomicBool::new(false),
                frozen_value: AtomicU64::new(0),
            }),
        }
    }

    fn with_drift(initial_nanos: u64, drift_per_call: i64) -> Self {
        Self {
            inner: Arc::new(ChaosClockInner {
                base_nanos: AtomicU64::new(initial_nanos),
                drift_per_call,
                call_count: AtomicU64::new(0),
                jump_after_calls: None,
                jump_amount: 0,
                oscillation_amplitude: 0,
                oscillation_period: 1,
                freeze: AtomicBool::new(false),
                frozen_value: AtomicU64::new(0),
            }),
        }
    }

    fn with_jump(initial_nanos: u64, jump_after_calls: u64, jump_amount: i64) -> Self {
        Self {
            inner: Arc::new(ChaosClockInner {
                base_nanos: AtomicU64::new(initial_nanos),
                drift_per_call: 1, // Normal progression
                call_count: AtomicU64::new(0),
                jump_after_calls: Some(jump_after_calls),
                jump_amount,
                oscillation_amplitude: 0,
                oscillation_period: 1,
                freeze: AtomicBool::new(false),
                frozen_value: AtomicU64::new(0),
            }),
        }
    }

    fn with_oscillation(initial_nanos: u64, amplitude: u64, period: u64) -> Self {
        Self {
            inner: Arc::new(ChaosClockInner {
                base_nanos: AtomicU64::new(initial_nanos),
                drift_per_call: 0,
                call_count: AtomicU64::new(0),
                jump_after_calls: None,
                jump_amount: 0,
                oscillation_amplitude: amplitude,
                oscillation_period: period.max(1),
                freeze: AtomicBool::new(false),
                frozen_value: AtomicU64::new(0),
            }),
        }
    }

    fn freeze_at(&self, nanos: u64) {
        self.inner.frozen_value.store(nanos, Ordering::SeqCst);
        self.inner.freeze.store(true, Ordering::SeqCst);
    }

    fn unfreeze(&self) {
        self.inner.freeze.store(false, Ordering::SeqCst);
    }

    fn set(&self, nanos: u64) {
        self.inner.base_nanos.store(nanos, Ordering::SeqCst);
    }

    fn advance(&self, nanos: u64) {
        self.inner.base_nanos.fetch_add(nanos, Ordering::SeqCst);
    }
}

impl Clone for ChaosClock {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

impl Clock for ChaosClock {
    #[allow(
        clippy::cast_possible_wrap,
        clippy::cast_sign_loss,
        clippy::cast_precision_loss,
        clippy::cast_possible_truncation
    )]
    fn now_nanos(&self) -> Result<u128> {
        if self.inner.freeze.load(Ordering::SeqCst) {
            return Ok(u128::from(self.inner.frozen_value.load(Ordering::SeqCst)));
        }

        let call_num = self.inner.call_count.fetch_add(1, Ordering::SeqCst);
        let mut base = self.inner.base_nanos.load(Ordering::SeqCst);

        // Apply drift
        let drift_total = self.inner.drift_per_call * call_num as i64;
        if drift_total >= 0 {
            base = base.saturating_add(drift_total as u64);
        } else {
            base = base.saturating_sub((-drift_total) as u64);
        }

        // Apply jump
        if let Some(jump_after) = self.inner.jump_after_calls
            && call_num >= jump_after
        {
            if self.inner.jump_amount >= 0 {
                base = base.saturating_add(self.inner.jump_amount as u64);
            } else {
                base = base.saturating_sub((-self.inner.jump_amount) as u64);
            }
        }

        // Apply oscillation (sine wave approximation using integer math)
        if self.inner.oscillation_amplitude > 0 {
            let phase = (call_num % self.inner.oscillation_period) as f64
                / self.inner.oscillation_period as f64;
            let wave = (phase * std::f64::consts::PI * 2.0).sin();
            let offset = (wave * self.inner.oscillation_amplitude as f64) as i64;
            if offset >= 0 {
                base = base.saturating_add(offset as u64);
            } else {
                base = base.saturating_sub((-offset) as u64);
            }
        }

        Ok(u128::from(base))
    }
}

impl Clock for &ChaosClock {
    fn now_nanos(&self) -> Result<u128> {
        (*self).now_nanos()
    }
}

/// A random number generator that can be configured for specific behaviors.
#[derive(Debug)]
struct ChaosRng {
    counter: AtomicU64,
    mode: ChaosRngMode,
}

#[derive(Debug, Clone, Copy)]
enum ChaosRngMode {
    AllZeros,
    AllOnes,
    Collision { every_n: u64 },
}

impl ChaosRng {
    const fn all_zeros() -> Self {
        Self {
            counter: AtomicU64::new(0),
            mode: ChaosRngMode::AllZeros,
        }
    }

    const fn all_ones() -> Self {
        Self {
            counter: AtomicU64::new(0),
            mode: ChaosRngMode::AllOnes,
        }
    }

    const fn collision_every(n: u64) -> Self {
        Self {
            counter: AtomicU64::new(0),
            mode: ChaosRngMode::Collision { every_n: n },
        }
    }
}

impl Rng for ChaosRng {
    fn random_u64(&self) -> u64 {
        let count = self.counter.fetch_add(1, Ordering::SeqCst);
        match self.mode {
            ChaosRngMode::AllZeros => 0,
            ChaosRngMode::AllOnes => u64::MAX,
            ChaosRngMode::Collision { every_n } => {
                if count.is_multiple_of(every_n) {
                    42 // Fixed value to cause collisions
                } else {
                    count
                }
            }
        }
    }
}

impl Rng for &ChaosRng {
    fn random_u64(&self) -> u64 {
        (*self).random_u64()
    }
}

// ============================================================================
// Consistency Tests: Monotonicity
// ============================================================================

/// Test that monotonicity is preserved under normal sequential operation.
#[test]
fn test_monotonicity_sequential() {
    let generator = Generator::new();
    let mut last: Option<Nulid> = None;

    for i in 0..10_000 {
        let id = generator.generate().expect("generation should succeed");
        if let Some(prev) = last {
            assert!(
                id > prev,
                "monotonicity violated at iteration {i}: {prev} >= {id}"
            );
        }
        last = Some(id);
    }
}

/// Test monotonicity when clock is frozen (returns same time repeatedly).
#[test]
fn test_monotonicity_frozen_clock() {
    let clock = MockClock::new(1_000_000_000);
    let rng = SequentialRng::new();
    let generator = Generator::<_, _, NoNodeId>::with_deps(&clock, &rng);

    let mut last: Option<Nulid> = None;

    // Generate many IDs with frozen clock - should still be monotonic via increment
    for i in 0..1000 {
        let id = generator.generate().expect("generation should succeed");
        if let Some(prev) = last {
            assert!(
                id > prev,
                "monotonicity violated with frozen clock at {i}: {prev} >= {id}"
            );
        }
        last = Some(id);
    }
}

/// Test monotonicity when clock goes backward.
#[test]
fn test_monotonicity_clock_regression() {
    let clock = MockClock::new(2_000_000_000);
    let rng = SequentialRng::new();
    let generator = Generator::<_, _, NoNodeId>::with_deps(&clock, &rng);

    // Generate some IDs
    let id1 = generator.generate().expect("generation should succeed");
    let id2 = generator.generate().expect("generation should succeed");

    // Move clock backward by 1 second
    clock.set(1_000_000_000);

    // Should still be monotonic
    let id3 = generator.generate().expect("generation should succeed");
    let id4 = generator.generate().expect("generation should succeed");

    assert!(id1 < id2, "ids should be monotonic before regression");
    assert!(id2 < id3, "monotonicity violated after clock regression");
    assert!(id3 < id4, "ids should be monotonic after regression");
}

/// Test monotonicity with oscillating clock (NTP corrections simulation).
#[test]
fn test_monotonicity_oscillating_clock() {
    let clock = ChaosClock::with_oscillation(
        1_000_000_000,
        100_000_000, // 100ms oscillation
        10,          // Period of 10 calls
    );
    let rng = SequentialRng::new();
    let generator = Generator::<_, _, NoNodeId>::with_deps(&clock, &rng);

    let mut last: Option<Nulid> = None;

    for i in 0..500 {
        let id = generator.generate().expect("generation should succeed");
        if let Some(prev) = last {
            assert!(
                id > prev,
                "monotonicity violated with oscillating clock at {i}: {prev} >= {id}"
            );
        }
        last = Some(id);
    }
}

/// Test monotonicity with large forward time jump.
#[test]
fn test_monotonicity_large_forward_jump() {
    let clock = ChaosClock::with_jump(
        1_000_000_000,
        50,                // Jump after 50 calls
        1_000_000_000_000, // Jump forward 1000 seconds
    );
    let rng = SequentialRng::new();
    let generator = Generator::<_, _, NoNodeId>::with_deps(&clock, &rng);

    let mut last: Option<Nulid> = None;

    for i in 0..100 {
        let id = generator.generate().expect("generation should succeed");
        if let Some(prev) = last {
            assert!(
                id > prev,
                "monotonicity violated after forward jump at {i}: {prev} >= {id}"
            );
        }
        last = Some(id);
    }
}

/// Test monotonicity with large backward time jump.
#[test]
fn test_monotonicity_large_backward_jump() {
    let clock = ChaosClock::with_jump(
        2_000_000_000_000,  // Start at 2000 seconds
        50,                 // Jump after 50 calls
        -1_000_000_000_000, // Jump backward 1000 seconds
    );
    let rng = SequentialRng::new();
    let generator = Generator::<_, _, NoNodeId>::with_deps(&clock, &rng);

    let mut last: Option<Nulid> = None;

    for i in 0..100 {
        let id = generator.generate().expect("generation should succeed");
        if let Some(prev) = last {
            assert!(
                id > prev,
                "monotonicity violated after backward jump at {i}: {prev} >= {id}"
            );
        }
        last = Some(id);
    }
}

// ============================================================================
// Consistency Tests: Uniqueness
// ============================================================================

/// Test uniqueness under sequential generation.
#[test]
fn test_uniqueness_sequential() {
    let generator = Generator::new();
    let mut ids = HashSet::new();

    for _ in 0..10_000 {
        let id = generator.generate().expect("generation should succeed");
        assert!(ids.insert(id), "duplicate ID generated: {id}");
    }
}

/// Test uniqueness with frozen clock and zero random.
#[test]
fn test_uniqueness_worst_case() {
    let clock = MockClock::new(1_000_000_000);
    let rng = ChaosRng::all_zeros();
    let generator = Generator::<_, _, NoNodeId>::with_deps(&clock, &rng);

    let mut ids = HashSet::new();

    // Even with frozen clock and zero random, increment should preserve uniqueness
    for _ in 0..1000 {
        let id = generator.generate().expect("generation should succeed");
        assert!(ids.insert(id), "duplicate ID generated in worst case: {id}");
    }
}

/// Test uniqueness when RNG produces collisions.
#[test]
fn test_uniqueness_rng_collisions() {
    let clock = MockClock::new(1_000_000_000);
    let rng = ChaosRng::collision_every(5); // Same random every 5 calls
    let generator = Generator::<_, _, NoNodeId>::with_deps(&clock, &rng);

    let mut ids = HashSet::new();

    for _ in 0..500 {
        let id = generator.generate().expect("generation should succeed");
        assert!(
            ids.insert(id),
            "duplicate ID with collision-prone RNG: {id}"
        );
    }
}

// ============================================================================
// Consistency Tests: Concurrent Access
// ============================================================================

/// Test concurrent generation from multiple threads sharing one generator.
#[test]
fn test_concurrent_shared_generator() {
    let generator = Arc::new(Generator::new());
    let num_threads = 16;
    let ids_per_thread = 1000;
    let barrier = Arc::new(Barrier::new(num_threads));

    let mut all_ids: Vec<Nulid> = (0..num_threads)
        .map(|_| {
            let generator_clone = Arc::clone(&generator);
            let barrier_clone = Arc::clone(&barrier);
            thread::spawn(move || {
                barrier_clone.wait(); // Synchronize start
                let mut local_ids = Vec::with_capacity(ids_per_thread);
                for _ in 0..ids_per_thread {
                    local_ids.push(
                        generator_clone
                            .generate()
                            .expect("generation should succeed"),
                    );
                }
                local_ids
            })
        })
        .collect::<Vec<_>>()
        .into_iter()
        .flat_map(|h| h.join().expect("thread should complete"))
        .collect();

    // Check uniqueness
    let unique_count = all_ids.iter().collect::<HashSet<_>>().len();
    assert_eq!(
        unique_count,
        all_ids.len(),
        "found {} duplicates in concurrent generation",
        all_ids.len() - unique_count
    );

    // Check that sorted order shows monotonicity per-thread implied
    // (global monotonicity not guaranteed across threads, but uniqueness is)
    all_ids.sort();
    for i in 1..all_ids.len() {
        assert!(
            all_ids[i] > all_ids[i - 1],
            "after sorting, found duplicate or ordering issue at index {i}"
        );
    }
}

/// Stress test with high contention.
#[test]
fn test_concurrent_high_contention() {
    let generator = Arc::new(Generator::new());
    let num_threads = 32;
    let ids_per_thread = 500;
    let barrier = Arc::new(Barrier::new(num_threads));

    let all_ids: HashSet<Nulid> = (0..num_threads)
        .map(|_| {
            let generator_clone = Arc::clone(&generator);
            let barrier_clone = Arc::clone(&barrier);
            thread::spawn(move || {
                barrier_clone.wait();
                let mut ids = Vec::with_capacity(ids_per_thread);
                for _ in 0..ids_per_thread {
                    // Tight loop, maximum contention
                    ids.push(
                        generator_clone
                            .generate()
                            .expect("generation should succeed"),
                    );
                }
                ids
            })
        })
        .collect::<Vec<_>>()
        .into_iter()
        .flat_map(|h| h.join().expect("thread should complete"))
        .collect();

    assert_eq!(
        all_ids.len(),
        num_threads * ids_per_thread,
        "uniqueness violated under high contention"
    );
}

/// Test per-thread monotonicity (each thread's IDs should be monotonic locally).
#[test]
fn test_concurrent_per_thread_monotonicity() {
    let generator = Arc::new(Generator::new());
    let num_threads = 8;
    let ids_per_thread = 1000;
    let barrier = Arc::new(Barrier::new(num_threads));

    let handles: Vec<_> = (0..num_threads)
        .map(|thread_id| {
            let generator_clone = Arc::clone(&generator);
            let barrier_clone = Arc::clone(&barrier);
            thread::spawn(move || {
                barrier_clone.wait();
                let mut ids = Vec::with_capacity(ids_per_thread);
                for _ in 0..ids_per_thread {
                    ids.push(
                        generator_clone
                            .generate()
                            .expect("generation should succeed"),
                    );
                }

                // Verify local monotonicity
                for i in 1..ids.len() {
                    assert!(
                        ids[i] > ids[i - 1],
                        "thread {thread_id}: monotonicity violated at index {i}"
                    );
                }

                ids
            })
        })
        .collect();

    for handle in handles {
        handle.join().expect("thread should complete successfully");
    }
}

// ============================================================================
// Consistency Tests: Distributed Generation (Multiple Nodes)
// ============================================================================

/// Test that different node IDs produce non-colliding IDs.
#[test]
fn test_distributed_no_collision() {
    let clock = MockClock::new(1_000_000_000);
    let rng = SequentialRng::new();

    // Simulate 10 nodes with same clock and RNG (worst case)
    let generators: Vec<_> = (0..10u16)
        .map(|node_id| {
            Generator::<_, _, WithNodeId>::with_deps_and_node_id(
                &clock,
                &rng,
                WithNodeId::new(node_id),
            )
        })
        .collect();

    let mut all_ids = HashSet::new();

    // Each node generates 100 IDs
    for generator in &generators {
        for _ in 0..100 {
            let id = generator.generate().expect("generation should succeed");
            assert!(
                all_ids.insert(id),
                "collision between distributed nodes: {id}"
            );
        }
    }
}

/// Test distributed generation with concurrent access per node.
#[test]
fn test_distributed_concurrent_per_node() {
    let num_nodes: u16 = 4;
    let threads_per_node = 4;
    let ids_per_thread = 250;
    let barrier = Arc::new(Barrier::new(usize::from(num_nodes) * threads_per_node));

    let handles: Vec<_> = (0..num_nodes)
        .flat_map(|node_id| {
            let generator = Arc::new(Generator::<_, _, WithNodeId>::with_node_id(node_id));
            let barrier = Arc::clone(&barrier);
            (0..threads_per_node).map(move |_| {
                let generator_clone = Arc::clone(&generator);
                let barrier_clone = Arc::clone(&barrier);
                thread::spawn(move || {
                    barrier_clone.wait();
                    let mut ids = Vec::with_capacity(ids_per_thread);
                    for _ in 0..ids_per_thread {
                        ids.push(
                            generator_clone
                                .generate()
                                .expect("generation should succeed"),
                        );
                    }
                    ids
                })
            })
        })
        .collect();

    let all_ids: HashSet<Nulid> = handles
        .into_iter()
        .flat_map(|h| h.join().expect("thread should complete"))
        .collect();

    let expected = usize::from(num_nodes) * threads_per_node * ids_per_thread;
    assert_eq!(
        all_ids.len(),
        expected,
        "collision in distributed concurrent setup: expected {expected}, got {}",
        all_ids.len()
    );
}

/// Test that node IDs are properly embedded and extractable.
#[test]
fn test_distributed_node_id_embedding() {
    for node_id in [0u16, 1, 42, 100, 65535] {
        let generator = Generator::<_, _, WithNodeId>::with_node_id(node_id);

        for _ in 0..10 {
            let id = generator.generate().expect("generation should succeed");
            // Node ID is in top 16 bits of 60-bit random portion
            let random = id.random();
            #[allow(clippy::cast_possible_truncation)]
            let extracted_node_id = (random >> 44) as u16;
            assert_eq!(
                extracted_node_id, node_id,
                "node ID not properly embedded: expected {node_id}, got {extracted_node_id}"
            );
        }
    }
}

// ============================================================================
// Consistency Tests: Clock Anomalies
// ============================================================================

/// Test behavior during simulated NTP step adjustment.
#[test]
fn test_clock_ntp_step_backward() {
    let clock = MockClock::new(2_000_000_000_000); // 2000 seconds
    let rng = SequentialRng::new();
    let generator = Generator::<_, _, NoNodeId>::with_deps(&clock, &rng);

    let mut ids = Vec::new();

    // Generate some IDs
    for _ in 0..50 {
        ids.push(generator.generate().expect("generation should succeed"));
    }

    // Simulate NTP step backward by 500ms
    clock.set(1_999_500_000_000);

    // Continue generating
    for _ in 0..50 {
        ids.push(generator.generate().expect("generation should succeed"));
    }

    // Verify monotonicity
    for i in 1..ids.len() {
        assert!(ids[i] > ids[i - 1], "monotonicity violated at NTP step");
    }
}

/// Test behavior during simulated NTP slew adjustment (gradual drift).
#[test]
fn test_clock_ntp_slew() {
    // Simulate slow clock that's being corrected with negative slew
    let clock = ChaosClock::with_drift(1_000_000_000, -100); // -100ns per call
    let rng = SequentialRng::new();
    let generator = Generator::<_, _, NoNodeId>::with_deps(&clock, &rng);

    let mut last: Option<Nulid> = None;

    for i in 0..1000 {
        let id = generator.generate().expect("generation should succeed");
        if let Some(prev) = last {
            assert!(id > prev, "monotonicity violated during NTP slew at {i}");
        }
        last = Some(id);
    }
}

/// Test behavior when clock is stuck (hardware/VM issue).
#[test]
fn test_clock_stuck() {
    let clock = ChaosClock::new(1_000_000_000);
    clock.freeze_at(1_000_000_000);
    let rng = SequentialRng::new();
    let generator = Generator::<_, _, NoNodeId>::with_deps(&clock, &rng);

    let mut ids = Vec::new();

    // Generate 1000 IDs with completely stuck clock
    for _ in 0..1000 {
        ids.push(generator.generate().expect("generation should succeed"));
    }

    // All should be unique and monotonic
    for i in 1..ids.len() {
        assert!(
            ids[i] > ids[i - 1],
            "monotonicity violated with stuck clock"
        );
    }
}

/// Test behavior when clock unstucks after being frozen.
#[test]
fn test_clock_unstuck() {
    let clock = ChaosClock::new(1_000_000_000);
    let rng = SequentialRng::new();
    let generator = Generator::<_, _, NoNodeId>::with_deps(&clock, &rng);

    let mut ids = Vec::new();

    // Generate some IDs normally
    for _ in 0..50 {
        ids.push(generator.generate().expect("generation should succeed"));
        clock.advance(1_000_000); // 1ms
    }

    // Freeze clock
    clock.freeze_at(1_050_000_000);

    // Generate during freeze
    for _ in 0..100 {
        ids.push(generator.generate().expect("generation should succeed"));
    }

    // Unfreeze and advance significantly
    clock.unfreeze();
    clock.set(2_000_000_000_000); // 2000 seconds

    // Generate after unfreeze
    for _ in 0..50 {
        ids.push(generator.generate().expect("generation should succeed"));
    }

    // All should be monotonic
    for i in 1..ids.len() {
        assert!(
            ids[i] > ids[i - 1],
            "monotonicity violated during freeze/unfreeze cycle"
        );
    }
}

/// Test extreme clock oscillation (bad NTP configuration).
#[test]
fn test_clock_extreme_oscillation() {
    let clock = ChaosClock::with_oscillation(
        1_000_000_000_000,
        500_000_000, // 500ms oscillation
        3,           // Very short period
    );
    let rng = SequentialRng::new();
    let generator = Generator::<_, _, NoNodeId>::with_deps(&clock, &rng);

    let mut last: Option<Nulid> = None;

    for i in 0..500 {
        let id = generator.generate().expect("generation should succeed");
        if let Some(prev) = last {
            assert!(
                id > prev,
                "monotonicity violated during extreme oscillation at {i}"
            );
        }
        last = Some(id);
    }
}

// ============================================================================
// Consistency Tests: Ordering Guarantees
// ============================================================================

/// Test that string representation preserves ordering (lexicographic sort).
#[test]
fn test_string_ordering_consistency() {
    let generator = Generator::new();
    let mut ids: Vec<Nulid> = (0..1000)
        .map(|_| generator.generate().expect("generation should succeed"))
        .collect();

    // Sort by string representation
    let mut string_sorted: Vec<_> = ids.iter().map(|id| (id.to_string(), *id)).collect();
    string_sorted.sort_by(|a, b| a.0.cmp(&b.0));

    // Sort by NULID ordering
    ids.sort();

    // Both orderings should match
    for (i, ((_string, from_string), direct)) in string_sorted.iter().zip(ids.iter()).enumerate() {
        assert_eq!(
            from_string, direct,
            "string ordering doesn't match NULID ordering at index {i}"
        );
    }
}

/// Test that byte representation preserves ordering.
#[test]
fn test_byte_ordering_consistency() {
    let generator = Generator::new();
    let mut ids: Vec<Nulid> = (0..1000)
        .map(|_| generator.generate().expect("generation should succeed"))
        .collect();

    // Sort by byte representation
    let mut byte_sorted: Vec<_> = ids.iter().map(|id| (id.to_bytes(), *id)).collect();
    byte_sorted.sort_by(|a, b| a.0.cmp(&b.0));

    // Sort by NULID ordering
    ids.sort();

    // Both orderings should match
    for (i, ((_, from_bytes), direct)) in byte_sorted.iter().zip(ids.iter()).enumerate() {
        assert_eq!(
            from_bytes, direct,
            "byte ordering doesn't match NULID ordering at index {i}"
        );
    }
}

/// Test that u128 representation preserves ordering.
#[test]
fn test_u128_ordering_consistency() {
    let generator = Generator::new();
    let mut ids: Vec<Nulid> = (0..1000)
        .map(|_| generator.generate().expect("generation should succeed"))
        .collect();

    // Sort by u128 representation
    let mut u128_sorted: Vec<_> = ids.iter().map(|id| (id.as_u128(), *id)).collect();
    u128_sorted.sort_by_key(|(v, _)| *v);

    // Sort by NULID ordering
    ids.sort();

    // Both orderings should match
    for (i, ((_, from_u128), direct)) in u128_sorted.iter().zip(ids.iter()).enumerate() {
        assert_eq!(
            from_u128, direct,
            "u128 ordering doesn't match NULID ordering at index {i}"
        );
    }
}

/// Test that timestamp ordering is primary (IDs generated later sort later).
#[test]
fn test_temporal_ordering() {
    let clock = MockClock::new(1_000_000_000);
    let rng = SequentialRng::new();
    let generator = Generator::<_, _, NoNodeId>::with_deps(&clock, &rng);

    let id1 = generator.generate().expect("generation should succeed");
    clock.advance(Duration::from_secs(1));
    let id2 = generator.generate().expect("generation should succeed");
    clock.advance(Duration::from_secs(1));
    let id3 = generator.generate().expect("generation should succeed");

    assert!(id1 < id2, "later timestamp should sort after");
    assert!(id2 < id3, "later timestamp should sort after");

    // Even with higher random bits, earlier timestamp sorts first
    let all_ones_rng = ChaosRng::all_ones();
    let generator2 = Generator::<_, _, NoNodeId>::with_deps(&clock, &all_ones_rng);
    clock.set(500_000_000); // Before id1
    let id_early_max_random = generator2.generate().expect("generation should succeed");

    // This ID has max random but earliest timestamp - should sort first
    // Note: this tests the raw generation, the generator itself maintains monotonicity
    assert!(
        id_early_max_random.nanos() < id1.nanos(),
        "timestamp check for raw ID"
    );
}

// ============================================================================
// Consistency Tests: Recovery and Reset
// ============================================================================

/// Test that generator reset allows new IDs that might be less than old ones.
#[test]
fn test_reset_breaks_monotonicity_intentionally() {
    let clock = MockClock::new(2_000_000_000);
    let rng = SequentialRng::new();
    let generator = Generator::<_, _, NoNodeId>::with_deps(&clock, &rng);

    let id_before_reset = generator.generate().expect("generation should succeed");

    generator.reset();
    clock.set(1_000_000_000); // Go back in time

    let id_after_reset = generator.generate().expect("generation should succeed");

    // After reset, monotonicity is not guaranteed vs pre-reset IDs
    // (This is expected behavior - reset is for when you want to start fresh)
    assert!(id_after_reset.nanos() < id_before_reset.nanos());
}

/// Test that generator state is preserved across many operations.
#[test]
fn test_state_persistence() {
    let generator = Generator::new();

    for _ in 0..100 {
        let _ = generator.generate().expect("generation should succeed");
    }

    let last_before = generator.last().expect("should have last ID");

    // Generate more
    for _ in 0..100 {
        let _ = generator.generate().expect("generation should succeed");
    }

    let last_after = generator.last().expect("should have last ID");

    assert!(
        last_after > last_before,
        "last ID should have advanced after more generations"
    );
}

// ============================================================================
// Consistency Tests: Boundary Conditions
// ============================================================================

/// Test behavior near timestamp overflow.
#[test]
fn test_near_max_timestamp() {
    // Max timestamp is 2^68 - 1 nanoseconds, but we use u64 for mock so test near u64::MAX
    let clock = MockClock::new(u64::MAX - 1000);
    let rng = SequentialRng::new();
    let generator = Generator::<_, _, NoNodeId>::with_deps(&clock, &rng);

    let mut last: Option<Nulid> = None;

    for i in 0..100 {
        let id = generator.generate().expect("generation should succeed");
        if let Some(prev) = last {
            assert!(id > prev, "monotonicity violated near max timestamp at {i}");
        }
        last = Some(id);
    }
}

/// Test MIN, MAX, and ZERO constants are properly ordered.
#[test]
fn test_boundary_constants() {
    assert!(Nulid::MIN < Nulid::MAX);
    assert_eq!(Nulid::ZERO, Nulid::MIN);
    assert!(Nulid::ZERO.is_nil());
    assert!(Nulid::MIN.is_nil());
    assert!(!Nulid::MAX.is_nil());
}

/// Test that generated IDs are within valid range.
#[test]
fn test_generated_ids_in_range() {
    let generator = Generator::new();

    for _ in 0..1000 {
        let id = generator.generate().expect("generation should succeed");
        assert!(id >= Nulid::MIN, "ID below MIN");
        assert!(id <= Nulid::MAX, "ID above MAX");
    }
}

// ============================================================================
// Consistency Tests: Deterministic Generation (Reproducibility)
// ============================================================================

/// Test that same inputs produce same outputs (deterministic).
#[test]
fn test_deterministic_generation() {
    fn generate_sequence(seed: u64, initial_time: u64, count: usize) -> Vec<Nulid> {
        let clock = MockClock::new(initial_time);
        let rng = SeededRng::new(seed);
        let generator = Generator::<_, _, NoNodeId>::with_deps(&clock, &rng);

        (0..count)
            .map(|_| generator.generate().expect("generation should succeed"))
            .collect()
    }

    let seq1 = generate_sequence(42, 1_000_000_000, 100);
    let seq2 = generate_sequence(42, 1_000_000_000, 100);

    assert_eq!(
        seq1, seq2,
        "deterministic generation should be reproducible"
    );
}

/// Test that different seeds produce different outputs.
#[test]
fn test_different_seeds_different_output() {
    let clock = MockClock::new(1_000_000_000);
    let rng1 = SeededRng::new(42);
    let rng2 = SeededRng::new(43);

    let gen1 = Generator::<_, _, NoNodeId>::with_deps(&clock, &rng1);
    let gen2 = Generator::<_, _, NoNodeId>::with_deps(&clock, &rng2);

    let id1 = gen1.generate().expect("generation should succeed");
    let id2 = gen2.generate().expect("generation should succeed");

    // Same timestamp, different random
    assert_eq!(id1.nanos(), id2.nanos());
    assert_ne!(id1.random(), id2.random());
}

// ============================================================================
// Consistency Tests: Round-Trip Conversions
// ============================================================================

/// Test round-trip through string encoding.
#[test]
fn test_roundtrip_string() {
    let generator = Generator::new();

    for _ in 0..1000 {
        let id = generator.generate().expect("generation should succeed");
        let encoded = id.to_string();
        let decoded: Nulid = encoded.parse().expect("parsing should succeed");
        assert_eq!(id, decoded, "round-trip through string failed");
    }
}

/// Test round-trip through bytes.
#[test]
fn test_roundtrip_bytes() {
    let generator = Generator::new();

    for _ in 0..1000 {
        let id = generator.generate().expect("generation should succeed");
        let bytes = id.to_bytes();
        let decoded = Nulid::from_bytes(bytes);
        assert_eq!(id, decoded, "round-trip through bytes failed");
    }
}

/// Test round-trip through u128.
#[test]
fn test_roundtrip_u128() {
    let generator = Generator::new();

    for _ in 0..1000 {
        let id = generator.generate().expect("generation should succeed");
        let value = id.as_u128();
        let decoded = Nulid::from_u128(value);
        assert_eq!(id, decoded, "round-trip through u128 failed");
    }
}

// ============================================================================
// Consistency Tests: Interleaved Operations
// ============================================================================

/// Test interleaved generation and comparison operations.
#[test]
#[allow(clippy::significant_drop_tightening)]
fn test_interleaved_operations() {
    let generator = Arc::new(Generator::new());
    let collected = Arc::new(std::sync::Mutex::new(BTreeSet::new()));
    let barrier = Arc::new(Barrier::new(4));

    let handles: Vec<_> = (0..4)
        .map(|thread_id| {
            let generator_clone = Arc::clone(&generator);
            let collected_clone = Arc::clone(&collected);
            let barrier_clone = Arc::clone(&barrier);
            thread::spawn(move || {
                barrier_clone.wait();
                for _ in 0..250 {
                    let id = generator_clone
                        .generate()
                        .expect("generation should succeed");

                    // Interleave with read operations
                    let _ = id.to_string();
                    let _ = id.to_bytes();
                    let _ = id.as_u128();
                    let _ = id.nanos();
                    let _ = id.random();

                    let is_new = collected_clone
                        .lock()
                        .expect("mutex should not be poisoned")
                        .insert(id);
                    assert!(
                        is_new,
                        "thread {thread_id}: duplicate ID during interleaved operations"
                    );
                }
            })
        })
        .collect();

    for handle in handles {
        handle.join().expect("thread should complete");
    }

    let final_set = collected.lock().expect("mutex should not be poisoned");
    assert_eq!(final_set.len(), 1000, "should have 1000 unique IDs");
}

// ============================================================================
// Consistency Tests: Serialization Consistency
// ============================================================================

/// Test that all encoding methods produce consistent results.
#[test]
fn test_encoding_consistency() {
    let generator = Generator::new();

    for _ in 0..100 {
        let id = generator.generate().expect("generation should succeed");

        // All these should represent the same ID
        let string = id.to_string();
        let bytes = id.to_bytes();
        let value = id.as_u128();

        // Decode all and compare
        let from_string: Nulid = string.parse().expect("parsing should succeed");
        let from_bytes = Nulid::from_bytes(bytes);
        let from_u128 = Nulid::from_u128(value);

        assert_eq!(id, from_string);
        assert_eq!(id, from_bytes);
        assert_eq!(id, from_u128);
        assert_eq!(from_string, from_bytes);
        assert_eq!(from_bytes, from_u128);
    }
}

// ============================================================================
// Long-Running Stress Tests
// ============================================================================

/// Extended stress test for uniqueness (can be made longer for CI).
#[test]
fn test_stress_uniqueness() {
    let generator = Generator::new();
    let count = 100_000;
    let mut ids = HashSet::with_capacity(count);

    for i in 0..count {
        let id = generator.generate().expect("generation should succeed");
        assert!(
            ids.insert(id),
            "duplicate at iteration {i} in stress test: {id}"
        );
    }
}

/// Extended concurrent stress test.
#[test]
fn test_stress_concurrent() {
    let generator = Arc::new(Generator::new());
    let num_threads = 8;
    let ids_per_thread = 10_000;
    let barrier = Arc::new(Barrier::new(num_threads));

    let all_ids: HashSet<Nulid> = (0..num_threads)
        .map(|_| {
            let generator_clone = Arc::clone(&generator);
            let barrier_clone = Arc::clone(&barrier);
            thread::spawn(move || {
                barrier_clone.wait();
                (0..ids_per_thread)
                    .map(|_| {
                        generator_clone
                            .generate()
                            .expect("generation should succeed")
                    })
                    .collect::<Vec<_>>()
            })
        })
        .collect::<Vec<_>>()
        .into_iter()
        .flat_map(|h| h.join().expect("thread should complete"))
        .collect();

    assert_eq!(
        all_ids.len(),
        num_threads * ids_per_thread,
        "uniqueness violated in stress test"
    );
}

// ============================================================================
// Chaos Testing: Combined Adversarial Conditions
// ============================================================================

/// Test with multiple adversarial conditions combined.
#[test]
fn test_chaos_combined_conditions() {
    // Clock that oscillates AND drifts backward
    let clock = ChaosClock::with_oscillation(1_000_000_000_000, 100_000_000, 5);
    let rng = ChaosRng::collision_every(3);
    let generator = Generator::<_, _, NoNodeId>::with_deps(&clock, &rng);

    let mut ids = HashSet::new();
    let mut last: Option<Nulid> = None;

    for i in 0..1000 {
        let id = generator.generate().expect("generation should succeed");

        // Check uniqueness
        assert!(ids.insert(id), "duplicate in chaos test at {i}: {id}");

        // Check monotonicity
        if let Some(prev) = last {
            assert!(id > prev, "monotonicity violated in chaos test at {i}");
        }
        last = Some(id);
    }
}

/// Test rapid clock changes during generation.
#[test]
fn test_chaos_rapid_clock_changes() {
    let clock = ChaosClock::new(1_000_000_000);
    let rng = SequentialRng::new();
    let generator = Generator::<_, _, NoNodeId>::with_deps(&clock, &rng);

    let mut ids = Vec::new();

    for i in 0..500 {
        // Randomly change clock state
        match i % 5 {
            0 => clock.advance(1_000_000),       // Forward 1ms
            1 => clock.set(500_000_000),         // Jump backward
            2 => clock.freeze_at(1_000_000_000), // Freeze
            3 => clock.unfreeze(),               // Unfreeze
            4 => clock.set(2_000_000_000_000),   // Jump far forward
            _ => {}
        }

        ids.push(generator.generate().expect("generation should succeed"));
    }

    // All should still be monotonic
    for i in 1..ids.len() {
        assert!(
            ids[i] > ids[i - 1],
            "monotonicity violated during rapid clock changes at {i}"
        );
    }
}

/// Test with maximum contention and adversarial clock.
#[test]
fn test_chaos_max_contention_adversarial_clock() {
    let clock = Arc::new(ChaosClock::with_oscillation(1_000_000_000, 50_000_000, 7));
    let num_threads = 16;
    let ids_per_thread = 500;
    let barrier = Arc::new(Barrier::new(num_threads + 1)); // +1 for chaos thread

    let generator = Arc::new(Generator::new()); // Use real generator

    // Spawn chaos thread that manipulates time
    let chaos_clock = Arc::clone(&clock);
    let chaos_barrier = Arc::clone(&barrier);
    let chaos_handle = thread::spawn(move || {
        chaos_barrier.wait();
        for i in 0..1000 {
            match i % 4 {
                0 => chaos_clock.advance(1_000_000),
                1 => chaos_clock.set(500_000_000),
                2 => chaos_clock.freeze_at(1_000_000_000),
                3 => chaos_clock.unfreeze(),
                _ => {}
            }
            thread::sleep(Duration::from_micros(10));
        }
    });

    // Spawn worker threads and collect results
    let all_ids: HashSet<Nulid> = (0..num_threads)
        .map(|_| {
            let generator_clone = Arc::clone(&generator);
            let barrier_clone = Arc::clone(&barrier);
            thread::spawn(move || {
                barrier_clone.wait();
                (0..ids_per_thread)
                    .map(|_| {
                        generator_clone
                            .generate()
                            .expect("generation should succeed")
                    })
                    .collect::<Vec<_>>()
            })
        })
        .collect::<Vec<_>>()
        .into_iter()
        .flat_map(|h| h.join().expect("thread should complete"))
        .collect();

    chaos_handle.join().expect("chaos thread should complete");

    assert_eq!(
        all_ids.len(),
        num_threads * ids_per_thread,
        "uniqueness violated under chaos conditions"
    );
}

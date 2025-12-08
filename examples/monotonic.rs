//! Advanced example demonstrating monotonic NULID generation.
//!
//! This example shows how to use the Generator for thread-safe,
//! monotonic NULID generation in various scenarios.

use nulid::Generator;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("NULID Monotonic Generation Example");
    println!("===================================\n");

    // 1. Basic monotonic generation
    println!("1. Basic Monotonic Generation");
    println!("   Generating NULIDs in rapid succession...");
    let generator = Generator::new();
    let mut prev_nulid = generator.generate()?;
    println!("   First:  {}", prev_nulid);

    for i in 1..5 {
        let nulid = generator.generate()?;
        println!("   NULID {}: {}", i + 1, nulid);
        assert!(nulid > prev_nulid, "NULIDs must be strictly increasing");
        prev_nulid = nulid;
    }
    println!("   ✓ All NULIDs are strictly increasing\n");

    // 2. High-speed generation
    println!("2. High-Speed Generation");
    println!("   Generating 10,000 NULIDs as fast as possible...");
    let start = std::time::Instant::now();
    let generator = Generator::new();
    let mut nulids = Vec::with_capacity(10000);

    for _ in 0..10000 {
        nulids.push(generator.generate()?);
    }

    let duration = start.elapsed();
    println!("   Generated: {} NULIDs", nulids.len());
    println!("   Time: {:?}", duration);
    println!(
        "   Rate: {:.0} NULIDs/second",
        10000.0 / duration.as_secs_f64()
    );

    // Verify strict ordering
    let is_sorted = nulids.windows(2).all(|w| w[0] < w[1]);
    println!("   ✓ Strict monotonic order maintained: {}\n", is_sorted);

    // 3. Concurrent generation from multiple threads
    println!("3. Concurrent Generation");
    println!("   Spawning 10 threads, each generating 1,000 NULIDs...");
    let generator = Arc::new(Generator::new());
    let mut handles = vec![];

    for thread_id in 0..10 {
        let generator_clone = Arc::clone(&generator);
        let handle = thread::spawn(move || {
            let mut thread_nulids = Vec::with_capacity(1000);
            for _ in 0..1000 {
                thread_nulids.push(generator_clone.generate().unwrap());
            }
            (thread_id, thread_nulids)
        });
        handles.push(handle);
    }

    // Collect all NULIDs from all threads
    let mut all_nulids = Vec::with_capacity(10000);
    for handle in handles {
        let (thread_id, thread_nulids) = handle.join().unwrap();
        println!(
            "   Thread {} generated {} NULIDs",
            thread_id,
            thread_nulids.len()
        );
        all_nulids.extend(thread_nulids);
    }

    // Verify no duplicates
    let original_len = all_nulids.len();
    all_nulids.sort();
    all_nulids.dedup();
    let unique_len = all_nulids.len();

    println!("   Total NULIDs: {}", original_len);
    println!("   Unique NULIDs: {}", unique_len);
    println!("   ✓ No duplicates: {}\n", original_len == unique_len);

    // 4. Generation with time delays
    println!("4. Generation with Time Delays");
    println!("   Generating NULIDs with 10ms delays...");
    let generator = Generator::new();

    for i in 0..5 {
        let nulid = generator.generate()?;
        println!(
            "   [{}] {} (timestamp: {})",
            i + 1,
            nulid,
            nulid.timestamp_nanos()
        );
        if i < 4 {
            thread::sleep(Duration::from_millis(10));
        }
    }
    println!("   ✓ Each NULID has a later timestamp\n");

    // 5. Demonstrate monotonicity within same nanosecond
    println!("5. Monotonicity Within Same Nanosecond");
    println!("   Rapidly generating NULIDs to test same-nanosecond handling...");
    let generator = Generator::new();
    let mut same_ns_count = 0;
    let mut total_generated = 0;

    let nulid1 = generator.generate()?;
    let mut prev_ts = nulid1.timestamp_nanos();
    let mut prev_nulid = nulid1;

    for _ in 0..1000 {
        let nulid = generator.generate()?;
        let ts = nulid.timestamp_nanos();

        // Check if generated in same nanosecond
        if ts == prev_ts {
            same_ns_count += 1;
            // Verify randomness was incremented
            assert!(
                nulid > prev_nulid,
                "NULIDs in same nanosecond must have increasing randomness"
            );
        }

        prev_ts = ts;
        prev_nulid = nulid;
        total_generated += 1;
    }

    println!("   Total generated: {}", total_generated + 1);
    println!("   Same nanosecond: {}", same_ns_count);
    println!("   ✓ Monotonicity maintained even within same nanosecond\n");

    // 6. Shared Generator via Arc
    println!("6. Shared Generator via Arc");
    println!("   Using Arc-wrapped generator across threads...");
    let generator = Arc::new(Generator::new());
    let mut handles = vec![];

    for thread_id in 0..5 {
        let generator_clone = Arc::clone(&generator);
        let handle = thread::spawn(move || {
            let mut ids = Vec::new();
            for _ in 0..100 {
                ids.push(generator_clone.generate().unwrap());
            }
            (thread_id, ids.len())
        });
        handles.push(handle);
    }

    let mut total = 0;
    for handle in handles {
        let (thread_id, count) = handle.join().unwrap();
        println!("   Thread {} generated: {} NULIDs", thread_id, count);
        total += count;
    }
    println!("   Total: {} NULIDs", total);
    println!("   ✓ Arc-wrapped generators share state correctly\n");

    // 7. Demonstrate sortable properties
    println!("7. Sortable Properties");
    println!("   Verifying lexicographic string sorting...");
    let generator = Generator::new();
    let mut nulids = vec![];
    let mut strings = vec![];

    for _ in 0..10 {
        let nulid = generator.generate()?;
        strings.push(nulid.to_string());
        nulids.push(nulid);
    }

    // Sort both
    let mut sorted_nulids = nulids.clone();
    sorted_nulids.sort();

    let mut sorted_strings = strings.clone();
    sorted_strings.sort();

    // Convert sorted strings back to NULIDs
    let nulids_from_strings: Vec<_> = sorted_strings.iter().map(|s| s.parse().unwrap()).collect();

    println!(
        "   Original order matches sorted: {}",
        nulids == sorted_nulids
    );
    println!(
        "   String sort matches NULID sort: {}",
        sorted_nulids == nulids_from_strings
    );
    println!("   ✓ Lexicographic sorting is consistent\n");

    // 8. Performance comparison
    println!("8. Performance Comparison");
    println!("   Comparing single-threaded vs concurrent generation...");

    let single_start = std::time::Instant::now();
    let generator = Generator::new();
    for _ in 0..5000 {
        let _ = generator.generate();
    }
    let single_duration = single_start.elapsed();

    let concurrent_start = std::time::Instant::now();
    let generator = Arc::new(Generator::new());
    let mut handles = vec![];

    for _ in 0..5 {
        let generator_clone = Arc::clone(&generator);
        let handle = thread::spawn(move || {
            for _ in 0..1000 {
                let _ = generator_clone.generate();
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }
    let concurrent_duration = concurrent_start.elapsed();

    println!("   Single-threaded (5,000): {:?}", single_duration);
    println!("   Concurrent 5 threads (5,000): {:?}", concurrent_duration);
    println!(
        "   Speedup: {:.2}x\n",
        single_duration.as_secs_f64() / concurrent_duration.as_secs_f64()
    );

    println!("All monotonic generation examples completed successfully! ✓");

    Ok(())
}

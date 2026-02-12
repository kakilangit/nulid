//! WASM Example demonstrating NULID generation in WebAssembly environments.
//!
//! This example shows how NULID works with the `wasm` feature enabled,
//! using `web-time` for browser-compatible timing.

#![allow(clippy::similar_names)]

use nulid::Nulid;

fn main() -> Result<(), Box<dyn core::error::Error>> {
    println!("NULID WASM Example");
    println!("===================\n");

    // Generate a new NULID using WASM-compatible timing
    println!("1. Generating NULID with WASM timing...");
    let nulid1 = Nulid::new()?;
    println!("   NULID: {nulid1}");
    println!("   Timestamp (ns): {}", nulid1.nanos());
    println!();

    // Generate another NULID to show monotonicity
    println!("2. Generating another NULID...");
    let nulid2 = Nulid::new()?;
    println!("   NULID: {nulid2}");
    println!("   Timestamp (ns): {}", nulid2.nanos());
    println!();

    // Demonstrate that WASM timing maintains ordering
    println!("3. Verifying monotonic ordering with WASM timing...");
    if nulid1 < nulid2 {
        println!("   {nulid1} < {nulid2} ✓");
    } else {
        println!("   {nulid1} >= {nulid2}");
    }
    println!();

    // Show nanosecond precision from web-time
    println!("4. Demonstrating nanosecond precision...");
    let nanos1 = nulid1.nanos();
    let nanos2 = nulid2.nanos();
    let diff_ns = nanos2 - nanos1;
    println!("   Time difference: {diff_ns} nanoseconds");
    println!(
        "   Has sub-millisecond precision: {}",
        if diff_ns < 1_000_000 { "✓" } else { "✗" }
    );
    println!();

    // Generate multiple NULIDs to show consistency
    println!("5. Generating multiple NULIDs with WASM timing...");
    let mut ids = Vec::new();
    for i in 0..5 {
        let nulid = Nulid::new()?;
        println!("   [{:02}] {} (ns: {})", i + 1, nulid, nulid.nanos());
        ids.push(nulid);
    }
    println!();

    // Verify all are unique and sorted
    println!("6. Verifying uniqueness and sorting...");
    let is_unique = ids.windows(2).all(|w| w[0] != w[1]);
    let is_sorted = ids.windows(2).all(|w| w[0] < w[1]);
    println!("   All unique: {}", if is_unique { "✓" } else { "✗" });
    println!("   Sorted: {}", if is_sorted { "✓" } else { "✗" });
    println!();

    // Show that string representation works in WASM
    println!("7. String serialization (WASM-compatible)...");
    let nulid_str = nulid1.to_string();
    println!("   Original: {nulid_str}");
    let parsed: Nulid = nulid_str.parse()?;
    println!("   Parsed:   {parsed}");
    println!("   Match: {}", if nulid1 == parsed { "✓" } else { "✗" });
    println!();

    println!("WASM example completed successfully! ✓");

    Ok(())
}

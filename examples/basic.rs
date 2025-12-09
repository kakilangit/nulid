//! Basic example demonstrating NULID generation and usage.

#![allow(clippy::similar_names)]

use nulid::Nulid;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("NULID Basic Example");
    println!("===================\n");

    // Generate a new NULID
    println!("1. Generating a new NULID...");
    let nulid1 = Nulid::new()?;
    println!("   NULID: {nulid1}");
    println!("   Timestamp (ns): {}", nulid1.timestamp_nanos());
    println!();

    // Generate another NULID
    println!("2. Generating another NULID...");
    let nulid2 = Nulid::new()?;
    println!("   NULID: {nulid2}");
    println!("   Timestamp (ns): {}", nulid2.timestamp_nanos());
    println!();

    // Demonstrate sorting
    println!("3. Demonstrating lexicographic sorting...");
    if nulid1 < nulid2 {
        println!("   {nulid1} < {nulid2} ✓");
    } else {
        println!("   {nulid1} >= {nulid2}");
    }
    println!();

    // Parse from string
    println!("4. Parsing NULID from string...");
    let nulid_str = nulid1.to_string();
    println!("   Original: {nulid_str}");
    let parsed: Nulid = nulid_str.parse()?;
    println!("   Parsed:   {parsed}");
    println!("   Match: {}", if nulid1 == parsed { "✓" } else { "✗" });
    println!();

    // Case-insensitive parsing
    println!("5. Case-insensitive parsing...");
    let lowercase = nulid_str.to_lowercase();
    println!("   Lowercase: {lowercase}");
    let parsed_lower: Nulid = lowercase.parse()?;
    println!("   Parsed:    {parsed_lower}");
    println!(
        "   Match: {}",
        if nulid1 == parsed_lower { "✓" } else { "✗" }
    );
    println!();

    // Convert to bytes and back
    println!("6. Byte serialization...");
    let bytes = nulid1.to_bytes();
    println!("   Bytes: {bytes:02X?}");
    println!("   Length: {} bytes", bytes.len());
    let from_bytes = Nulid::from_bytes(bytes);
    println!("   Reconstructed: {from_bytes}");
    println!("   Match: {}", if nulid1 == from_bytes { "✓" } else { "✗" });
    println!();

    // Generate multiple NULIDs
    println!("7. Generating multiple NULIDs...");
    let mut ids = Vec::new();
    for i in 0..5 {
        let nulid = Nulid::new()?;
        println!("   [{:02}] {}", i + 1, nulid);
        ids.push(nulid);
    }
    println!();

    // Verify sorting
    println!("8. Verifying sorted order...");
    let is_sorted = ids.windows(2).all(|w| w[0] <= w[1]);
    println!(
        "   Generated in order: {}",
        if is_sorted { "✓" } else { "✗" }
    );
    println!();

    println!("All examples completed successfully! ✓");

    Ok(())
}

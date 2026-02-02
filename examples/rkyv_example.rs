//! Example demonstrating rkyv zero-copy serialization with NULID.
//!
//! This example shows how to use rkyv for efficient serialization and deserialization
//! of NULID values, including zero-copy access to archived data.
//!
//! Run with: cargo run --example rkyv_example --features rkyv

#![allow(clippy::doc_markdown)]

use nulid::Nulid;
use rkyv::{access_unchecked, to_bytes};

fn main() -> Result<(), Box<dyn core::error::Error>> {
    println!("=== NULID rkyv Serialization Example ===\n");

    // Create a new NULID
    let nulid = Nulid::new()?;
    println!("Original NULID: {nulid}");
    println!("  Timestamp (nanos): {}", nulid.nanos());
    println!("  Random component: {}", nulid.random());
    println!();

    // Serialize using rkyv
    println!("Serializing with rkyv...");
    let bytes = to_bytes::<rkyv::rancor::Error>(&nulid)?;
    println!("  Serialized size: {} bytes", bytes.len());
    println!();

    // Zero-copy access to archived data
    println!("Accessing archived data (zero-copy)...");
    let _archived = unsafe { access_unchecked::<rkyv::Archived<Nulid>>(&bytes) };
    println!("  Archived data accessed without deserialization");
    println!();

    // Deserialize back to NULID
    println!("Deserializing back to NULID...");
    let deserialized: Nulid = rkyv::from_bytes::<Nulid, rkyv::rancor::Error>(&bytes)?;
    println!("  Deserialized NULID: {deserialized}");
    println!("  Timestamp (nanos): {}", deserialized.nanos());
    println!("  Random component: {}", deserialized.random());
    println!();

    // Verify roundtrip
    assert_eq!(nulid, deserialized);
    println!("✓ Roundtrip successful: original == deserialized");
    println!();

    // Example with multiple NULIDs
    println!("=== Serializing Multiple NULIDs ===\n");
    let nulids = vec![
        Nulid::new()?,
        Nulid::new()?,
        Nulid::nil(),
        Nulid::from_u128(0xDEAD_BEEF_CAFE_BABE_0123_4567_89AB_CDEF),
    ];

    println!("Original NULIDs:");
    for (i, id) in nulids.iter().enumerate() {
        println!("  [{i}] {id}");
    }
    println!();

    // Serialize the vector
    let bytes = to_bytes::<rkyv::rancor::Error>(&nulids)?;
    println!(
        "Serialized {} NULIDs to {} bytes",
        nulids.len(),
        bytes.len()
    );
    println!();

    // Deserialize the vector
    let deserialized: Vec<Nulid> = rkyv::from_bytes::<Vec<Nulid>, rkyv::rancor::Error>(&bytes)?;

    println!("Deserialized NULIDs:");
    for (i, id) in deserialized.iter().enumerate() {
        println!("  [{i}] {id}");
    }
    println!();

    // Verify all match
    assert_eq!(nulids.len(), deserialized.len());
    for (original, recovered) in nulids.iter().zip(deserialized.iter()) {
        assert_eq!(original, recovered);
    }
    println!("✓ All {} NULIDs matched after roundtrip", nulids.len());
    println!();

    // Demonstrate ordering preservation
    println!("=== Ordering Preservation ===\n");
    let mut sorted_nulids = vec![
        Nulid::from_nanos(3000, 300),
        Nulid::from_nanos(1000, 100),
        Nulid::from_nanos(2000, 200),
    ];
    println!("Before sorting:");
    for id in &sorted_nulids {
        println!("  {} (timestamp: {})", id, id.nanos());
    }
    println!();

    sorted_nulids.sort();
    println!("After sorting:");
    for id in &sorted_nulids {
        println!("  {} (timestamp: {})", id, id.nanos());
    }
    println!();

    // Serialize and deserialize sorted NULIDs
    let bytes = to_bytes::<rkyv::rancor::Error>(&sorted_nulids)?;
    let recovered: Vec<Nulid> = rkyv::from_bytes::<Vec<Nulid>, rkyv::rancor::Error>(&bytes)?;

    println!("After rkyv roundtrip:");
    for id in &recovered {
        println!("  {} (timestamp: {})", id, id.nanos());
    }
    println!();

    // Verify ordering is preserved
    for i in 0..recovered.len() - 1 {
        assert!(recovered[i] < recovered[i + 1]);
    }
    println!("✓ Ordering preserved after serialization");
    println!();

    println!("=== Example Complete ===");
    Ok(())
}

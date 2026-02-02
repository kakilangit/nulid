//! Example demonstrating postgres-types integration with NULID.
//!
//! This example shows how to use NULID with the postgres-types crate for
//! encoding and decoding NULIDs as `PostgreSQL` UUID types.
//!
//! Run with: cargo run --example postgres_types_example --features postgres-types

#![allow(clippy::too_many_lines)]
#![allow(clippy::doc_markdown)]

use nulid::Nulid;
use postgres_types::{FromSql, ToSql, Type};

fn main() -> Result<(), Box<dyn core::error::Error + Send + Sync>> {
    println!("=== NULID postgres-types Integration Example ===\n");

    // Create a new NULID
    let nulid = Nulid::new()?;
    println!("Original NULID: {nulid}");
    println!("  As u128: 0x{:032x}", nulid.as_u128());
    println!("  Timestamp (nanos): {}", nulid.nanos());
    println!("  Random component: {}", nulid.random());
    println!();

    // Encode to PostgreSQL UUID format
    println!("Encoding as PostgreSQL UUID...");
    let ty = Type::UUID;
    let mut buf = bytes::BytesMut::new();
    nulid.to_sql(&ty, &mut buf)?;

    println!("  Encoded size: {} bytes", buf.len());
    println!("  Encoded bytes: {}", hex_dump(&buf));
    println!();

    // Decode from PostgreSQL UUID format
    println!("Decoding from PostgreSQL UUID...");
    let decoded = Nulid::from_sql(&ty, &buf)?;
    println!("  Decoded NULID: {decoded}");
    println!("  As u128: 0x{:032x}", decoded.as_u128());
    println!("  Timestamp (nanos): {}", decoded.nanos());
    println!("  Random component: {}", decoded.random());
    println!();

    // Verify roundtrip
    assert_eq!(nulid, decoded);
    println!("✓ Roundtrip successful: original == decoded");
    println!(
        "✓ Timestamp preserved: {} == {}",
        nulid.nanos(),
        decoded.nanos()
    );
    println!(
        "✓ Random component preserved: {} == {}",
        nulid.random(),
        decoded.random()
    );
    println!();

    // Test with nil NULID
    println!("=== Testing with Nil NULID ===\n");
    let nil = Nulid::nil();
    println!("Nil NULID: {nil}");
    println!("  Is nil: {}", nil.is_nil());
    println!();

    let mut buf = bytes::BytesMut::new();
    nil.to_sql(&ty, &mut buf)?;
    let decoded_nil = Nulid::from_sql(&ty, &buf)?;

    println!("After roundtrip:");
    println!("  Decoded NULID: {decoded_nil}");
    println!("  Is nil: {}", decoded_nil.is_nil());
    println!();

    assert!(decoded_nil.is_nil());
    println!("✓ Nil NULID roundtrip successful");
    println!();

    // Test with specific value
    println!("=== Testing with Specific Value ===\n");
    let specific = Nulid::from_u128(0x0123_4567_89AB_CDEF_FEDC_BA98_7654_3210);
    println!("Specific NULID: {specific}");
    println!("  As u128: 0x{:032x}", specific.as_u128());
    println!();

    let mut buf = bytes::BytesMut::new();
    specific.to_sql(&ty, &mut buf)?;
    let decoded_specific = Nulid::from_sql(&ty, &buf)?;

    println!("After roundtrip:");
    println!("  Decoded NULID: {decoded_specific}");
    println!("  As u128: 0x{:032x}", decoded_specific.as_u128());
    println!();

    assert_eq!(specific.as_u128(), decoded_specific.as_u128());
    println!("✓ Specific value roundtrip successful");
    println!();

    // Test multiple NULIDs
    println!("=== Testing Multiple NULIDs ===\n");
    let nulids = vec![
        Nulid::new()?,
        Nulid::new()?,
        Nulid::nil(),
        Nulid::from_nanos(1_234_567_890_123_456_789, 0xABC_DEF),
    ];

    println!("Original NULIDs:");
    for (i, id) in nulids.iter().enumerate() {
        println!("  [{i}] {id}");
    }
    println!();

    // Encode and decode each
    let mut decoded_nulids = Vec::new();
    for id in &nulids {
        let mut buf = bytes::BytesMut::new();
        id.to_sql(&ty, &mut buf)?;
        let decoded = Nulid::from_sql(&ty, &buf)?;
        decoded_nulids.push(decoded);
    }

    println!("After roundtrip:");
    for (i, id) in decoded_nulids.iter().enumerate() {
        println!("  [{i}] {id}");
    }
    println!();

    // Verify all match
    for (original, decoded) in nulids.iter().zip(decoded_nulids.iter()) {
        assert_eq!(original, decoded);
    }
    println!("✓ All {} NULIDs matched after roundtrip", nulids.len());
    println!();

    // Demonstrate PostgreSQL UUID compatibility
    println!("=== PostgreSQL UUID Compatibility ===\n");
    println!("NULIDs are stored as PostgreSQL UUIDs (16 bytes, big-endian)");
    println!("This allows NULID to be used in PostgreSQL UUID columns");
    println!("while maintaining nanosecond precision and lexicographic sortability.");
    println!();

    let id1 = Nulid::from_nanos(1000, 100);
    let id2 = Nulid::from_nanos(2000, 200);

    println!("Example: Lexicographic ordering");
    println!("  NULID 1: {} (timestamp: {})", id1, id1.nanos());
    println!("  NULID 2: {} (timestamp: {})", id2, id2.nanos());
    println!();

    assert!(id1 < id2);
    println!("✓ NULID 1 < NULID 2 (lexicographically sorted by timestamp)");
    println!();

    println!("=== Example Complete ===");
    Ok(())
}

fn hex_dump(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<Vec<_>>()
        .join(" ")
}

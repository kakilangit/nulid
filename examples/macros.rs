//! Example demonstrating the `nulid!()` macro.
//!
//! This example shows how to use the `nulid!()` macro for convenient
//! NULID generation with different error handling strategies.
//!
//! Run with: cargo run --example macros --features macros
#![allow(clippy::expect_used)]
#![allow(clippy::similar_names)]

use nulid::{Nulid, nulid};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== nulid!() Macro Example ===\n");

    // Simple usage - generates a NULID, panics on error
    println!("--- Simple Generation ---");
    let id1 = nulid!();
    println!("Generated ID 1: {id1}");

    let id2 = nulid!();
    println!("Generated ID 2: {id2}");

    // IDs should be different
    assert_ne!(id1, id2);
    println!("✓ IDs are unique\n");

    // With error handling - returns Result
    println!("--- With Error Handling (fallible mode) ---");
    let id3 = nulid!(?)?;
    println!("Generated ID 3: {id3}");

    let id4 = nulid!(?).expect("Failed to generate NULID");
    println!("Generated ID 4: {id4}");

    // Can use in Result-returning context
    let id5 = generate_id()?;
    println!("Generated ID 5: {id5}");

    println!("\n--- Multiple Generations ---");
    let mut ids = Vec::new();
    for i in 0..5 {
        let id = nulid!();
        println!("ID {}: {}", i + 1, id);
        ids.push(id);
    }

    // All IDs should be unique
    for i in 0..ids.len() {
        for j in (i + 1)..ids.len() {
            assert_ne!(ids[i], ids[j]);
        }
    }
    println!("✓ All {} IDs are unique", ids.len());

    // IDs should be sortable by timestamp
    println!("\n--- Lexicographic Ordering ---");
    let first = nulid!();
    std::thread::sleep(std::time::Duration::from_millis(2));
    let second = nulid!();

    assert!(first < second);
    println!("First:  {first}");
    println!("Second: {second}");
    println!("✓ IDs are lexicographically sorted by timestamp");

    println!("\n--- Comparison with Nulid::new() ---");
    println!("nulid!()           = {}", nulid!());
    println!("Nulid::new()?      = {}", Nulid::new()?);
    println!("nulid!(?)?         = {}", nulid!(?)?);

    println!("\n=== Example Complete ===");
    Ok(())
}

fn generate_id() -> Result<Nulid, Box<dyn std::error::Error>> {
    // Using fallible mode in a Result-returning function
    Ok(nulid!(?)?)
}

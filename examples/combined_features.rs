//! Example demonstrating both `Id` derive and `nulid!()` macro features.
//!
//! This example shows how to use both features together for maximum convenience
//! when working with typed NULID wrappers.
//!
//! Run with: cargo run --example `combined_features` --features derive,macros
#![allow(clippy::similar_names)]

use nulid::{Nulid, nulid};
use nulid_derive::Id;

#[derive(Id)]
pub struct UserId(Nulid);

#[derive(Id)]
pub struct OrderId(pub Nulid);

#[derive(Id)]
pub struct ProductId(Nulid);

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Combined Features Example ===\n");
    println!("Using both `Id` derive macro and `nulid!()` macro\n");

    // Generate typed IDs using the nulid!() macro
    println!("--- Creating Typed IDs with nulid!() ---");
    let user_id = UserId::from(nulid!());
    let order_id = OrderId::from(nulid!());
    let product_id = ProductId::from(nulid!());

    println!("User ID:    {user_id}");
    println!("Order ID:   {order_id}");
    println!("Product ID: {product_id}");

    // Using fallible variant with typed IDs
    println!("\n--- Fallible Generation ---");
    let user_id2 = UserId::from(nulid!(?)?);
    println!("User ID 2:  {user_id2}");

    // Parse from strings (provided by Id derive)
    println!("\n--- Parsing from Strings ---");
    let user_id_str = user_id.to_string();
    let parsed_user_id = UserId::try_from(user_id_str.as_str())?;
    println!("Original:   {user_id}");
    println!("Parsed:     {parsed_user_id}");
    assert_eq!(user_id, parsed_user_id);
    println!("✓ Parsing successful");

    // Type safety in action
    println!("\n--- Type Safety ---");
    println!("UserId and OrderId are different types even though both wrap Nulid:");
    println!("  UserId:  {user_id}");
    println!("  OrderId: {order_id}");
    // This would be a compile error:
    // assert_eq!(user_id, order_id);
    println!("✓ Types are distinct (compile-time safety)");

    // Building a simple data structure
    println!("\n--- Building Data Structures ---");
    let mut users = std::collections::HashMap::new();

    for i in 1..=3 {
        let id = UserId::from(nulid!());
        users.insert(id, format!("User {i}"));
        println!("Created user: {} -> {}", id, users[&id]);
    }

    // Demonstrate conversion chains
    println!("\n--- Conversion Chains ---");
    let id1 = nulid!();
    let typed_id = UserId::from(id1);
    let back_to_nulid: Nulid = typed_id.into();
    let as_string = typed_id.to_string();
    let from_string: UserId = as_string.parse()?;

    println!("Original NULID:     {id1}");
    println!("As UserId:          {typed_id}");
    println!("Back to Nulid:      {back_to_nulid}");
    println!("As String:          {as_string}");
    println!("Parsed from String: {from_string}");
    assert_eq!(id1, back_to_nulid);
    assert_eq!(typed_id, from_string);
    println!("✓ All conversions preserve value");

    // Ordering and sorting
    println!("\n--- Ordering ---");
    let ids = [
        UserId::from(nulid!()),
        UserId::from(nulid!()),
        UserId::from(nulid!()),
    ];

    println!("Generated IDs:");
    for (i, id) in ids.iter().enumerate() {
        println!("  [{i}] {id}");
    }

    // IDs should already be sorted by timestamp since they were created sequentially
    println!("\nIDs are naturally sorted by creation time:");
    for i in 0..ids.len() - 1 {
        assert!(ids[i] < ids[i + 1]);
        println!("  {} < {} ✓", ids[i], ids[i + 1]);
    }

    // Demonstrate with error handling
    println!("\n--- Error Handling ---");
    match create_user() {
        Ok(id) => println!("Created user with ID: {id}"),
        Err(e) => println!("Failed to create user: {e}"),
    }

    // Demonstrate with Option
    println!("\n--- Working with Option ---");
    let maybe_user_id = try_create_user().ok();
    match maybe_user_id {
        Some(id) => println!("Successfully created user: {id}"),
        None => println!("Failed to create user"),
    }

    // Using as_ref
    println!("\n--- AsRef trait ---");
    let nulid_ref: &Nulid = user_id.as_ref();
    println!("User ID as &Nulid: {nulid_ref}");
    assert_eq!(nulid_ref, &Nulid::from(user_id));
    println!("✓ AsRef works correctly");

    println!("\n=== Example Complete ===");
    println!("\nBenefits of combining these features:");
    println!("  • Type safety: Different ID types can't be mixed");
    println!("  • Convenience: nulid!() for quick generation");
    println!("  • Error handling: nulid!(?) for Result-based flow");
    println!("  • Automatic traits: TryFrom, Display, FromStr, etc.");
    println!("  • Zero runtime overhead: Everything is compile-time");

    Ok(())
}

fn create_user() -> Result<UserId, Box<dyn std::error::Error>> {
    // Using fallible macro in a Result-returning function
    Ok(UserId::from(nulid!(?)?))
}

fn try_create_user() -> Result<UserId, Box<dyn std::error::Error>> {
    // Another example of fallible creation
    let id = nulid!(?)?;
    Ok(UserId::from(id))
}

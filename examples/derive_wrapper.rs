//! Example demonstrating the `Id` derive macro.
//!
//! This example shows how to use the `Id` derive macro to automatically
//! implement common traits for types that wrap `Nulid`.
//!
//! Run with: cargo run --example `derive_wrapper` --features derive

use nulid::{Nulid, Result};
use nulid_derive::Id;

#[derive(Id, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UserId(Nulid);

#[derive(Id, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct OrderId(pub Nulid);

#[derive(Id, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ProductId(Nulid);

fn main() -> Result<()> {
    println!("=== Id Derive Macro Example ===\n");

    // Create a new UserId
    let user_id = UserId::from(Nulid::new()?);
    println!("Generated UserId: {user_id}");

    // Convert to string and parse back
    let user_id_str = user_id.to_string();
    println!("UserId as string: {user_id_str}");

    // TryFrom<&str>
    let parsed_user_id = UserId::try_from(user_id_str.as_str())?;
    println!("Parsed from &str: {parsed_user_id}");
    assert_eq!(user_id, parsed_user_id);

    // TryFrom<String>
    let parsed_user_id2 = UserId::try_from(user_id_str.clone())?;
    println!("Parsed from String: {parsed_user_id2}");
    assert_eq!(user_id, parsed_user_id2);

    // FromStr trait
    let parsed_user_id3: UserId = user_id_str.parse()?;
    println!("Parsed via FromStr: {parsed_user_id3}");
    assert_eq!(user_id, parsed_user_id3);

    println!("\n--- Conversion Between Nulid and Wrapper ---\n");

    // From<Nulid> for UserId
    let nulid = Nulid::new()?;
    let order_id = OrderId::from(nulid);
    println!("OrderId from Nulid: {order_id}");

    // From<OrderId> for Nulid
    let back_to_nulid: Nulid = order_id.into();
    println!("Back to Nulid: {back_to_nulid}");
    assert_eq!(nulid, back_to_nulid);

    // AsRef<Nulid>
    let nulid_ref: &Nulid = order_id.as_ref();
    println!("AsRef<Nulid>: {nulid_ref}");
    assert_eq!(&nulid, nulid_ref);

    println!("\n--- Multiple Wrapper Types ---\n");

    // Different wrapper types are type-safe
    let product_id = ProductId::from(Nulid::new()?);
    println!("ProductId: {product_id}");
    println!("OrderId:   {order_id}");

    // This would be a compile error (different types):
    // assert_eq!(product_id, order_id);

    // But you can compare their underlying Nulid values:
    let product_nulid: Nulid = product_id.into();
    let order_nulid: Nulid = order_id.into();
    println!(
        "Product and Order IDs are different: {}",
        product_nulid != order_nulid
    );

    println!("\n--- Error Handling ---\n");

    // Invalid string
    match UserId::try_from("invalid-nulid-string") {
        Ok(id) => println!("Unexpected success: {id}"),
        Err(e) => println!("Expected error parsing invalid string: {e}"),
    }

    // Invalid length
    match UserId::try_from("SHORT") {
        Ok(id) => println!("Unexpected success: {id}"),
        Err(e) => println!("Expected error with wrong length: {e}"),
    }

    println!("\n=== Example Complete ===");
    Ok(())
}

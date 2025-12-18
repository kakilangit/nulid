//! Example demonstrating the `Id` derive macro.
//!
//! This example shows how to use the `Id` derive macro to automatically
//! implement common traits for types that wrap `Nulid`.
//!
//! Run with: cargo run --example `derive_wrapper` --features derive

use nulid::{Nulid, Result};
use nulid_derive::Id;

#[derive(Id)]
pub struct UserId(Nulid);

#[derive(Id)]
pub struct OrderId(pub Nulid);

#[derive(Id)]
pub struct ProductId(Nulid);

fn main() -> Result<()> {
    println!("=== Id Derive Macro Example ===\n");

    // Create a new UserId using new()
    let user_id = UserId::new()?;
    println!("Generated UserId with new(): {user_id}");

    // Create using Default
    let default_user_id = UserId::default();
    println!("Default UserId (ZERO): {default_user_id}");

    // Create using From<Nulid>
    let user_id_from_nulid = UserId::from(Nulid::new()?);
    println!("UserId from Nulid: {user_id_from_nulid}");

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

    // Create OrderId using new()
    let order_id = OrderId::new()?;
    println!("OrderId with new(): {order_id}");

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
    let product_id = ProductId::new()?;
    println!("ProductId: {product_id}");
    println!("OrderId:   {order_id}");

    // Default instances
    let default_product = ProductId::default();
    println!("Default ProductId (ZERO): {default_product}");

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

    println!("\n--- Direct Access to Nulid Methods (via Deref) ---\n");

    // With Deref, you can call any Nulid method directly on the wrapper type
    let user_id = UserId::new()?;

    // Access timestamp information
    let nanos = user_id.nanos();
    let micros = user_id.micros();
    let millis = user_id.millis();
    let seconds = user_id.seconds();
    let subsec_nanos = user_id.subsec_nanos();

    println!("Direct method access on UserId:");
    println!("  Nanoseconds:  {nanos}");
    println!("  Microseconds: {micros}");
    println!("  Milliseconds: {millis}");
    println!("  Seconds:      {seconds}");
    println!("  Subsec nanos: {subsec_nanos}");

    // Access random component
    let random = user_id.random();
    println!("  Random bits:  {random}");

    // Get both parts at once
    let (timestamp, rand) = user_id.parts();
    println!("  Parts: timestamp={timestamp}, random={rand}");

    // Convert to different formats
    let as_u128 = user_id.as_u128();
    let as_bytes = user_id.to_bytes();
    println!("  As u128: {as_u128}");
    println!("  As bytes: {:?}", &as_bytes[..4]); // Show first 4 bytes

    // Check if nil
    let default_id = UserId::default();
    println!("\n  Is default ID nil? {}", default_id.is_nil());
    println!("  Is regular ID nil? {}", user_id.is_nil());

    // All of these methods are called directly on UserId without needing to
    // extract the inner Nulid first - this is the power of Deref!

    println!("\n=== Example Complete ===");
    Ok(())
}

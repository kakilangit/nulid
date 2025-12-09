//! Example demonstrating NULID serialization with serde.
//!
//! Run with: `cargo run --example serde_example --features serde`

#![allow(clippy::unwrap_used)]
#![allow(clippy::too_many_lines)]

#[cfg(feature = "serde")]
use nulid::Nulid;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "serde")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Define structs at the top of function
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct User {
        id: Nulid,
        name: String,
        email: String,
        created_at: Nulid,
    }

    #[derive(Debug, Serialize, Deserialize)]
    struct Database {
        users: Vec<User>,
        total_count: usize,
    }

    println!("NULID Serde Integration Example");
    println!("================================\n");

    // Create a user with NULID identifiers
    println!("1. Creating a User struct with NULID fields...");
    let user = User {
        id: Nulid::new()?,
        name: "Alice Johnson".to_string(),
        email: "alice@example.com".to_string(),
        created_at: Nulid::new()?,
    };
    let user_id = user.id;
    let created_at = user.created_at;
    println!("   User ID: {user_id}");
    println!("   Created At: {created_at}");
    println!();

    // Serialize to JSON
    println!("2. Serializing to JSON...");
    let json = serde_json::to_string_pretty(&user)?;
    println!("   JSON:\n{json}");
    println!();

    // Deserialize from JSON
    println!("3. Deserializing from JSON...");
    let deserialized: User = serde_json::from_str(&json)?;
    let deser_id = deserialized.id;
    println!("   Deserialized User ID: {deser_id}");
    let match_result = if user == deserialized { "✓" } else { "✗" };
    println!("   Match: {match_result}");
    println!();

    println!("4. Working with collections...");
    let mut users = Vec::new();
    for i in 0..3 {
        users.push(User {
            id: Nulid::new()?,
            name: format!("User {}", i + 1),
            email: format!("user{}@example.com", i + 1),
            created_at: Nulid::new()?,
        });
    }

    let db = Database {
        total_count: users.len(),
        users,
    };

    let db_json = serde_json::to_string_pretty(&db)?;
    println!("   Database JSON:\n{db_json}");
    println!();

    // Deserialize the database
    println!("5. Deserializing database...");
    let deserialized_db: Database = serde_json::from_str(&db_json)?;
    let total_count = deserialized_db.total_count;
    println!("   Total users: {total_count}");
    for (i, user_item) in deserialized_db.users.iter().enumerate() {
        let idx = i + 1;
        let name = &user_item.name;
        let id = user_item.id;
        println!("   [{idx}] {name} - {id}");
    }
    println!();

    // Demonstrate MessagePack serialization
    println!("6. MessagePack serialization...");
    let msgpack_bytes = rmp_serde::to_vec(&user)?;
    let bytes_len = msgpack_bytes.len();
    println!("   MessagePack size: {bytes_len} bytes");
    let msgpack_user: User = rmp_serde::from_slice(&msgpack_bytes)?;
    let msgpack_id = msgpack_user.id;
    println!("   Deserialized ID: {msgpack_id}");
    let match_result = if user.id == msgpack_user.id {
        "✓"
    } else {
        "✗"
    };
    println!("   Match: {match_result}");
    println!();

    // Demonstrate TOML serialization
    println!("7. TOML serialization...");
    let toml_str = toml::to_string_pretty(&user)?;
    println!("   TOML:\n{toml_str}");

    // TOML deserialization may have issues with string lifetimes in some versions
    match toml::from_str::<User>(&toml_str) {
        Ok(toml_user) => {
            let toml_match = if user == toml_user { "✓" } else { "✗" };
            println!("   Deserialized successfully");
            println!("   Match: {toml_match}");
        }
        Err(e) => {
            println!("   Note: TOML deserialization has known issues with some types");
            println!("   Error details: {e}");
            println!("   Serialization to TOML works, deserialization skipped");
        }
    }
    println!();

    // Demonstrate sorting with serialized data
    println!("8. Sorting NULIDs maintains order after serialization...");
    let mut nulids = [
        Nulid::new()?,
        Nulid::new()?,
        Nulid::new()?,
        Nulid::new()?,
        Nulid::new()?,
    ];

    // Sort NULIDs
    nulids.sort();
    println!("   Sorted NULIDs:");
    for (i, id) in nulids.iter().enumerate() {
        let idx = i + 1;
        println!("     [{idx}] {id}");
    }
    println!();

    // Serialize the sorted NULIDs to JSON
    let json_ids: Vec<String> = nulids
        .iter()
        .filter_map(|id| serde_json::to_string(id).ok())
        .collect();

    // Verify JSON strings maintain sort order
    let mut json_ids_sorted = json_ids.clone();
    json_ids_sorted.sort();
    let order_maintained = if json_ids == json_ids_sorted {
        "✓"
    } else {
        "✗"
    };
    println!("   JSON serialized strings maintain sort order: {order_maintained}");
    println!();

    println!("All serde examples completed successfully! ✓");

    Ok(())
}

#[cfg(not(feature = "serde"))]
fn main() {
    eprintln!("This example requires the 'serde' feature.");
    eprintln!("Run with: cargo run --example serde_example --features serde");
    std::process::exit(1);
}

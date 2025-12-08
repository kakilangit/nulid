//! Example demonstrating NULID serialization with serde.
//!
//! Run with: cargo run --example serde_example --features serde

#[cfg(feature = "serde")]
use nulid::Nulid;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "serde")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("NULID Serde Integration Example");
    println!("================================\n");

    // Define a struct that uses NULID
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct User {
        id: Nulid,
        name: String,
        email: String,
        created_at: Nulid,
    }

    // Create a user with NULID identifiers
    println!("1. Creating a User struct with NULID fields...");
    let user = User {
        id: Nulid::new()?,
        name: "Alice Johnson".to_string(),
        email: "alice@example.com".to_string(),
        created_at: Nulid::new()?,
    };
    println!("   User ID: {}", user.id);
    println!("   Created At: {}", user.created_at);
    println!();

    // Serialize to JSON
    println!("2. Serializing to JSON...");
    let json = serde_json::to_string_pretty(&user)?;
    println!("   JSON:\n{}", json);
    println!();

    // Deserialize from JSON
    println!("3. Deserializing from JSON...");
    let deserialized: User = serde_json::from_str(&json)?;
    println!("   Deserialized User ID: {}", deserialized.id);
    println!("   Match: {}", if user == deserialized { "✓" } else { "✗" });
    println!();

    // Demonstrate with a collection
    #[derive(Debug, Serialize, Deserialize)]
    struct Database {
        users: Vec<User>,
        total_count: usize,
    }

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
    println!("   Database JSON:\n{}", db_json);
    println!();

    // Deserialize the database
    println!("5. Deserializing database...");
    let deserialized_db: Database = serde_json::from_str(&db_json)?;
    println!("   Total users: {}", deserialized_db.total_count);
    for (i, user) in deserialized_db.users.iter().enumerate() {
        println!("   [{}] {} - {}", i + 1, user.name, user.id);
    }
    println!();

    // Demonstrate MessagePack serialization
    println!("6. MessagePack serialization...");
    let msgpack_bytes = rmp_serde::to_vec(&user)?;
    println!("   MessagePack size: {} bytes", msgpack_bytes.len());
    let msgpack_user: User = rmp_serde::from_slice(&msgpack_bytes)?;
    println!("   Deserialized ID: {}", msgpack_user.id);
    println!(
        "   Match: {}",
        if user.id == msgpack_user.id {
            "✓"
        } else {
            "✗"
        }
    );
    println!();

    // Demonstrate TOML serialization
    println!("7. TOML serialization...");
    let toml_str = toml::to_string_pretty(&user)?;
    println!("   TOML:\n{}", toml_str);
    let toml_user: User = toml::from_str(&toml_str)?;
    println!("   Match: {}", if user == toml_user { "✓" } else { "✗" });
    println!();

    // Demonstrate sorting with serialized data
    println!("8. Sorting NULIDs maintains order after serialization...");
    let mut nulids = vec![
        Nulid::new()?,
        Nulid::new()?,
        Nulid::new()?,
        Nulid::new()?,
        Nulid::new()?,
    ];

    // Serialize each
    let json_ids: Vec<String> = nulids
        .iter()
        .map(|id| serde_json::to_string(id).unwrap())
        .collect();

    // Sort NULIDs
    nulids.sort();
    println!("   Sorted NULIDs:");
    for (i, id) in nulids.iter().enumerate() {
        println!("     [{}] {}", i + 1, id);
    }
    println!();

    // Verify JSON strings also sort correctly
    let mut json_ids_sorted = json_ids.clone();
    json_ids_sorted.sort();
    println!(
        "   JSON strings maintain sort order: {}",
        if json_ids == json_ids_sorted {
            "✓"
        } else {
            "✗"
        }
    );
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

//! Example demonstrating feature-gated traits for the `Id` derive macro.
//!
//! This example shows how the `Id` derive macro automatically implements
//! feature-gated traits like `serde`, `uuid`, `sqlx`, and `postgres-types`
//! when their corresponding features are enabled.
//!
//! Run with: cargo run --example derive_features --features derive,serde,uuid,sqlx,postgres-types

#![allow(
    clippy::expect_used,
    clippy::similar_names,
    clippy::items_after_statements,
    clippy::too_many_lines,
    clippy::doc_markdown
)]

use nulid::{Nulid, Result};
use nulid_derive::Id;

#[derive(Id)]
pub struct UserId(Nulid);

#[derive(Id)]
pub struct OrderId(pub Nulid);

fn main() -> Result<()> {
    println!("=== Id Derive Macro - Feature-Gated Traits ===\n");

    let user_id = UserId::new()?;
    println!("Generated UserId: {user_id}\n");

    // Feature: serde
    #[cfg(feature = "serde")]
    {
        println!("--- Serde Support ---");

        // Serialize to JSON
        let json = serde_json::to_string(&user_id).expect("Failed to serialize to JSON");
        println!("Serialized to JSON: {json}");

        // Deserialize from JSON
        let deserialized: UserId =
            serde_json::from_str(&json).expect("Failed to deserialize from JSON");
        println!("Deserialized from JSON: {deserialized}");
        assert_eq!(user_id, deserialized);

        // Binary serialization with bincode
        let encoded = bincode::serde::encode_to_vec(user_id, bincode::config::standard())
            .expect("Failed to encode with bincode");
        println!(
            "Encoded with bincode ({} bytes): {:?}...",
            encoded.len(),
            &encoded[..8]
        );

        let (decoded, _): (UserId, usize) =
            bincode::serde::decode_from_slice(&encoded, bincode::config::standard())
                .expect("Failed to decode with bincode");
        println!("Decoded with bincode: {decoded}");
        assert_eq!(user_id, decoded);

        println!();
    }

    // Feature: uuid
    #[cfg(feature = "uuid")]
    {
        println!("--- UUID Support ---");

        // Convert to UUID
        let uuid = user_id.to_uuid();
        println!("Converted to UUID: {uuid}");

        // Convert from UUID
        let from_uuid = UserId::from_uuid(uuid);
        println!("Converted from UUID: {from_uuid}");
        assert_eq!(user_id, from_uuid);

        // Using From trait
        let uuid2: uuid::Uuid = user_id.into();
        println!("Using Into<Uuid>: {uuid2}");
        assert_eq!(uuid, uuid2);

        // Using From trait (reverse)
        let user_id2: UserId = uuid.into();
        println!("Using From<Uuid>: {user_id2}");
        assert_eq!(user_id, user_id2);

        println!();
    }

    // Feature: sqlx
    #[cfg(feature = "sqlx")]
    {
        println!("--- SQLx Support ---");

        use sqlx::{Type, TypeInfo};

        // Type information
        let type_info = <UserId as Type<sqlx::Postgres>>::type_info();
        println!("PostgreSQL type: {}", type_info.name());

        // Encode/Decode would require a database connection, so we just show the trait is implemented
        println!("✓ Implements Type<Postgres>");
        println!("✓ Implements Encode<Postgres>");
        println!("✓ Implements Decode<Postgres>");
        println!("✓ Implements PgHasArrayType");

        println!();
    }

    // Feature: postgres-types
    #[cfg(feature = "postgres-types")]
    {
        println!("--- postgres-types Support ---");

        use bytes::BytesMut;
        use postgres_types::{FromSql, ToSql, Type as PgType};

        let pg_type = PgType::UUID;

        // Serialize to PostgreSQL format
        let mut buf = BytesMut::new();
        user_id
            .to_sql(&pg_type, &mut buf)
            .expect("Failed to serialize");
        println!("Serialized to PostgreSQL ({} bytes)", buf.len());

        // Deserialize from PostgreSQL format
        let deserialized = UserId::from_sql(&pg_type, &buf).expect("Failed to deserialize");
        println!("Deserialized from PostgreSQL: {deserialized}");
        assert_eq!(user_id, deserialized);

        println!("✓ Implements ToSql");
        println!("✓ Implements FromSql");

        println!();
    }

    // Note: rkyv support
    println!("--- rkyv Support ---");
    println!("For rkyv support, manually add derive attributes to your wrapper type:");
    println!("#[derive(Id)]");
    println!(
        "#[cfg_attr(feature = \"rkyv\", derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize))]"
    );
    println!("pub struct UserId(Nulid);");
    println!();

    println!("--- Multiple ID Types with Features ---\n");

    let order_id = OrderId::new()?;
    println!("OrderId: {order_id}");

    #[cfg(feature = "serde")]
    {
        let order_json = serde_json::to_string(&order_id).expect("Failed to serialize");
        println!("OrderId as JSON: {order_json}");
    }

    #[cfg(feature = "uuid")]
    {
        let order_uuid = order_id.to_uuid();
        println!("OrderId as UUID: {order_uuid}");
    }

    println!("\n=== Example Complete ===");

    #[cfg(not(all(
        feature = "serde",
        feature = "uuid",
        feature = "sqlx",
        feature = "postgres-types"
    )))]
    {
        println!(
            "\nNote: Not all features are enabled. Run with --features derive,serde,uuid,sqlx,postgres-types to see all examples."
        );
    }

    Ok(())
}

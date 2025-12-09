//! Example demonstrating NULID usage with `SQLx` and `PostgreSQL`.
//!
//! This example shows how to:
//! - Store NULIDs as UUIDs in `PostgreSQL`
//! - Query records by NULID
//! - Use NULID in structs with `sqlx::FromRow`
//! - Leverage NULID's sortability for time-ordered queries
//!
//! # Setup
//!
//! 1. Install `PostgreSQL` and create a database:
//!    ```bash
//!    createdb nulid_example
//!    ```
//!
//! 2. Set the `DATABASE_URL` environment variable:
//!    ```bash
//!    export DATABASE_URL="postgresql://localhost/nulid_example"
//!    ```
//!
//! 3. Run the example:
//!    ```bash
//!    cargo run --example sqlx_postgres --features sqlx
//!    ```
//!
//! # Schema
//!
//! ```sql
//! CREATE TABLE users (
//!     id UUID PRIMARY KEY,
//!     name TEXT NOT NULL,
//!     email TEXT NOT NULL,
//!     created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
//! );
//!
//! CREATE TABLE events (
//!     id UUID PRIMARY KEY,
//!     user_id UUID NOT NULL REFERENCES users(id),
//!     event_type TEXT NOT NULL,
//!     payload JSONB,
//!     created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
//! );
//!
//! CREATE INDEX idx_events_user_id ON events(user_id);
//! CREATE INDEX idx_events_created_at ON events(created_at);
//! ```

#[cfg(feature = "sqlx")]
use nulid::Nulid;
#[cfg(feature = "sqlx")]
use sqlx::postgres::PgPoolOptions;
#[cfg(feature = "sqlx")]
use sqlx::{PgPool, Row};

#[cfg(feature = "sqlx")]
#[derive(Debug, sqlx::FromRow)]
#[allow(dead_code)]
struct User {
    id: Nulid,
    name: String,
    email: String,
}

#[cfg(feature = "sqlx")]
#[derive(Debug, sqlx::FromRow)]
#[allow(dead_code)]
#[allow(clippy::struct_field_names)]
struct Event {
    id: Nulid,
    user_id: Nulid,
    event_type: String,
    payload: Option<serde_json::Value>,
}

#[cfg(feature = "sqlx")]
async fn setup_database(pool: &PgPool) -> Result<(), sqlx::Error> {
    // Create users table
    sqlx::query(
        r"
        CREATE TABLE IF NOT EXISTS users (
            id UUID PRIMARY KEY,
            name TEXT NOT NULL,
            email TEXT NOT NULL,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )
        ",
    )
    .execute(pool)
    .await?;

    // Create events table
    sqlx::query(
        r"
        CREATE TABLE IF NOT EXISTS events (
            id UUID PRIMARY KEY,
            user_id UUID NOT NULL REFERENCES users(id),
            event_type TEXT NOT NULL,
            payload JSONB,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )
        ",
    )
    .execute(pool)
    .await?;

    // Create indexes
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_events_user_id ON events(user_id)")
        .execute(pool)
        .await?;

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_events_created_at ON events(created_at)")
        .execute(pool)
        .await?;

    Ok(())
}

#[cfg(feature = "sqlx")]
async fn insert_user(pool: &PgPool, id: Nulid, name: &str, email: &str) -> Result<(), sqlx::Error> {
    sqlx::query("INSERT INTO users (id, name, email) VALUES ($1, $2, $3)")
        .bind(id)
        .bind(name)
        .bind(email)
        .execute(pool)
        .await?;

    println!("✓ Inserted user: {name} ({id})");
    Ok(())
}

#[cfg(feature = "sqlx")]
async fn get_user(pool: &PgPool, id: Nulid) -> Result<User, sqlx::Error> {
    sqlx::query_as::<_, User>("SELECT id, name, email FROM users WHERE id = $1")
        .bind(id)
        .fetch_one(pool)
        .await
}

#[cfg(feature = "sqlx")]
async fn insert_event(
    pool: &PgPool,
    id: Nulid,
    user_id: Nulid,
    event_type: &str,
    payload: Option<serde_json::Value>,
) -> Result<(), sqlx::Error> {
    sqlx::query("INSERT INTO events (id, user_id, event_type, payload) VALUES ($1, $2, $3, $4)")
        .bind(id)
        .bind(user_id)
        .bind(event_type)
        .bind(payload)
        .execute(pool)
        .await?;

    println!("✓ Inserted event: {event_type} for user {user_id}");
    Ok(())
}

#[cfg(feature = "sqlx")]
async fn get_user_events(pool: &PgPool, user_id: Nulid) -> Result<Vec<Event>, sqlx::Error> {
    sqlx::query_as::<_, Event>(
        "SELECT id, user_id, event_type, payload FROM events WHERE user_id = $1 ORDER BY id",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
}

#[cfg(feature = "sqlx")]
async fn get_recent_events(pool: &PgPool, limit: i64) -> Result<Vec<Event>, sqlx::Error> {
    sqlx::query_as::<_, Event>(
        "SELECT id, user_id, event_type, payload FROM events ORDER BY id DESC LIMIT $1",
    )
    .bind(limit)
    .fetch_all(pool)
    .await
}

#[cfg(feature = "sqlx")]
async fn count_users(pool: &PgPool) -> Result<i64, sqlx::Error> {
    let row = sqlx::query("SELECT COUNT(*) as count FROM users")
        .fetch_one(pool)
        .await?;
    Ok(row.get("count"))
}

#[cfg(feature = "sqlx")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 NULID + SQLx + PostgreSQL Example\n");

    // Get database URL from environment
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://localhost/nulid_example".to_string());

    println!("📡 Connecting to database: {database_url}\n");

    // Create connection pool
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    // Setup database schema
    println!("🔧 Setting up database schema...");
    setup_database(&pool).await?;
    println!();

    // Generate NULIDs for users
    println!("📝 Creating users...");
    let user1_id = Nulid::new()?;
    let user2_id = Nulid::new()?;

    insert_user(&pool, user1_id, "Alice Smith", "alice@example.com").await?;
    insert_user(&pool, user2_id, "Bob Jones", "bob@example.com").await?;
    println!();

    // Retrieve user
    println!("🔍 Fetching user...");
    let user = get_user(&pool, user1_id).await?;
    println!("✓ Found user: {user:?}\n");

    // Generate events with NULIDs (naturally sorted by time)
    println!("📊 Creating events...");
    for i in 0..5 {
        let event_id = Nulid::new()?;
        let event_type = if i % 2 == 0 { "login" } else { "page_view" };
        let payload = serde_json::json!({
            "ip": "192.168.1.1",
            "user_agent": "Mozilla/5.0",
            "index": i
        });

        insert_event(&pool, event_id, user1_id, event_type, Some(payload)).await?;

        // Small delay to ensure different timestamps
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    }

    // Create events for second user
    for _i in 0..3 {
        let event_id = Nulid::new()?;
        insert_event(&pool, event_id, user2_id, "api_call", None).await?;
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    }
    println!();

    // Query user events (sorted by NULID = sorted by time)
    println!("📋 Fetching user events (sorted by NULID)...");
    let events = get_user_events(&pool, user1_id).await?;
    for (i, event) in events.iter().enumerate() {
        println!("  Event {}: {} at {}", i + 1, event.event_type, event.id);
    }
    println!();

    // Query recent events
    println!("🕐 Fetching recent events (DESC)...");
    let recent = get_recent_events(&pool, 5).await?;
    for event in &recent {
        println!(
            "  {} - {} (user: {})",
            event.id, event.event_type, event.user_id
        );
    }
    println!();

    // Count users
    let user_count = count_users(&pool).await?;
    println!("👥 Total users: {user_count}\n");

    // Demonstrate NULID -> UUID conversion
    println!("🔄 NULID ↔ UUID Conversion:");
    println!("  NULID:  {user1_id}");
    println!("  UUID:   {}", user1_id.to_uuid());
    println!("  Stored as UUID in PostgreSQL, queried as NULID in Rust!");
    println!();

    // Demonstrate sortability
    println!("✨ NULID Benefits:");
    println!("  ✓ Stored as native UUID in PostgreSQL (16 bytes)");
    println!("  ✓ Automatically sorted by creation time");
    println!("  ✓ No need for separate created_at columns for ordering");
    println!("  ✓ Nanosecond precision prevents collisions");
    println!("  ✓ Compatible with existing UUID-based systems");
    println!();

    // Cleanup
    println!("🧹 Cleaning up...");
    sqlx::query("DROP TABLE IF EXISTS events CASCADE")
        .execute(&pool)
        .await?;
    sqlx::query("DROP TABLE IF EXISTS users CASCADE")
        .execute(&pool)
        .await?;
    println!("✓ Tables dropped\n");

    println!("✅ Example completed successfully!");

    Ok(())
}

#[cfg(not(feature = "sqlx"))]
fn main() {
    println!("This example requires the 'sqlx' feature to be enabled.");
    println!("Run with: cargo run --example sqlx_postgres --features sqlx");
}

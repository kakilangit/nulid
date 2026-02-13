//! Example demonstrating NULID usage with `SQLx` and `SQLite`.
//!
//! This example shows how to:
//! - Store NULIDs as BLOBs in `SQLite`
//! - Query records by NULID
//! - Use NULID in structs with `sqlx::FromRow`
//! - Leverage NULID's sortability for time-ordered queries
//!
//! # Setup
//!
//! 1. This example uses an in-memory `SQLite` database, so no additional setup is needed.
//!
//! 2. Run the example:
//!    ```bash
//!    cargo run --example sqlx_sqlite --features sqlx-sqlite
//!    ```
//!
//! # Schema
//!
//! ```sql
//! CREATE TABLE users (
//!     id BLOB PRIMARY KEY,
//!     name TEXT NOT NULL,
//!     email TEXT NOT NULL,
//!     created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
//! );
//!
//! CREATE TABLE events (
//!     id BLOB PRIMARY KEY,
//!     user_id BLOB NOT NULL REFERENCES users(id),
//!     event_type TEXT NOT NULL,
//!     payload TEXT,
//!     created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
//! );
//!
//! CREATE INDEX idx_events_user_id ON events(user_id);
//! CREATE INDEX idx_events_created_at ON events(created_at);
//! ```

use nulid::Nulid;
use sqlx::{Row, SqlitePool, sqlite::SqlitePoolOptions};

#[derive(Debug, sqlx::FromRow)]
#[allow(dead_code)]
struct User {
    id: Nulid,
    name: String,
    email: String,
}

#[derive(Debug, sqlx::FromRow)]
#[allow(dead_code)]
#[allow(clippy::struct_field_names)]
struct Event {
    id: Nulid,
    user_id: Nulid,
    event_type: String,
    payload: Option<String>,
}

async fn setup_database(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    // Create users table
    sqlx::query(
        r"
        CREATE TABLE IF NOT EXISTS users (
            id BLOB PRIMARY KEY,
            name TEXT NOT NULL,
            email TEXT NOT NULL,
            created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
        )
        ",
    )
    .execute(pool)
    .await?;

    // Create events table
    sqlx::query(
        r"
        CREATE TABLE IF NOT EXISTS events (
            id BLOB PRIMARY KEY,
            user_id BLOB NOT NULL REFERENCES users(id),
            event_type TEXT NOT NULL,
            payload TEXT,
            created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
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

async fn insert_user(
    pool: &SqlitePool,
    id: Nulid,
    name: &str,
    email: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query("INSERT INTO users (id, name, email) VALUES (?, ?, ?)")
        .bind(id)
        .bind(name)
        .bind(email)
        .execute(pool)
        .await?;

    println!("✓ Inserted user: {name} ({id})");
    Ok(())
}

async fn get_user(pool: &SqlitePool, id: Nulid) -> Result<User, sqlx::Error> {
    sqlx::query_as::<_, User>("SELECT id, name, email FROM users WHERE id = ?")
        .bind(id)
        .fetch_one(pool)
        .await
}

async fn insert_event(
    pool: &SqlitePool,
    id: Nulid,
    user_id: Nulid,
    event_type: &str,
    payload: Option<String>,
) -> Result<(), sqlx::Error> {
    sqlx::query("INSERT INTO events (id, user_id, event_type, payload) VALUES (?, ?, ?, ?)")
        .bind(id)
        .bind(user_id)
        .bind(event_type)
        .bind(payload)
        .execute(pool)
        .await?;

    println!("✓ Inserted event: {event_type} for user {user_id}");
    Ok(())
}

async fn get_user_events(pool: &SqlitePool, user_id: Nulid) -> Result<Vec<Event>, sqlx::Error> {
    sqlx::query_as::<_, Event>(
        "SELECT id, user_id, event_type, payload FROM events WHERE user_id = ? ORDER BY id",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
}

async fn get_recent_events(pool: &SqlitePool, limit: i64) -> Result<Vec<Event>, sqlx::Error> {
    sqlx::query_as::<_, Event>(
        "SELECT id, user_id, event_type, payload FROM events ORDER BY id DESC LIMIT ?",
    )
    .bind(limit)
    .fetch_all(pool)
    .await
}

async fn count_users(pool: &SqlitePool) -> Result<i64, sqlx::Error> {
    let row = sqlx::query("SELECT COUNT(*) as count FROM users")
        .fetch_one(pool)
        .await?;
    Ok(row.get("count"))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn core::error::Error>> {
    println!("🚀 NULID + SQLx + SQLite Example\n");

    println!("📡 Connecting to in-memory SQLite database...\n");

    // Create connection pool for in-memory database
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(":memory:")
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
        let payload =
            format!("{{\"ip\": \"192.168.1.1\", \"user_agent\": \"Mozilla/5.0\", \"index\": {i}}}");

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

    // Demonstrate NULID info
    println!("🔄 NULID Info:");
    println!("  NULID:  {user1_id}");
    println!("  Bytes:  {:?}", user1_id.to_bytes());
    println!("  Stored as BLOB in SQLite, queried as NULID in Rust!");
    println!();

    // Demonstrate sortability
    println!("✨ NULID Benefits:");
    println!("  ✓ Stored as BLOB in SQLite (16 bytes)");
    println!("  ✓ Automatically sorted by creation time");
    println!("  ✓ No need for separate created_at columns for ordering");
    println!("  ✓ Nanosecond precision prevents collisions");
    println!("  ✓ Compatible with existing UUID-based systems via conversion");
    println!();

    // Cleanup
    println!("🧹 Cleaning up...");
    sqlx::query("DROP TABLE IF EXISTS events")
        .execute(&pool)
        .await?;
    sqlx::query("DROP TABLE IF EXISTS users")
        .execute(&pool)
        .await?;
    println!("✓ Tables dropped\n");

    println!("✅ Example completed successfully!");

    Ok(())
}

#[cfg(not(feature = "sqlx-sqlite"))]
fn main() {
    println!("This example requires the 'sqlx-sqlite' feature to be enabled.");
    println!("Run with: cargo run --example sqlx_sqlite --features sqlx-sqlite");
}

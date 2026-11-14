//! Example demonstrating NULIDs stored as 16-byte `BYTEA` in `PostgreSQL`.
//!
//! This is the `sqlx-postgres-bytea` counterpart of the `sqlx_postgres`
//! example: same flow, but the id columns are `BYTEA(16)` instead of `UUID`,
//! which is what `sqlx-postgres-bytea` maps `Nulid` to. It shows how to:
//! - Store NULIDs as raw 16-byte values in `PostgreSQL`
//! - Query records by NULID
//! - Verify the stored bytes are exactly `to_bytes()` (no UUID shuffling)
//! - Leverage NULID's sortability for time-ordered queries
//!
//! # Security Notice
//!
//! This example uses a default database URL without authentication for local
//! development convenience. **This is NOT suitable for production use.**
//!
//! In production environments, you MUST:
//! - Use strong authentication (username/password or certificates)
//! - Enable SSL/TLS connections (`sslmode=require`)
//! - Apply the principle of least privilege for database permissions
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
//!    cargo run --example sqlx_postgres_bytea --features sqlx-postgres-bytea
//!    ```
//!
//! # Schema
//!
//! ```sql
//! CREATE TABLE users (
//!     id BYTEA PRIMARY KEY,
//!     name TEXT NOT NULL,
//!     created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
//! );
//!
//! CREATE TABLE events (
//!     id BYTEA PRIMARY KEY,
//!     user_id BYTEA NOT NULL REFERENCES users(id),
//!     event_type TEXT NOT NULL,
//!     created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
//! );
//! ```

use nulid::Nulid;
use sqlx::postgres::PgPoolOptions;
use sqlx::{PgPool, Row};

#[derive(Debug, sqlx::FromRow)]
#[allow(dead_code)]
struct User {
    id: Nulid,
    name: String,
}

#[derive(Debug, sqlx::FromRow)]
#[allow(dead_code)]
#[allow(clippy::struct_field_names)]
struct Event {
    id: Nulid,
    user_id: Nulid,
    event_type: String,
}

async fn setup_database(pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::query(
        r"
        CREATE TABLE IF NOT EXISTS users (
            id BYTEA PRIMARY KEY,
            name TEXT NOT NULL,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )
        ",
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r"
        CREATE TABLE IF NOT EXISTS events (
            id BYTEA PRIMARY KEY,
            user_id BYTEA NOT NULL REFERENCES users(id),
            event_type TEXT NOT NULL,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )
        ",
    )
    .execute(pool)
    .await?;

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_events_user_id ON events(user_id)")
        .execute(pool)
        .await?;

    Ok(())
}

async fn insert_user(pool: &PgPool, id: Nulid, name: &str) -> Result<(), sqlx::Error> {
    sqlx::query("INSERT INTO users (id, name) VALUES ($1, $2)")
        .bind(id)
        .bind(name)
        .execute(pool)
        .await?;
    println!("Inserted user: {name}");
    Ok(())
}

async fn get_user(pool: &PgPool, id: Nulid) -> Result<User, sqlx::Error> {
    sqlx::query_as::<_, User>("SELECT id, name FROM users WHERE id = $1")
        .bind(id)
        .fetch_one(pool)
        .await
}

async fn insert_event(
    pool: &PgPool,
    id: Nulid,
    user_id: Nulid,
    event_type: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query("INSERT INTO events (id, user_id, event_type) VALUES ($1, $2, $3)")
        .bind(id)
        .bind(user_id)
        .bind(event_type)
        .execute(pool)
        .await?;
    println!("Inserted event: {event_type}");
    Ok(())
}

async fn get_user_events(pool: &PgPool, user_id: Nulid) -> Result<Vec<Event>, sqlx::Error> {
    sqlx::query_as::<_, Event>(
        "SELECT id, user_id, event_type FROM events WHERE user_id = $1 ORDER BY id",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
}

/// Proves the stored bytes are exactly `to_bytes()`: reads the id back as
/// raw hex from PostgreSQL (`encode(id, 'hex')`) and compares it with the
/// hex form of `to_bytes()` built on the Rust side.
async fn stored_bytes_match(pool: &PgPool, id: Nulid) -> Result<bool, sqlx::Error> {
    let stored: String = sqlx::query_scalar("SELECT encode(id, 'hex') FROM users WHERE id = $1")
        .bind(id)
        .fetch_one(pool)
        .await?;
    let expected: String = id
        .to_bytes()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    Ok(stored == expected)
}

#[tokio::main]
#[cfg(feature = "sqlx-postgres-bytea")]
async fn main() -> Result<(), Box<dyn core::error::Error>> {
    println!("NULID + SQLx + PostgreSQL (BYTEA storage) Example\n");

    let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        eprintln!("WARNING: Using default database URL for local development only!");
        "postgresql://localhost/nulid_example".to_string()
    });

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    println!("Setting up database schema...");
    setup_database(&pool).await?;
    println!();

    println!("Creating a user...");
    let user_id = Nulid::new()?;
    insert_user(&pool, user_id, "Alice Smith").await?;

    println!("Fetching user (BYTEA -> Nulid decode)...");
    let user = get_user(&pool, user_id).await?;
    println!("Found user: {user:?}\n");

    println!("Verifying the stored bytes are exactly to_bytes()...");
    if stored_bytes_match(&pool, user_id).await? {
        println!("  stored bytes == to_bytes(): OK\n");
    } else {
        println!("  stored bytes differ from to_bytes()!\n");
    }

    println!("Creating time-ordered events...");
    for i in 0..5 {
        let event_id = Nulid::new()?;
        let event_type = if i % 2 == 0 { "login" } else { "page_view" };
        insert_event(&pool, event_id, user_id, event_type).await?;
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    }
    println!();

    println!("Fetching user events (sorted by NULID = sorted by time)...");
    let events = get_user_events(&pool, user_id).await?;
    for (i, event) in events.iter().enumerate() {
        println!("  Event {}: {} at {}", i + 1, event.event_type, event.id);
    }
    println!();

    println!("Benefits of BYTEA storage:");
    println!("  - Stored as raw 16 bytes (no UUID byte-order convention)");
    println!("  - Automatically sorted by creation time");
    println!("  - Plays well with schemas that already use bytea(16) ids");
    println!();

    println!("Cleaning up...");
    sqlx::query("DROP TABLE IF EXISTS events CASCADE")
        .execute(&pool)
        .await?;
    sqlx::query("DROP TABLE IF EXISTS users CASCADE")
        .execute(&pool)
        .await?;
    println!("Tables dropped\n");

    println!("Example completed successfully!");
    Ok(())
}

#[cfg(not(feature = "sqlx-postgres-bytea"))]
fn main() {
    println!("This example requires the 'sqlx-postgres-bytea' feature to be enabled.");
    println!("Run with: cargo run --example sqlx_postgres_bytea --features sqlx-postgres-bytea");
}

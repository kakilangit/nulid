# nulid_derive

Derive macros for types that wrap `Nulid`.

This crate provides procedural macros to automatically implement common traits for newtype wrappers around `Nulid`, eliminating boilerplate code.

## Features

### Core Traits

The `Id` derive macro automatically implements:

- `TryFrom<String>` - Parse from owned String
- `TryFrom<&str>` - Parse from string slice
- `From<Nulid>` - Create wrapper from Nulid
- `From<WrapperType> for Nulid` - Extract inner Nulid
- `AsRef<Nulid>` - Borrow inner Nulid
- `Deref<Target = Nulid>` - Direct access to all Nulid methods
- `DerefMut` - Mutable access to inner Nulid
- `std::fmt::Display` - Format as Base32 string
- `std::fmt::Debug` - Debug formatting
- `std::str::FromStr` - Parse from string using `.parse()`
- `Copy` - Value semantics (automatically provides `Clone`)
- `PartialEq` and `Eq` - Equality comparison with other wrappers
- `PartialEq<Nulid>` - Direct equality comparison with `Nulid`
- `PartialOrd` and `Ord` - Ordering comparison with other wrappers
- `PartialOrd<Nulid>` - Direct ordering comparison with `Nulid`
- `Hash` - Hashing support for collections
- `Default` - Creates a default instance with `Nulid::ZERO`

### Constructor Methods

It also provides:

- `new()` method - Creates a new instance with a freshly generated `Nulid`
- `now()` method - Alias for `new()`
- `nil()` method - Creates a nil/zero instance
- `min()` method - Returns the minimum possible instance
- `max()` method - Returns the maximum possible instance
- `from_datetime(SystemTime)` - Creates from specific time
- `from_nanos(u128, u64)` - Creates from timestamp and random
- `from_u128(u128)` - Creates from raw u128
- `from_bytes([u8; 16])` - Creates from byte array

### Feature-Gated Traits

When the corresponding features are enabled, additional trait implementations are automatically generated:

#### `serde` feature

- `Serialize` - Serialization support for JSON, bincode, etc.
- `Deserialize` - Deserialization support

```toml
[dependencies]
# The 'serde' feature is automatically propagated to nulid_derive
nulid = { version = "0.5", features = ["derive", "serde"] }
```

```rust
use nulid::Id;
use serde::{Serialize, Deserialize};

#[derive(Id)]  // Automatically implements Serialize + Deserialize
pub struct UserId(nulid::Nulid);

fn main() -> nulid::Result<()> {
    let user_id = UserId::new()?;

    // Serialize to JSON
    let json = serde_json::to_string(&user_id)?;

    // Deserialize from JSON
    let parsed: UserId = serde_json::from_str(&json)?;

    Ok(())
}
```

#### `chrono` feature

- `chrono_datetime()` method - Convert to `chrono::DateTime<Utc>`
- `from_chrono_datetime(DateTime<Utc>)` method - Create from chrono DateTime

```toml
[dependencies]
# The 'chrono' feature is automatically propagated to nulid_derive
nulid = { version = "0.5", features = ["derive", "chrono"] }
```

```rust
use nulid::Id;
use chrono::{DateTime, Utc};

#[derive(Id)]  // Automatically implements chrono methods
pub struct UserId(nulid::Nulid);

fn main() -> nulid::Result<()> {
    let user_id = UserId::new()?;

    // Convert to chrono DateTime
    let dt: DateTime<Utc> = user_id.chrono_datetime();
    println!("User ID timestamp: {}", dt);

    // Create from chrono DateTime
    let now = Utc::now();
    let user_id2 = UserId::from_chrono_datetime(now)?;

    Ok(())
}
```

#### `uuid` feature

- `From<uuid::Uuid>` - Convert from UUID
- `Into<uuid::Uuid>` - Convert to UUID
- `to_uuid()` method - Convert to UUID
- `from_uuid(Uuid)` method - Create from UUID

```toml
[dependencies]
# The 'uuid' feature is automatically propagated to nulid_derive
nulid = { version = "0.5", features = ["derive", "uuid"] }
```

```rust
use nulid::Id;
use uuid::Uuid;

#[derive(Id)]  // Automatically implements UUID conversions
pub struct UserId(nulid::Nulid);

fn main() -> nulid::Result<()> {
    let user_id = UserId::new()?;

    // Convert to UUID
    let uuid = user_id.to_uuid();

    // Convert from UUID
    let from_uuid = UserId::from_uuid(uuid);

    // Using From/Into traits
    let uuid2: Uuid = user_id.into();
    let user_id2: UserId = uuid.into();

    Ok(())
}
```

#### `sqlx` feature

- `Type<Postgres>` - PostgreSQL type support
- `Encode<Postgres>` - Encoding for PostgreSQL
- `Decode<Postgres>` - Decoding from PostgreSQL
- `PgHasArrayType` - Array type support

```toml
[dependencies]
# The 'sqlx' feature is automatically propagated to nulid_derive
nulid = { version = "0.5", features = ["derive", "sqlx"] }
sqlx = { version = "0.8", features = ["postgres", "uuid"] }
```

```rust
use nulid::Id;
use sqlx::PgPool;

#[derive(Id)]  // Automatically implements SQLx traits
pub struct UserId(nulid::Nulid);

#[derive(sqlx::FromRow)]
struct User {
    id: UserId,  // Can be used directly in SQLx queries!
    name: String,
}

async fn insert_user(pool: &PgPool, id: UserId, name: &str) -> sqlx::Result<()> {
    sqlx::query("INSERT INTO users (id, name) VALUES ($1, $2)")
        .bind(id)  // UserId can be bound directly
        .bind(name)
        .execute(pool)
        .await?;
    Ok(())
}
```

#### `postgres-types` feature

- `FromSql` - Deserialize from PostgreSQL
- `ToSql` - Serialize to PostgreSQL

```toml
[dependencies]
# The 'postgres-types' feature is automatically propagated to nulid_derive
nulid = { version = "0.5", features = ["derive", "postgres-types"] }
postgres-types = "0.2"
```

```rust
use nulid::Id;
use postgres_types::{ToSql, FromSql};

#[derive(Id)]  // Automatically implements ToSql + FromSql
pub struct UserId(nulid::Nulid);

// Can now be used with the postgres crate
// let row = client.query_one("SELECT id FROM users WHERE id = $1", &[&user_id])?;
```

#### `proto` feature

- `to_proto()` method - Convert to protobuf message
- `from_proto(ProtoNulid)` method - Create from protobuf message
- `From<WrapperType> for ProtoNulid` - Convert to protobuf
- `From<ProtoNulid> for WrapperType` - Convert from protobuf

```toml
[dependencies]
# The 'proto' feature is automatically propagated to nulid_derive
nulid = { version = "0.5", features = ["derive", "proto"] }
prost = "0.14"
```

```rust
use nulid::Id;
use nulid::proto::nulid::Nulid as ProtoNulid;
use prost::Message;

#[derive(Id)]  // Automatically implements protobuf conversions
pub struct UserId(nulid::Nulid);

fn main() -> nulid::Result<()> {
    let user_id = UserId::new()?;

    // Convert to protobuf message
    let proto = user_id.to_proto();

    // Encode to bytes
    let encoded = proto.encode_to_vec();

    // Decode from bytes
    let decoded = ProtoNulid::decode(&*encoded).unwrap();

    // Convert back to UserId
    let user_id2 = UserId::from_proto(decoded);

    // Using From/Into traits
    let proto2: ProtoNulid = user_id.into();
    let user_id3: UserId = proto2.into();

    Ok(())
}
```

### Feature Propagation

**Important**: When you enable the `derive` feature along with other features (like `serde`, `uuid`, `sqlx`, `postgres-types`, `proto`, or `chrono`) on the `nulid` crate, those features are **automatically propagated** to `nulid_derive`. You don't need to enable them separately on both crates.

```toml
#  Correct - features are automatically propagated to nulid_derive
[dependencies]
nulid = { version = "0.5", features = ["derive", "serde", "uuid", "sqlx"] }

#  Not necessary - you don't need to enable features on nulid_derive manually
[dependencies]
nulid = { version = "0.5", features = ["derive", "serde"] }
nulid_derive = { version = "0.5", features = ["serde"] }  # This is redundant
```

This automatic propagation works for all feature-gated traits:

- `serde` → enables `Serialize` and `Deserialize` implementations
- `chrono` → enables chrono `DateTime` conversion methods
- `uuid` → enables UUID conversion traits
- `sqlx` → enables SQLx PostgreSQL traits
- `postgres-types` → enables `FromSql` and `ToSql` traits
- `proto` → enables Protocol Buffers conversion methods

## Basic Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
nulid = { version = "0.5", features = ["derive"] }
```

Then use the derive macro on your wrapper types:

```rust
use nulid::{Nulid, Id};

#[derive(Id)]
pub struct UserId(Nulid);

#[derive(Id)]
pub struct OrderId(pub Nulid);

fn main() -> nulid::Result<()> {
    // Create new ID with fresh NULID
    let user_id = UserId::new()?;

    // Create default ID (ZERO)
    let default_id = UserId::default();

    // Parse from &str
    let user_id2 = UserId::try_from("01H0JQ4VEFSBV974PRXXWEK5ZW")?;

    // Parse from String
    let user_id3 = UserId::try_from("01H0JQ4VEFSBV974PRXXWEK5ZW".to_string())?;

    // Parse using FromStr
    let user_id4: UserId = "01H0JQ4VEFSBV974PRXXWEK5ZW".parse()?;

    // Create from Nulid
    let nulid = Nulid::new()?;
    let order_id = OrderId::from(nulid);

    // Extract inner Nulid
    let back_to_nulid: Nulid = order_id.into();

    // Borrow inner Nulid
    let nulid_ref: &Nulid = order_id.as_ref();

    // Display as string
    println!("User ID: {}", user_id);

    // Direct comparison with Nulid
    assert_eq!(order_id, nulid);
    assert!(order_id <= nulid);

    // Access Nulid methods directly via Deref
    let nanos = user_id.nanos();
    let random = user_id.random();
    let (timestamp, rand) = user_id.parts();
    println!("Timestamp: {}, Random: {}", timestamp, rand);

    Ok(())
}
```

## Direct Access to Nulid Methods

With `Deref` and `DerefMut` traits, wrapper types can directly access all `Nulid` methods without needing to extract or dereference the inner value:

```rust
use nulid::Id;

#[derive(Id)]
pub struct UserId(nulid::Nulid);

fn main() -> nulid::Result<()> {
    let user_id = UserId::new()?;

    // Access timestamp methods directly
    let nanos = user_id.nanos();           // Get nanoseconds
    let micros = user_id.micros();         // Get microseconds
    let millis = user_id.millis();         // Get milliseconds
    let seconds = user_id.seconds();       // Get seconds
    let subsec = user_id.subsec_nanos();   // Get subsecond nanoseconds

    // Access random component
    let random = user_id.random();

    // Get both parts
    let (timestamp, rand) = user_id.parts();

    // Convert to different formats
    let as_u128 = user_id.as_u128();
    let as_bytes = user_id.to_bytes();

    // Check if nil
    let default_id = UserId::default();
    assert!(default_id.is_nil());
    assert!(!user_id.is_nil());

    // All Nulid methods are available directly on UserId!
    Ok(())
}
```

## Requirements

The derive macro requires:

1. The type must be a tuple struct
2. It must have exactly one field
3. That field must be of type `Nulid`

Valid examples:

```rust
#[derive(Id)]
pub struct UserId(Nulid);           //  Private field

#[derive(Id)]
pub struct OrderId(pub Nulid);      //  Public field
```

Invalid examples:

```rust
#[derive(Id)]
pub struct UserId {                 //  Not a tuple struct
    nulid: Nulid,
}

#[derive(Id)]
pub struct UserId(Nulid, String);   //  Multiple fields

#[derive(Id)]
pub struct UserId(String);          //  Wrong type
```

## Type Safety

Using wrapper types provides type safety by preventing accidental mixing of different ID types:

```rust
use nulid::{Nulid, Id};

#[derive(Id)]
pub struct UserId(Nulid);

#[derive(Id)]
pub struct OrderId(Nulid);

fn process_user(id: UserId) { /* ... */ }
fn process_order(id: OrderId) { /* ... */ }

let user_id = UserId::from(Nulid::new()?);
let order_id = OrderId::from(Nulid::new()?);

process_user(user_id);   //  Correct type
// process_user(order_id);  //  Compile error: expected UserId, found OrderId
```

## Error Handling

All parsing methods return `Result<T, nulid::Error>`, allowing proper error handling:

```rust
use nulid::{Error, Id};

#[derive(Id)]
pub struct UserId(nulid::Nulid);

match UserId::try_from("invalid-string") {
    Ok(id) => println!("Parsed: {}", id),
    Err(Error::InvalidLength { expected, found }) => {
        eprintln!("Wrong length: expected {}, got {}", expected, found);
    }
    Err(Error::InvalidChar(ch, pos)) => {
        eprintln!("Invalid character '{}' at position {}", ch, pos);
    }
    Err(e) => eprintln!("Parse error: {}", e),
}
```

## Integration with Other Traits

The derive macro works well with other derive macros and automatically provides feature-gated trait implementations:

```rust
use nulid::{Nulid, Id};

#[derive(Id)]
pub struct UserId(Nulid);

// Standard traits are automatically implemented:
// Debug, Copy (Clone), PartialEq, Eq, Hash, PartialOrd, Ord

// With features enabled, additional traits are automatically implemented:
// - serde feature: Serialize, Deserialize
// - chrono feature: chrono_datetime(), from_chrono_datetime()
// - uuid feature: From<Uuid>, Into<Uuid>
// - sqlx feature: Type<Postgres>, Encode, Decode
// - postgres-types feature: FromSql, ToSql
// - proto feature: to_proto(), from_proto(), From<ProtoNulid>
```

## Combining Multiple Features

You can enable multiple features at once to get all the trait implementations you need:

```toml
[dependencies]
nulid = { version = "0.5", features = ["derive", "serde", "uuid", "sqlx", "chrono", "proto"] }
```

```rust
use nulid::Id;

#[derive(Id)]
pub struct UserId(nulid::Nulid);

// Now UserId automatically implements:
// - All core traits (Debug, Copy, PartialEq, etc.)
// - Serde traits (Serialize, Deserialize)
// - Chrono methods (chrono_datetime(), from_chrono_datetime())
// - UUID conversions (From<Uuid>, Into<Uuid>)
// - SQLx traits (Type<Postgres>, Encode, Decode)
// - Proto methods (to_proto(), from_proto())
// Plus all the constructor methods (new, nil, min, max, etc.)
```

## Examples

See the [examples directory](https://github.com/kakilangit/nulid/tree/main/examples) in the nulid repository for more usage examples.

## License

This project is licensed under the MIT License - see the LICENSE file for details.

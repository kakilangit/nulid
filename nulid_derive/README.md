# nulid_derive

Derive macros for types that wrap `Nulid`.

This crate provides procedural macros to automatically implement common traits for newtype wrappers around `Nulid`, eliminating boilerplate code.

## Features

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

It also provides:

- `new()` method - Creates a new instance with a freshly generated `Nulid`

## Usage

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
pub struct UserId(Nulid);           // ✓ Private field

#[derive(Id)]
pub struct OrderId(pub Nulid);      // ✓ Public field
```

Invalid examples:

```rust
#[derive(Id)]
pub struct UserId {                 // ✗ Not a tuple struct
    nulid: Nulid,
}

#[derive(Id)]
pub struct UserId(Nulid, String);   // ✗ Multiple fields

#[derive(Id)]
pub struct UserId(String);          // ✗ Wrong type
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

process_user(user_id);   // ✓ Correct type
// process_user(order_id);  // ✗ Compile error: expected UserId, found OrderId
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

The derive macro works well with other derive macros:

```rust
use nulid::{Nulid, Id};

#[derive(Id)]
pub struct UserId(Nulid);

// Standard traits are automatically implemented!
// UserId now has: Debug, Copy (Clone), PartialEq, Eq, Hash, PartialOrd, Ord

// You can also add serde support if the serde feature is enabled in nulid
#[cfg(feature = "serde")]
#[derive(Id, serde::Serialize, serde::Deserialize)]
pub struct OrderId(Nulid);
```

## Examples

See the [examples directory](https://github.com/kakilangit/nulid/tree/main/examples) in the nulid repository for more usage examples.

## License

This project is licensed under the MIT License - see the LICENSE file for details.

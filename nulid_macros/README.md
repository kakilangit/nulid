# nulid_macros

Procedural macros for convenient NULID generation.

This crate provides macros to simplify working with NULIDs in your Rust code.

## Features

The `nulid!()` macro provides convenient NULID generation with flexible error handling:

- `nulid!()` - Generate a NULID, panicking on error (for convenience)
- `nulid!(?)` - Generate a NULID, returning `Result<Nulid, Error>` (for error handling)

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
nulid = { version = "0.5", features = ["macros"] }
```

Then use the macro in your code:

```rust
use nulid::nulid;

fn main() {
    // Simple generation (panics on error)
    let id1 = nulid!();
    println!("Generated: {}", id1);

    // Multiple generations
    let id2 = nulid!();
    let id3 = nulid!();

    assert_ne!(id1, id2);
    assert_ne!(id2, id3);
}
```

## Error Handling

Use `nulid!(?)` when you need to handle errors gracefully:

```rust
use nulid::nulid;

fn create_user_id() -> nulid::Result<nulid::Nulid> {
    // Returns Result for explicit error handling
    let id = nulid!(?)?;
    Ok(id)
}

fn main() -> nulid::Result<()> {
    let user_id = create_user_id()?;
    println!("User ID: {}", user_id);

    // Or use with expect
    let order_id = nulid!(?).expect("Failed to generate order ID");
    println!("Order ID: {}", order_id);

    Ok(())
}
```

## When to Use Each Variant

### Use `nulid!()`

- In tests where panicking is acceptable
- In initialization code where failure should stop the program
- When you want concise, readable code and don't need error recovery

```rust
use nulid::nulid;

#[test]
fn test_user_creation() {
    let user_id = nulid!(); // Panic is fine in tests
    // ... rest of test
}

fn main() {
    let config_id = nulid!(); // App initialization
    // ... rest of app
}
```

### Use `nulid!(?)`

- In library code where you want to propagate errors
- In production code that needs graceful error handling
- When integrating with other Result-returning code

```rust
use nulid::nulid;

pub fn create_entity() -> nulid::Result<Entity> {
    let id = nulid!(?)?; // Propagate errors to caller
    Ok(Entity { id, /* ... */ })
}

fn handle_request() -> Result<Response, AppError> {
    let request_id = nulid!(?).map_err(|e| AppError::IdGeneration(e))?;
    // ... process request
}
```

## Comparison with Direct API

The macro provides syntactic sugar over the direct API:

```rust
use nulid::{nulid, Nulid};

// These are equivalent:
let id1 = nulid!();
let id2 = Nulid::new().expect("Failed to generate NULID");

// These are equivalent:
let id3 = nulid!(?)?;
let id4 = Nulid::new()?;
```

The macro makes code more concise and readable, especially when generating multiple IDs:

```rust
// With macro
let (user_id, session_id, request_id) = (nulid!(), nulid!(), nulid!());

// Without macro
let user_id = Nulid::new().expect("Failed to generate NULID");
let session_id = Nulid::new().expect("Failed to generate NULID");
let request_id = Nulid::new().expect("Failed to generate NULID");
```

## Performance

The macro has zero runtime overhead - it expands to direct function calls at compile time:

```rust
// This macro call:
let id = nulid!();

// Expands to:
let id = ::nulid::Nulid::new().expect("Failed to generate NULID");
```

## Examples

See the [examples directory](https://github.com/kakilangit/nulid/tree/main/examples) in the nulid repository for more usage examples.

## License

This project is licensed under the MIT License - see the LICENSE file for details.

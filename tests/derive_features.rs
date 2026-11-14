//! Compile-level regression test: `#[derive(Id)]` must work under **each**
//! sqlx storage mapping on its own.
//!
//! `nulid_derive`'s features are plain flags that `nulid` turns on per feature,
//! and 0.12.0 shipped `sqlx-postgres = ["uuid"]` — so a `sqlx-postgres-bytea`
//! build dragged in the derive's UUID support and generated code referring to
//! `nulid::__private::uuid`, which the bytea mapping does not enable. Every
//! downstream crate deriving an id for a `bytea(16)` schema failed to compile.
//!
//! Keeping the wrapper definition here (rather than only in the consumer crate)
//! means a plain `cargo test --features sqlx-postgres-bytea,derive` compiles the
//! combination, which is what CI runs for each mapping.

#![allow(dead_code)]

use nulid::Nulid;

/// A wrapper as a schema with `bytea(16)` id columns writes it.
#[derive(nulid::Id)]
pub struct ByteaId(Nulid);

#[test]
fn a_derived_id_compiles_and_round_trips_under_the_active_mapping() {
    let id = ByteaId::from(Nulid::from_nanos(1_700_000_000_000_000_000, 7));
    assert_eq!(ByteaId::from(Nulid::from(id)), id);
    // The derive's conversions that a repository actually uses.
    let bytes: [u8; 16] = id.into();
    assert_eq!(ByteaId::from(bytes), id);
    assert_eq!(format!("{id}"), format!("{}", Nulid::from(id)));
}

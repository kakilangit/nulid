//! Feature-specific implementations for Id-derived types.
//!
//! This module contains code generation for optional feature support,
//! mirroring the feature organization in the main `nulid` crate.
//!
//! Each module checks this crate's features when the derive expands, and
//! generates nothing for a disabled one. `nulid` forwards each of its features
//! here (`serde = [..., "nulid_derive?/serde"]`), so the impls follow the
//! features enabled on `nulid`. Emitting `#[cfg(feature = "...")]` instead
//! would test the consuming crate's own features, which never match.
//!
//! Generated code names every optional crate through `::nulid::__private`, so
//! a consumer needs no direct dependency on `serde`, `sqlx`, `uuid` or the
//! others.

pub mod chrono;
pub mod jiff;
pub mod postgres_types;
pub mod serde;
pub mod sqlx;
pub mod uuid;

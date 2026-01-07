//! Feature-specific implementations for Id-derived types.
//!
//! This module contains code generation for optional feature support,
//! mirroring the feature organization in the main `nulid` crate.
//!
//! Each module generates code with `#[cfg(feature = "...")]` attributes
//! so features are evaluated in the consuming crate, not in the proc macro crate.

pub mod chrono;
pub mod postgres_types;
pub mod proto;
pub mod serde;
pub mod sqlx;
pub mod uuid;

//! Zero-copy serialization support for NULID via rkyv crate.
//!
//! This module provides rkyv Archive, Serialize, and Deserialize implementations
//! for NULID, enabling efficient zero-copy serialization and deserialization.
//!
//! The rkyv derive macros are applied directly to the `Nulid` struct in `nulid.rs`
//! using `#[cfg_attr(feature = "rkyv", derive(...))]` attributes.
//!
//! # Example
//!
//! ```ignore
//! use nulid::Nulid;
//! use rkyv::ser::{serializers::AllocSerializer, Serializer};
//! use rkyv::{Deserialize, Archive};
//!
//! // Create a NULID
//! let nulid = Nulid::new()?;
//!
//! // Serialize
//! let mut serializer = AllocSerializer::<256>::default();
//! serializer.serialize_value(&nulid)?;
//! let bytes = serializer.into_serializer().into_inner();
//!
//! // Zero-copy access
//! let archived = unsafe { rkyv::archived_root::<Nulid>(&bytes) };
//!
//! // Deserialize
//! let mut deserializer = rkyv::de::deserializers::SharedDeserializeMap::new();
//! let deserialized: Nulid = archived.deserialize(&mut deserializer)?;
//! ```
//!
//! See `examples/rkyv_example.rs` for a complete working example.

//! Derive macros for types that wrap `Nulid`.
//!
//! This crate provides derive macros to automatically implement common traits
//! for newtype wrappers around `Nulid`.
//!
//! # Examples
//!
//! ```ignore
//! use nulid::Nulid;
//! use nulid_derive::Id;
//!
//! #[derive(Id)]
//! pub struct UserId(Nulid);
//!
//! // Now you can use TryFrom:
//! let id = UserId::try_from("01HZQWER4TYUIOP9876QWERTY5")?;
//! let id2 = UserId::try_from("01HZQWER4TYUIOP9876QWERTY5".to_string())?;
//! ```

use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, parse_macro_input};

mod features;

/// Derives common traits for types that wrap `Nulid`.
///
/// This macro implements the following traits for a newtype wrapper:
/// - `TryFrom<String>`
/// - `TryFrom<&str>`
/// - `From<Nulid>`
/// - `From<T> for Nulid` (where T is your wrapper type)
/// - `AsRef<Nulid>`
/// - `Deref<Target = Nulid>` - Enables direct access to all Nulid methods
/// - `DerefMut` - Enables mutable access to the inner Nulid
/// - `std::fmt::Display`
/// - `std::fmt::Debug`
/// - `std::str::FromStr`
/// - `Copy` (which automatically provides `Clone`)
/// - `PartialEq` and `PartialEq<Nulid>` - Equality comparison with wrapper and inner type
/// - `Eq`
/// - `PartialOrd` and `PartialOrd<Nulid>` - Ordering comparison with wrapper and inner type
/// - `Ord`
/// - `Hash`
/// - `Default` - Creates a new instance with a default Nulid (ZERO)
///
/// # Feature-gated Traits
///
/// When the corresponding feature is enabled, the following traits are also implemented:
///
/// ## `serde` feature
/// - `Serialize` - Serialization support
/// - `Deserialize` - Deserialization support
///
/// ## `uuid` feature
/// - `From<uuid::Uuid>` - Convert from UUID
/// - `Into<uuid::Uuid>` - Convert to UUID
/// - `to_uuid()` method
/// - `from_uuid()` method
///
/// ## `sqlx` feature
/// - `Type<Postgres>` - PostgreSQL type support
/// - `Encode<Postgres>` - Encoding for PostgreSQL
/// - `Decode<Postgres>` - Decoding from PostgreSQL
/// - `PgHasArrayType` - Array type support
///
/// ## `postgres-types` feature
/// - `FromSql` - Deserialize from PostgreSQL
/// - `ToSql` - Serialize to PostgreSQL
///
/// ## `proto` feature
/// - `From<NulidProto>` - Convert from protobuf
/// - `Into<NulidProto>` - Convert to protobuf
/// - `to_proto()` method
/// - `from_proto()` method
///
/// # Constructor Methods
///
/// It also provides constructor methods that mirror Nulid's API:
/// - `new()` - Creates a new instance with a freshly generated Nulid
/// - `now()` - Alias for `new()`
/// - `nil()` - Creates a nil/zero instance
/// - `min()` - Returns the minimum possible instance (all zeros)
/// - `max()` - Returns the maximum possible instance (all ones)
/// - `from_datetime(SystemTime)` - Creates from specific time
/// - `from_nanos(u128, u64)` - Creates from timestamp and random
/// - `from_u128(u128)` - Creates from raw u128
/// - `from_bytes([u8; 16])` - Creates from byte array
///
/// With `Deref`, you can call any `Nulid` method directly on the wrapper type:
/// ```ignore
/// let user_id = UserId::new()?;
/// let nanos = user_id.nanos();  // Direct access to Nulid::nanos()
/// let random = user_id.random(); // Direct access to Nulid::random()
/// ```
///
/// # Requirements
///
/// The type must be a tuple struct with exactly one field of type `Nulid`.
///
/// # Examples
///
/// ```ignore
/// use nulid::Nulid;
/// use nulid_derive::Id;
///
/// #[derive(Id)]
/// pub struct UserId(Nulid);
///
/// #[derive(Id)]
/// pub struct OrderId(pub Nulid);
///
/// // Usage:
/// let user_id = UserId::try_from("01HZQWER4TYUIOP9876QWERTY5")?;
/// let nulid = Nulid::from(user_id);
/// let user_id2 = UserId::from(nulid);
/// println!("{}", user_id); // Prints the Base32-encoded string
///
/// // Comparison works automatically
/// assert_eq!(user_id, user_id2);
/// assert!(user_id <= user_id2);
///
/// // Direct comparison with Nulid
/// assert_eq!(user_id, nulid);
/// assert!(user_id <= nulid);
///
/// // Create new instance with fresh ID
/// let new_user_id = UserId::new()?;
///
/// // Create default instance (ZERO)
/// let default_user_id = UserId::default();
///
/// // Use min/max for range operations
/// let min_id = UserId::min();
/// let max_id = UserId::max();
///
/// // Access Nulid methods directly via Deref
/// let nanos = user_id.nanos();
/// let random = user_id.random();
/// let (timestamp, rand) = user_id.parts();
/// ```
#[allow(clippy::too_many_lines)]
#[proc_macro_derive(Id)]
pub fn derive_id(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    // Validate that this is a tuple struct with one field
    let Data::Struct(data_struct) = &input.data else {
        return syn::Error::new_spanned(&input, "Id can only be derived for structs")
            .to_compile_error()
            .into();
    };

    let Fields::Unnamed(fields) = &data_struct.fields else {
        return syn::Error::new_spanned(
            &data_struct.fields,
            "Id requires a tuple struct (e.g., struct UserId(Nulid))",
        )
        .to_compile_error()
        .into();
    };

    if fields.unnamed.len() != 1 {
        return syn::Error::new_spanned(&fields.unnamed, "Id requires exactly one field")
            .to_compile_error()
            .into();
    }

    // Generate core trait implementations
    let core_impls = quote! {
        impl #impl_generics ::std::convert::TryFrom<::std::string::String> for #name #ty_generics #where_clause {
            type Error = ::nulid::Error;

            fn try_from(s: ::std::string::String) -> ::std::result::Result<Self, Self::Error> {
                use ::std::str::FromStr;
                ::nulid::Nulid::from_str(&s).map(#name)
            }
        }

        impl #impl_generics ::std::convert::TryFrom<&str> for #name #ty_generics #where_clause {
            type Error = ::nulid::Error;

            fn try_from(s: &str) -> ::std::result::Result<Self, Self::Error> {
                use ::std::str::FromStr;
                ::nulid::Nulid::from_str(s).map(#name)
            }
        }

        impl #impl_generics ::std::convert::From<::nulid::Nulid> for #name #ty_generics #where_clause {
            fn from(nulid: ::nulid::Nulid) -> Self {
                #name(nulid)
            }
        }

        impl #impl_generics ::std::convert::From<#name #ty_generics> for ::nulid::Nulid #where_clause {
            fn from(wrapper: #name #ty_generics) -> Self {
                wrapper.0
            }
        }

        impl #impl_generics ::std::convert::AsRef<::nulid::Nulid> for #name #ty_generics #where_clause {
            fn as_ref(&self) -> &::nulid::Nulid {
                &self.0
            }
        }

        impl #impl_generics ::std::convert::From<u128> for #name #ty_generics #where_clause {
            fn from(value: u128) -> Self {
                #name(::nulid::Nulid::from_u128(value))
            }
        }

        impl #impl_generics ::std::convert::From<#name #ty_generics> for u128 #where_clause {
            fn from(wrapper: #name #ty_generics) -> Self {
                wrapper.0.as_u128()
            }
        }

        impl #impl_generics ::std::convert::From<[u8; 16]> for #name #ty_generics #where_clause {
            fn from(bytes: [u8; 16]) -> Self {
                #name(::nulid::Nulid::from_bytes(bytes))
            }
        }

        impl #impl_generics ::std::convert::From<#name #ty_generics> for [u8; 16] #where_clause {
            fn from(wrapper: #name #ty_generics) -> Self {
                wrapper.0.to_bytes()
            }
        }

        impl #impl_generics ::std::convert::AsRef<u128> for #name #ty_generics #where_clause {
            fn as_ref(&self) -> &u128 {
                self.0.as_ref()
            }
        }

        impl #impl_generics ::std::convert::TryFrom<&[u8]> for #name #ty_generics #where_clause {
            type Error = ::nulid::Error;

            fn try_from(bytes: &[u8]) -> ::std::result::Result<Self, Self::Error> {
                ::nulid::Nulid::try_from(bytes).map(#name)
            }
        }

        impl #impl_generics ::std::ops::Deref for #name #ty_generics #where_clause {
            type Target = ::nulid::Nulid;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl #impl_generics ::std::ops::DerefMut for #name #ty_generics #where_clause {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.0
            }
        }

        impl #impl_generics ::std::fmt::Display for #name #ty_generics #where_clause {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                ::std::fmt::Display::fmt(&self.0, f)
            }
        }

        impl #impl_generics ::std::str::FromStr for #name #ty_generics #where_clause {
            type Err = ::nulid::Error;

            fn from_str(s: &str) -> ::std::result::Result<Self, Self::Err> {
                ::nulid::Nulid::from_str(s).map(#name)
            }
        }

        impl #impl_generics ::std::fmt::Debug for #name #ty_generics #where_clause {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                f.debug_tuple(::std::stringify!(#name))
                    .field(&self.0)
                    .finish()
            }
        }

        #[allow(clippy::expl_impl_clone_on_copy)]
        impl #impl_generics ::std::clone::Clone for #name #ty_generics #where_clause {
            fn clone(&self) -> Self {
                *self
            }
        }

        impl #impl_generics ::std::marker::Copy for #name #ty_generics #where_clause {}

        impl #impl_generics ::std::cmp::PartialEq for #name #ty_generics #where_clause {
            fn eq(&self, other: &Self) -> bool {
                self.0 == other.0
            }
        }

        impl #impl_generics ::std::cmp::Eq for #name #ty_generics #where_clause {}

        impl #impl_generics ::std::cmp::PartialOrd for #name #ty_generics #where_clause {
            fn partial_cmp(&self, other: &Self) -> ::std::option::Option<::std::cmp::Ordering> {
                ::std::option::Option::Some(self.cmp(other))
            }
        }

        impl #impl_generics ::std::cmp::Ord for #name #ty_generics #where_clause {
            fn cmp(&self, other: &Self) -> ::std::cmp::Ordering {
                self.0.cmp(&other.0)
            }
        }

        impl #impl_generics ::std::hash::Hash for #name #ty_generics #where_clause {
            fn hash<H: ::std::hash::Hasher>(&self, state: &mut H) {
                self.0.hash(state);
            }
        }

        impl #impl_generics ::std::cmp::PartialEq<::nulid::Nulid> for #name #ty_generics #where_clause {
            fn eq(&self, other: &::nulid::Nulid) -> bool {
                self.0 == *other
            }
        }

        impl #impl_generics ::std::cmp::PartialOrd<::nulid::Nulid> for #name #ty_generics #where_clause {
            fn partial_cmp(&self, other: &::nulid::Nulid) -> ::std::option::Option<::std::cmp::Ordering> {
                self.0.partial_cmp(other)
            }
        }

        impl #impl_generics ::std::default::Default for #name #ty_generics #where_clause {
            fn default() -> Self {
                #name(::nulid::Nulid::default())
            }
        }

        impl #impl_generics #name #ty_generics #where_clause {
            /// Creates a new instance with a freshly generated Nulid.
            ///
            /// # Errors
            ///
            /// Returns an error if the Nulid generation fails.
            pub fn new() -> ::std::result::Result<Self, ::nulid::Error> {
                ::nulid::Nulid::new().map(#name)
            }

            /// Generates a new instance with the current timestamp and random bits.
            ///
            /// This is an alias for [`new()`](Self::new).
            ///
            /// # Errors
            ///
            /// Returns an error if:
            /// - The system time is before Unix epoch
            /// - Random number generation fails
            pub fn now() -> ::std::result::Result<Self, ::nulid::Error> {
                ::nulid::Nulid::now().map(#name)
            }

            /// Creates an instance from a `SystemTime` with random bits.
            ///
            /// # Examples
            ///
            /// ```ignore
            /// use std::time::SystemTime;
            /// let time = SystemTime::now();
            /// let id = UserId::from_datetime(time)?;
            /// ```
            ///
            /// # Errors
            ///
            /// Returns an error if:
            /// - The time is before Unix epoch
            /// - Random number generation fails
            pub fn from_datetime(time: ::std::time::SystemTime) -> ::std::result::Result<Self, ::nulid::Error> {
                ::nulid::Nulid::from_datetime(time).map(#name)
            }

            /// Creates a nil (zero) instance.
            ///
            /// # Examples
            ///
            /// ```ignore
            /// let nil_id = UserId::nil();
            /// assert!(nil_id.is_nil());
            /// ```
            #[must_use]
            pub const fn nil() -> Self {
                #name(::nulid::Nulid::nil())
            }

            /// Returns the minimum possible instance (all zeros).
            ///
            /// # Examples
            ///
            /// ```ignore
            /// let min_id = UserId::min();
            /// assert!(min_id.is_nil());
            /// ```
            #[must_use]
            pub const fn min() -> Self {
                #name(::nulid::Nulid::min())
            }

            /// Returns the maximum possible instance (all ones).
            ///
            /// # Examples
            ///
            /// ```ignore
            /// let max_id = UserId::max();
            /// assert_eq!(max_id.as_u128(), u128::MAX);
            /// ```
            #[must_use]
            pub const fn max() -> Self {
                #name(::nulid::Nulid::max())
            }

            /// Creates an instance from a 16-byte array (big-endian).
            ///
            /// # Examples
            ///
            /// ```ignore
            /// let bytes = [0u8; 16];
            /// let id = UserId::from_bytes(bytes);
            /// ```
            #[must_use]
            pub const fn from_bytes(bytes: [u8; 16]) -> Self {
                #name(::nulid::Nulid::from_bytes(bytes))
            }

            /// Creates an instance from a raw `u128` value.
            ///
            /// # Examples
            ///
            /// ```ignore
            /// let id = UserId::from_u128(0x0123_4567_89AB_CDEF_FEDC_BA98_7654_3210);
            /// ```
            #[must_use]
            pub const fn from_u128(value: u128) -> Self {
                #name(::nulid::Nulid::from_u128(value))
            }

            /// Creates an instance from a timestamp (nanoseconds) and random value.
            ///
            /// The timestamp is masked to 68 bits and the random value is masked to 60 bits.
            ///
            /// # Examples
            ///
            /// ```ignore
            /// let id = UserId::from_nanos(1_000_000_000_000, 12345);
            /// ```
            #[must_use]
            pub const fn from_nanos(timestamp_nanos: u128, random: u64) -> Self {
                #name(::nulid::Nulid::from_nanos(timestamp_nanos, random))
            }
        }
    };

    // Generate feature-gated implementations
    // Always generate the code with #[cfg] attributes so they're evaluated in the consuming crate
    let serde_impls =
        features::serde::generate_serde_impls(name, &impl_generics, &ty_generics, &where_clause);
    let uuid_impls =
        features::uuid::generate_uuid_impls(name, &impl_generics, &ty_generics, &where_clause);
    let sqlx_impls =
        features::sqlx::generate_sqlx_impls(name, &impl_generics, &ty_generics, &where_clause);
    let postgres_impls = features::postgres_types::generate_postgres_types_impls(
        name,
        &impl_generics,
        &ty_generics,
        &where_clause,
    );
    let chrono_impls =
        features::chrono::generate_chrono_impls(name, &impl_generics, &ty_generics, &where_clause);
    let proto_impls =
        features::proto::generate_proto_impls(name, &impl_generics, &ty_generics, &where_clause);

    // Combine all implementations
    let expanded = quote! {
        #core_impls
        #serde_impls
        #uuid_impls
        #sqlx_impls
        #postgres_impls
        #chrono_impls
        #proto_impls
    };

    TokenStream::from(expanded)
}

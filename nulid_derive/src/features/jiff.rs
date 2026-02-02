//! Jiff integration for Id-derived types.
//!
//! This module provides code generation for jiff Timestamp conversion implementations
//! for types that derive `Id`, delegating to the inner `Nulid`'s jiff support.

use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

/// Generates jiff trait implementations for the Id wrapper type.
///
/// This generates `TryFrom<jiff::Timestamp>` and `TryInto<jiff::Timestamp>` implementations
/// that delegate to the inner `Nulid` type's implementations.
pub fn generate_jiff_impls(
    name: &Ident,
    impl_generics: &syn::ImplGenerics,
    ty_generics: &syn::TypeGenerics,
    where_clause: &Option<&syn::WhereClause>,
) -> TokenStream {
    quote! {
        #[cfg(feature = "jiff")]
        impl #impl_generics ::core::convert::TryFrom<::jiff::Timestamp> for #name #ty_generics #where_clause {
            type Error = ::nulid::Error;

            fn try_from(ts: ::jiff::Timestamp) -> ::core::result::Result<Self, Self::Error> {
                ::nulid::Nulid::from_jiff_timestamp(ts).map(#name)
            }
        }

        #[cfg(feature = "jiff")]
        impl #impl_generics ::core::convert::TryFrom<#name #ty_generics> for ::jiff::Timestamp #where_clause {
            type Error = ::nulid::Error;

            fn try_from(wrapper: #name #ty_generics) -> ::core::result::Result<Self, Self::Error> {
                wrapper.0.jiff_timestamp()
            }
        }

        #[cfg(feature = "jiff")]
        impl #impl_generics #name #ty_generics #where_clause {
            /// Converts this ID to a `jiff::Timestamp`.
            ///
            /// # Examples
            ///
            /// ```ignore
            /// use jiff::Timestamp;
            ///
            /// let user_id = UserId::new()?;
            /// let ts: Timestamp = user_id.jiff_timestamp();
            /// println!("User ID timestamp: {}", ts);
            /// ```
            #[must_use]
            pub fn jiff_timestamp(self) -> ::core::result::Result<::jiff::Timestamp, ::nulid::Error> {
                self.0.jiff_timestamp()
            }

            /// Creates an ID from a `jiff::Timestamp` with random bits.
            ///
            /// # Examples
            ///
            /// ```ignore
            /// use jiff::Timestamp;
            ///
            /// let ts = Timestamp::from_second(1_704_067_200).unwrap();
            /// let user_id = UserId::from_jiff_timestamp(ts)?;
            /// println!("User ID from Timestamp: {}", user_id);
            /// ```
            ///
            /// # Errors
            ///
            /// Returns an error if random number generation fails.
            pub fn from_jiff_timestamp(ts: ::jiff::Timestamp) -> ::core::result::Result<Self, ::nulid::Error> {
                ::nulid::Nulid::from_jiff_timestamp(ts).map(#name)
            }
        }
    }
}

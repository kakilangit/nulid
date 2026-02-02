//! Chrono integration for Id-derived types.
//!
//! This module provides code generation for chrono DateTime conversion implementations
//! for types that derive `Id`, delegating to the inner `Nulid`'s chrono support.

use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

/// Generates chrono trait implementations for the Id wrapper type.
///
/// This generates `TryFrom<chrono::DateTime<Utc>>` and `TryInto<chrono::DateTime<Utc>>` implementations
/// that delegate to the inner `Nulid` type's implementations.
pub fn generate_chrono_impls(
    name: &Ident,
    impl_generics: &syn::ImplGenerics,
    ty_generics: &syn::TypeGenerics,
    where_clause: &Option<&syn::WhereClause>,
) -> TokenStream {
    quote! {
        #[cfg(feature = "chrono")]
        impl #impl_generics ::core::convert::TryFrom<::chrono::DateTime<::chrono::Utc>> for #name #ty_generics #where_clause {
            type Error = ::nulid::Error;

            fn try_from(dt: ::chrono::DateTime<::chrono::Utc>) -> ::core::result::Result<Self, Self::Error> {
                ::nulid::Nulid::from_chrono_datetime(dt).map(#name)
            }
        }

        #[cfg(feature = "chrono")]
        impl #impl_generics ::core::convert::TryFrom<#name #ty_generics> for ::chrono::DateTime<::chrono::Utc> #where_clause {
            type Error = ::nulid::Error;

            fn try_from(wrapper: #name #ty_generics) -> ::core::result::Result<Self, Self::Error> {
                wrapper.0.chrono_datetime()
            }
        }

        #[cfg(feature = "chrono")]
        impl #impl_generics #name #ty_generics #where_clause {
            /// Converts this ID to a `chrono::DateTime<Utc>`.
            ///
            /// # Examples
            ///
            /// ```ignore
            /// use chrono::{DateTime, Utc};
            ///
            /// let user_id = UserId::new()?;
            /// let dt: DateTime<Utc> = user_id.chrono_datetime();
            /// println!("User ID timestamp: {}", dt);
            /// ```
            #[must_use]
            pub fn chrono_datetime(self) -> ::core::result::Result<::chrono::DateTime<::chrono::Utc>, ::nulid::Error> {
                self.0.chrono_datetime()
            }

            /// Creates an ID from a `chrono::DateTime<Utc>` with random bits.
            ///
            /// # Examples
            ///
            /// ```ignore
            /// use chrono::{DateTime, Utc, TimeZone};
            ///
            /// let dt = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
            /// let user_id = UserId::from_chrono_datetime(dt)?;
            /// println!("User ID from DateTime: {}", user_id);
            /// ```
            ///
            /// # Errors
            ///
            /// Returns an error if random number generation fails.
            pub fn from_chrono_datetime(dt: ::chrono::DateTime<::chrono::Utc>) -> ::core::result::Result<Self, ::nulid::Error> {
                ::nulid::Nulid::from_chrono_datetime(dt).map(#name)
            }
        }
    }
}

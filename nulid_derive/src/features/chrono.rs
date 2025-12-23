//! Chrono integration for Id-derived types.
//!
//! This module provides code generation for chrono DateTime conversion implementations
//! for types that derive `Id`, delegating to the inner `Nulid`'s chrono support.

use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

/// Generates chrono trait implementations for the Id wrapper type.
///
/// This generates a `to_datetime()` method that delegates to the inner `Nulid` type's implementation.
pub fn generate_chrono_impls(
    name: &Ident,
    impl_generics: &syn::ImplGenerics,
    ty_generics: &syn::TypeGenerics,
    where_clause: &Option<&syn::WhereClause>,
) -> TokenStream {
    quote! {
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
            pub fn chrono_datetime(self) -> ::chrono::DateTime<::chrono::Utc> {
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
            pub fn from_chrono_datetime(dt: ::chrono::DateTime<::chrono::Utc>) -> ::std::result::Result<Self, ::nulid::Error> {
                ::nulid::Nulid::from_chrono_datetime(dt).map(#name)
            }
        }
    }
}

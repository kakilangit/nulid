//! UUID interoperability for Id-derived types.
//!
//! This module provides code generation for UUID conversion implementations
//! for types that derive `Id`, delegating to the inner `Nulid`'s UUID support.

use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

/// Generates UUID trait implementations for the Id wrapper type.
///
/// This generates `From<uuid::Uuid>` and `Into<uuid::Uuid>` implementations
/// that delegate to the inner `Nulid` type's implementations.
pub fn generate_uuid_impls(
    name: &Ident,
    impl_generics: &syn::ImplGenerics,
    ty_generics: &syn::TypeGenerics,
    where_clause: &Option<&syn::WhereClause>,
) -> TokenStream {
    quote! {
        #[cfg(feature = "uuid")]
        impl #impl_generics ::std::convert::From<::uuid::Uuid> for #name #ty_generics #where_clause {
            fn from(uuid: ::uuid::Uuid) -> Self {
                #name(::nulid::Nulid::from_uuid(uuid))
            }
        }

        #[cfg(feature = "uuid")]
        impl #impl_generics ::std::convert::From<#name #ty_generics> for ::uuid::Uuid #where_clause {
            fn from(wrapper: #name #ty_generics) -> Self {
                wrapper.0.to_uuid()
            }
        }

        #[cfg(feature = "uuid")]
        impl #impl_generics #name #ty_generics #where_clause {
            /// Converts this ID to a UUID.
            ///
            /// The 128-bit value is preserved exactly, maintaining full compatibility
            /// with UUID-based systems.
            #[must_use]
            pub const fn to_uuid(self) -> ::uuid::Uuid {
                self.0.to_uuid()
            }

            /// Creates an ID from a UUID.
            ///
            /// The 128-bit value is preserved exactly.
            #[must_use]
            pub const fn from_uuid(uuid: ::uuid::Uuid) -> Self {
                #name(::nulid::Nulid::from_uuid(uuid))
            }
        }
    }
}

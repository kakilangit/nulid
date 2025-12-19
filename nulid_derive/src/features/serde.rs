//! Serde serialization support for Id-derived types.
//!
//! This module provides code generation for `Serialize` and `Deserialize` implementations
//! for types that derive `Id`, delegating to the inner `Nulid`'s serde implementation.

use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

/// Generates serde trait implementations for the Id wrapper type.
///
/// This generates `Serialize` and `Deserialize` implementations that delegate
/// to the inner `Nulid` type's implementations.
pub fn generate_serde_impls(
    name: &Ident,
    impl_generics: &syn::ImplGenerics,
    ty_generics: &syn::TypeGenerics,
    where_clause: &Option<&syn::WhereClause>,
) -> TokenStream {
    quote! {
        #[cfg(feature = "serde")]
        impl #impl_generics ::serde::Serialize for #name #ty_generics #where_clause {
            fn serialize<S>(&self, serializer: S) -> ::std::result::Result<S::Ok, S::Error>
            where
                S: ::serde::Serializer,
            {
                self.0.serialize(serializer)
            }
        }

        #[cfg(feature = "serde")]
        impl<'de> ::serde::Deserialize<'de> for #name #where_clause {
            fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
            where
                D: ::serde::Deserializer<'de>,
            {
                ::nulid::Nulid::deserialize(deserializer).map(#name)
            }
        }
    }
}

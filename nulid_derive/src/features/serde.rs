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
    let mut tokens = TokenStream::new();
    if cfg!(feature = "serde") {
        tokens.extend(quote! {
            impl #impl_generics ::nulid::__private::serde::Serialize for #name #ty_generics #where_clause {
                fn serialize<S>(&self, serializer: S) -> ::core::result::Result<S::Ok, S::Error>
                where
                    S: ::nulid::__private::serde::Serializer,
                {
                    ::nulid::__private::serde::Serialize::serialize(&self.0, serializer)
                }
            }

            impl<'de> ::nulid::__private::serde::Deserialize<'de> for #name #where_clause {
                fn deserialize<D>(deserializer: D) -> ::core::result::Result<Self, D::Error>
                where
                    D: ::nulid::__private::serde::Deserializer<'de>,
                {
                    <::nulid::Nulid as ::nulid::__private::serde::Deserialize<'de>>::deserialize(deserializer).map(#name)
                }
            }
        });
    }
    tokens
}

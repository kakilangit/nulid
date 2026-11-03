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
    let mut tokens = TokenStream::new();
    if cfg!(feature = "uuid") {
        tokens.extend(quote! {
            impl #impl_generics ::core::convert::From<::nulid::__private::uuid::Uuid> for #name #ty_generics #where_clause {
                fn from(uuid: ::nulid::__private::uuid::Uuid) -> Self {
                    #name(::nulid::Nulid::from_uuid(uuid))
                }
            }

            impl #impl_generics ::core::convert::From<#name #ty_generics> for ::nulid::__private::uuid::Uuid #where_clause {
                fn from(wrapper: #name #ty_generics) -> Self {
                    wrapper.0.to_uuid()
                }
            }

            impl #impl_generics #name #ty_generics #where_clause {
                /// Converts this ID to a UUID.
                ///
                /// The 128-bit value is preserved exactly, maintaining full compatibility
                /// with UUID-based systems.
                #[must_use]
                pub const fn to_uuid(self) -> ::nulid::__private::uuid::Uuid {
                    self.0.to_uuid()
                }

                /// Creates an ID from a UUID.
                ///
                /// The 128-bit value is preserved exactly.
                #[must_use]
                pub const fn from_uuid(uuid: ::nulid::__private::uuid::Uuid) -> Self {
                    #name(::nulid::Nulid::from_uuid(uuid))
                }
            }
        });
    }
    tokens
}

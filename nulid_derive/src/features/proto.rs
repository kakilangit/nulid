//! Protobuf integration for Id-derived types.
//!
//! This module provides code generation for protobuf conversion implementations
//! for types that derive `Id`, delegating to the inner `Nulid`'s protobuf support.

use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

/// Generates protobuf trait implementations for the Id wrapper type.
///
/// This generates `From<ProtoNulid>` and `Into<ProtoNulid>` implementations
/// that delegate to the inner `Nulid` type's implementations.
pub fn generate_proto_impls(
    name: &Ident,
    impl_generics: &syn::ImplGenerics,
    ty_generics: &syn::TypeGenerics,
    where_clause: &Option<&syn::WhereClause>,
) -> TokenStream {
    quote! {
        #[cfg(feature = "proto")]
        impl #impl_generics ::std::convert::From<::nulid::proto::nulid::Nulid> for #name #ty_generics #where_clause {
            fn from(proto: ::nulid::proto::nulid::Nulid) -> Self {
                #name(::nulid::Nulid::from_proto(proto))
            }
        }

        #[cfg(feature = "proto")]
        impl #impl_generics ::std::convert::From<#name #ty_generics> for ::nulid::proto::nulid::Nulid #where_clause {
            fn from(wrapper: #name #ty_generics) -> Self {
                wrapper.0.to_proto()
            }
        }

        #[cfg(feature = "proto")]
        impl #impl_generics #name #ty_generics #where_clause {
            /// Converts this ID to its protobuf representation.
            ///
            /// The 128-bit value is split into high and low 64-bit parts.
            ///
            /// # Examples
            ///
            /// ```ignore
            /// let user_id = UserId::new()?;
            /// let proto = user_id.to_proto();
            /// ```
            #[must_use]
            pub const fn to_proto(self) -> ::nulid::proto::nulid::Nulid {
                self.0.to_proto()
            }

            /// Creates an ID from its protobuf representation.
            ///
            /// # Examples
            ///
            /// ```ignore
            /// let proto = ::nulid::proto::nulid::Nulid {
            ///     high: 0x0123456789ABCDEF,
            ///     low: 0xFEDCBA9876543210,
            /// };
            /// let user_id = UserId::from_proto(proto);
            /// ```
            #[must_use]
            pub const fn from_proto(proto: ::nulid::proto::nulid::Nulid) -> Self {
                #name(::nulid::Nulid::from_proto(proto))
            }
        }
    }
}

//! SQLx support for Id-derived types.
//!
//! This module provides code generation for SQLx trait implementations
//! for types that derive `Id`, delegating to the inner `Nulid`'s SQLx support.

use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

/// Generates SQLx trait implementations for the Id wrapper type.
///
/// This generates `Type<Postgres>`, `Encode`, `Decode`, and `PgHasArrayType`
/// implementations that delegate to the inner `Nulid` type's implementations.
pub fn generate_sqlx_impls(
    name: &Ident,
    impl_generics: &syn::ImplGenerics,
    ty_generics: &syn::TypeGenerics,
    where_clause: &Option<&syn::WhereClause>,
) -> TokenStream {
    quote! {
        #[cfg(feature = "sqlx")]
        impl #impl_generics ::sqlx::Type<::sqlx::Postgres> for #name #ty_generics #where_clause {
            fn type_info() -> ::sqlx::postgres::PgTypeInfo {
                <::nulid::Nulid as ::sqlx::Type<::sqlx::Postgres>>::type_info()
            }

            fn compatible(ty: &::sqlx::postgres::PgTypeInfo) -> bool {
                <::nulid::Nulid as ::sqlx::Type<::sqlx::Postgres>>::compatible(ty)
            }
        }

        #[cfg(feature = "sqlx")]
        impl #impl_generics ::sqlx::postgres::PgHasArrayType for #name #ty_generics #where_clause {
            fn array_type_info() -> ::sqlx::postgres::PgTypeInfo {
                <::nulid::Nulid as ::sqlx::postgres::PgHasArrayType>::array_type_info()
            }

            fn array_compatible(ty: &::sqlx::postgres::PgTypeInfo) -> bool {
                <::nulid::Nulid as ::sqlx::postgres::PgHasArrayType>::array_compatible(ty)
            }
        }

        #[cfg(feature = "sqlx")]
        impl #impl_generics ::sqlx::Encode<'_, ::sqlx::Postgres> for #name #ty_generics #where_clause {
            fn encode_by_ref(
                &self,
                buf: &mut ::sqlx::postgres::PgArgumentBuffer,
            ) -> ::core::result::Result<::sqlx::encode::IsNull, ::sqlx::error::BoxDynError> {
                <::nulid::Nulid as ::sqlx::Encode<::sqlx::Postgres>>::encode_by_ref(&self.0, buf)
            }
        }

        #[cfg(feature = "sqlx")]
        impl<'r> ::sqlx::Decode<'r, ::sqlx::Postgres> for #name #where_clause {
            fn decode(
                value: ::sqlx::postgres::PgValueRef<'r>,
            ) -> ::core::result::Result<Self, ::sqlx::error::BoxDynError> {
                <::nulid::Nulid as ::sqlx::Decode<::sqlx::Postgres>>::decode(value).map(#name)
            }
        }
    }
}

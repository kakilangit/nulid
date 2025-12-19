//! PostgreSQL types support for Id-derived types.
//!
//! This module provides code generation for `postgres-types` trait implementations
//! for types that derive `Id`, delegating to the inner `Nulid`'s postgres-types support.

use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

/// Generates postgres-types trait implementations for the Id wrapper type.
///
/// This generates `FromSql` and `ToSql` implementations that delegate
/// to the inner `Nulid` type's implementations.
pub fn generate_postgres_types_impls(
    name: &Ident,
    impl_generics: &syn::ImplGenerics,
    ty_generics: &syn::TypeGenerics,
    where_clause: &Option<&syn::WhereClause>,
) -> TokenStream {
    quote! {
        #[cfg(feature = "postgres-types")]
        impl<'a> ::postgres_types::FromSql<'a> for #name #where_clause {
            fn from_sql(
                ty: &::postgres_types::Type,
                raw: &'a [u8],
            ) -> ::std::result::Result<Self, ::std::boxed::Box<dyn ::std::error::Error + Sync + Send>> {
                <::nulid::Nulid as ::postgres_types::FromSql>::from_sql(ty, raw).map(#name)
            }

            fn accepts(ty: &::postgres_types::Type) -> bool {
                <::nulid::Nulid as ::postgres_types::FromSql>::accepts(ty)
            }
        }

        #[cfg(feature = "postgres-types")]
        impl #impl_generics ::postgres_types::ToSql for #name #ty_generics #where_clause {
            fn to_sql(
                &self,
                ty: &::postgres_types::Type,
                out: &mut ::bytes::BytesMut,
            ) -> ::std::result::Result<::postgres_types::IsNull, ::std::boxed::Box<dyn ::std::error::Error + Sync + Send>> {
                <::nulid::Nulid as ::postgres_types::ToSql>::to_sql(&self.0, ty, out)
            }

            fn accepts(ty: &::postgres_types::Type) -> bool {
                <::nulid::Nulid as ::postgres_types::ToSql>::accepts(ty)
            }

            fn to_sql_checked(
                &self,
                ty: &::postgres_types::Type,
                out: &mut ::bytes::BytesMut,
            ) -> ::std::result::Result<::postgres_types::IsNull, ::std::boxed::Box<dyn ::std::error::Error + Sync + Send>> {
                <::nulid::Nulid as ::postgres_types::ToSql>::to_sql_checked(&self.0, ty, out)
            }
        }
    }
}

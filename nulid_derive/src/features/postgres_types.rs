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
    let mut tokens = TokenStream::new();
    if cfg!(feature = "postgres-types") {
        tokens.extend(quote! {
            impl<'a> ::nulid::__private::postgres_types::FromSql<'a> for #name #where_clause {
                fn from_sql(
                    ty: &::nulid::__private::postgres_types::Type,
                    raw: &'a [u8],
                ) -> ::core::result::Result<Self, ::std::boxed::Box<dyn ::core::error::Error + Sync + Send>> {
                    <::nulid::Nulid as ::nulid::__private::postgres_types::FromSql>::from_sql(ty, raw).map(#name)
                }

                fn accepts(ty: &::nulid::__private::postgres_types::Type) -> bool {
                    <::nulid::Nulid as ::nulid::__private::postgres_types::FromSql>::accepts(ty)
                }
            }

            impl #impl_generics ::nulid::__private::postgres_types::ToSql for #name #ty_generics #where_clause {
                fn to_sql(
                    &self,
                    ty: &::nulid::__private::postgres_types::Type,
                    out: &mut ::nulid::__private::bytes::BytesMut,
                ) -> ::core::result::Result<::nulid::__private::postgres_types::IsNull, ::std::boxed::Box<dyn ::core::error::Error + Sync + Send>> {
                    <::nulid::Nulid as ::nulid::__private::postgres_types::ToSql>::to_sql(&self.0, ty, out)
                }

                fn accepts(ty: &::nulid::__private::postgres_types::Type) -> bool {
                    <::nulid::Nulid as ::nulid::__private::postgres_types::ToSql>::accepts(ty)
                }

                fn to_sql_checked(
                    &self,
                    ty: &::nulid::__private::postgres_types::Type,
                    out: &mut ::nulid::__private::bytes::BytesMut,
                ) -> ::core::result::Result<::nulid::__private::postgres_types::IsNull, ::std::boxed::Box<dyn ::core::error::Error + Sync + Send>> {
                    <::nulid::Nulid as ::nulid::__private::postgres_types::ToSql>::to_sql_checked(&self.0, ty, out)
                }
            }
        });
    }
    tokens
}

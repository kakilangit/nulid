//! SQLx support for Id-derived types.
//!
//! This module provides code generation for SQLx trait implementations
//! for types that derive `Id`, delegating to the inner `Nulid`'s SQLx support.
//! Supports PostgreSQL, SQLite, and MySQL/MariaDB databases.
//!
//! # Feature Flags
//!
//! Use individual features to only include the database drivers you need:
//! - `sqlx-postgres` - PostgreSQL support only
//! - `sqlx-sqlite` - SQLite support only
//! - `sqlx-mysql` - MySQL/MariaDB support only
//! - `sqlx` - All databases (convenience feature)

use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

/// Generates SQLx trait implementations for the Id wrapper type.
///
/// This generates implementations for PostgreSQL, SQLite, and MySQL/MariaDB:
///
/// PostgreSQL (requires `sqlx-postgres` feature):
/// - `Type<Postgres>`, `Encode`, `Decode`, and `PgHasArrayType`
///
/// SQLite (requires `sqlx-sqlite` feature):
/// - `Type<Sqlite>`, `Encode`, and `Decode`
///
/// MySQL/MariaDB (requires `sqlx-mysql` feature):
/// - `Type<MySql>`, `Encode`, and `Decode`
///
/// All implementations delegate to the inner `Nulid` type's implementations.
pub fn generate_sqlx_impls(
    name: &Ident,
    impl_generics: &syn::ImplGenerics,
    ty_generics: &syn::TypeGenerics,
    where_clause: &Option<&syn::WhereClause>,
) -> TokenStream {
    let mut tokens = TokenStream::new();
    if cfg!(feature = "sqlx-postgres") {
        tokens.extend(quote! {
            impl #impl_generics ::nulid::__private::sqlx_core::types::Type<::nulid::__private::sqlx_postgres::Postgres> for #name #ty_generics #where_clause {
                fn type_info() -> ::nulid::__private::sqlx_postgres::PgTypeInfo {
                    <::nulid::Nulid as ::nulid::__private::sqlx_core::types::Type<::nulid::__private::sqlx_postgres::Postgres>>::type_info()
                }

                fn compatible(ty: &::nulid::__private::sqlx_postgres::PgTypeInfo) -> bool {
                    <::nulid::Nulid as ::nulid::__private::sqlx_core::types::Type<::nulid::__private::sqlx_postgres::Postgres>>::compatible(ty)
                }
            }

            impl #impl_generics ::nulid::__private::sqlx_postgres::PgHasArrayType for #name #ty_generics #where_clause {
                fn array_type_info() -> ::nulid::__private::sqlx_postgres::PgTypeInfo {
                    <::nulid::Nulid as ::nulid::__private::sqlx_postgres::PgHasArrayType>::array_type_info()
                }

                fn array_compatible(ty: &::nulid::__private::sqlx_postgres::PgTypeInfo) -> bool {
                    <::nulid::Nulid as ::nulid::__private::sqlx_postgres::PgHasArrayType>::array_compatible(ty)
                }
            }

            impl #impl_generics ::nulid::__private::sqlx_core::encode::Encode<'_, ::nulid::__private::sqlx_postgres::Postgres> for #name #ty_generics #where_clause {
                fn encode_by_ref(
                    &self,
                    buf: &mut ::nulid::__private::sqlx_postgres::PgArgumentBuffer,
                ) -> ::core::result::Result<::nulid::__private::sqlx_core::encode::IsNull, ::nulid::__private::sqlx_core::error::BoxDynError> {
                    <::nulid::Nulid as ::nulid::__private::sqlx_core::encode::Encode<::nulid::__private::sqlx_postgres::Postgres>>::encode_by_ref(&self.0, buf)
                }
            }

            impl<'r> ::nulid::__private::sqlx_core::decode::Decode<'r, ::nulid::__private::sqlx_postgres::Postgres> for #name #where_clause {
                fn decode(
                    value: ::nulid::__private::sqlx_postgres::PgValueRef<'r>,
                ) -> ::core::result::Result<Self, ::nulid::__private::sqlx_core::error::BoxDynError> {
                    <::nulid::Nulid as ::nulid::__private::sqlx_core::decode::Decode<::nulid::__private::sqlx_postgres::Postgres>>::decode(value).map(#name)
                }
            }
        });
    }
    if cfg!(feature = "sqlx-sqlite") {
        tokens.extend(quote! {
            impl #impl_generics ::nulid::__private::sqlx_core::types::Type<::nulid::__private::sqlx_sqlite::Sqlite> for #name #ty_generics #where_clause {
                fn type_info() -> ::nulid::__private::sqlx_sqlite::SqliteTypeInfo {
                    <::nulid::Nulid as ::nulid::__private::sqlx_core::types::Type<::nulid::__private::sqlx_sqlite::Sqlite>>::type_info()
                }
            }

            impl<'q> ::nulid::__private::sqlx_core::encode::Encode<'q, ::nulid::__private::sqlx_sqlite::Sqlite> for #name #ty_generics #where_clause {
                fn encode_by_ref(
                    &self,
                    args: &mut ::nulid::__private::sqlx_sqlite::SqliteArgumentsBuffer,
                ) -> ::core::result::Result<::nulid::__private::sqlx_core::encode::IsNull, ::nulid::__private::sqlx_core::error::BoxDynError> {
                    <::nulid::Nulid as ::nulid::__private::sqlx_core::encode::Encode<::nulid::__private::sqlx_sqlite::Sqlite>>::encode_by_ref(&self.0, args)
                }
            }

            impl<'r> ::nulid::__private::sqlx_core::decode::Decode<'r, ::nulid::__private::sqlx_sqlite::Sqlite> for #name #where_clause {
                fn decode(
                    value: ::nulid::__private::sqlx_sqlite::SqliteValueRef<'r>,
                ) -> ::core::result::Result<Self, ::nulid::__private::sqlx_core::error::BoxDynError> {
                    <::nulid::Nulid as ::nulid::__private::sqlx_core::decode::Decode<::nulid::__private::sqlx_sqlite::Sqlite>>::decode(value).map(#name)
                }
            }
        });
    }
    if cfg!(feature = "sqlx-mysql") {
        tokens.extend(quote! {
            impl #impl_generics ::nulid::__private::sqlx_core::types::Type<::nulid::__private::sqlx_mysql::MySql> for #name #ty_generics #where_clause {
                fn type_info() -> ::nulid::__private::sqlx_mysql::MySqlTypeInfo {
                    <::nulid::Nulid as ::nulid::__private::sqlx_core::types::Type<::nulid::__private::sqlx_mysql::MySql>>::type_info()
                }
            }

            impl #impl_generics ::nulid::__private::sqlx_core::encode::Encode<'_, ::nulid::__private::sqlx_mysql::MySql> for #name #ty_generics #where_clause {
                fn encode_by_ref(
                    &self,
                    buf: &mut ::std::vec::Vec<u8>,
                ) -> ::core::result::Result<::nulid::__private::sqlx_core::encode::IsNull, ::nulid::__private::sqlx_core::error::BoxDynError> {
                    <::nulid::Nulid as ::nulid::__private::sqlx_core::encode::Encode<::nulid::__private::sqlx_mysql::MySql>>::encode_by_ref(&self.0, buf)
                }
            }

            impl<'r> ::nulid::__private::sqlx_core::decode::Decode<'r, ::nulid::__private::sqlx_mysql::MySql> for #name #where_clause {
                fn decode(
                    value: ::nulid::__private::sqlx_mysql::MySqlValueRef<'r>,
                ) -> ::core::result::Result<Self, ::nulid::__private::sqlx_core::error::BoxDynError> {
                    <::nulid::Nulid as ::nulid::__private::sqlx_core::decode::Decode<::nulid::__private::sqlx_mysql::MySql>>::decode(value).map(#name)
                }
            }
        });
    }
    tokens
}

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
    quote! {
        // ====================================================================
        // PostgreSQL implementations
        // ====================================================================

        #[cfg(feature = "sqlx-postgres")]
        impl #impl_generics ::sqlx_core::types::Type<::sqlx_postgres::Postgres> for #name #ty_generics #where_clause {
            fn type_info() -> ::sqlx_postgres::PgTypeInfo {
                <::nulid::Nulid as ::sqlx_core::types::Type<::sqlx_postgres::Postgres>>::type_info()
            }

            fn compatible(ty: &::sqlx_postgres::PgTypeInfo) -> bool {
                <::nulid::Nulid as ::sqlx_core::types::Type<::sqlx_postgres::Postgres>>::compatible(ty)
            }
        }

        #[cfg(feature = "sqlx-postgres")]
        impl #impl_generics ::sqlx_postgres::PgHasArrayType for #name #ty_generics #where_clause {
            fn array_type_info() -> ::sqlx_postgres::PgTypeInfo {
                <::nulid::Nulid as ::sqlx_postgres::PgHasArrayType>::array_type_info()
            }

            fn array_compatible(ty: &::sqlx_postgres::PgTypeInfo) -> bool {
                <::nulid::Nulid as ::sqlx_postgres::PgHasArrayType>::array_compatible(ty)
            }
        }

        #[cfg(feature = "sqlx-postgres")]
        impl #impl_generics ::sqlx_core::encode::Encode<'_, ::sqlx_postgres::Postgres> for #name #ty_generics #where_clause {
            fn encode_by_ref(
                &self,
                buf: &mut ::sqlx_postgres::PgArgumentBuffer,
            ) -> ::core::result::Result<::sqlx_core::encode::IsNull, ::sqlx_core::error::BoxDynError> {
                <::nulid::Nulid as ::sqlx_core::encode::Encode<::sqlx_postgres::Postgres>>::encode_by_ref(&self.0, buf)
            }
        }

        #[cfg(feature = "sqlx-postgres")]
        impl<'r> ::sqlx_core::decode::Decode<'r, ::sqlx_postgres::Postgres> for #name #where_clause {
            fn decode(
                value: ::sqlx_postgres::PgValueRef<'r>,
            ) -> ::core::result::Result<Self, ::sqlx_core::error::BoxDynError> {
                <::nulid::Nulid as ::sqlx_core::decode::Decode<::sqlx_postgres::Postgres>>::decode(value).map(#name)
            }
        }

        // ====================================================================
        // SQLite implementations
        // ====================================================================

        #[cfg(feature = "sqlx-sqlite")]
        impl #impl_generics ::sqlx_core::types::Type<::sqlx_sqlite::Sqlite> for #name #ty_generics #where_clause {
            fn type_info() -> ::sqlx_sqlite::SqliteTypeInfo {
                <::nulid::Nulid as ::sqlx_core::types::Type<::sqlx_sqlite::Sqlite>>::type_info()
            }
        }

        #[cfg(feature = "sqlx-sqlite")]
        impl<'q> ::sqlx_core::encode::Encode<'q, ::sqlx_sqlite::Sqlite> for #name #ty_generics #where_clause {
            fn encode_by_ref(
                &self,
                args: &mut ::std::vec::Vec<::sqlx_sqlite::SqliteArgumentValue<'q>>,
            ) -> ::core::result::Result<::sqlx_core::encode::IsNull, ::sqlx_core::error::BoxDynError> {
                <::nulid::Nulid as ::sqlx_core::encode::Encode<::sqlx_sqlite::Sqlite>>::encode_by_ref(&self.0, args)
            }
        }

        #[cfg(feature = "sqlx-sqlite")]
        impl<'r> ::sqlx_core::decode::Decode<'r, ::sqlx_sqlite::Sqlite> for #name #where_clause {
            fn decode(
                value: ::sqlx_sqlite::SqliteValueRef<'r>,
            ) -> ::core::result::Result<Self, ::sqlx_core::error::BoxDynError> {
                <::nulid::Nulid as ::sqlx_core::decode::Decode<::sqlx_sqlite::Sqlite>>::decode(value).map(#name)
            }
        }

        // ====================================================================
        // MySQL/MariaDB implementations
        // ====================================================================

        #[cfg(feature = "sqlx-mysql")]
        impl #impl_generics ::sqlx_core::types::Type<::sqlx_mysql::MySql> for #name #ty_generics #where_clause {
            fn type_info() -> ::sqlx_mysql::MySqlTypeInfo {
                <::nulid::Nulid as ::sqlx_core::types::Type<::sqlx_mysql::MySql>>::type_info()
            }
        }

        #[cfg(feature = "sqlx-mysql")]
        impl #impl_generics ::sqlx_core::encode::Encode<'_, ::sqlx_mysql::MySql> for #name #ty_generics #where_clause {
            fn encode_by_ref(
                &self,
                buf: &mut ::std::vec::Vec<u8>,
            ) -> ::core::result::Result<::sqlx_core::encode::IsNull, ::sqlx_core::error::BoxDynError> {
                <::nulid::Nulid as ::sqlx_core::encode::Encode<::sqlx_mysql::MySql>>::encode_by_ref(&self.0, buf)
            }
        }

        #[cfg(feature = "sqlx-mysql")]
        impl<'r> ::sqlx_core::decode::Decode<'r, ::sqlx_mysql::MySql> for #name #where_clause {
            fn decode(
                value: ::sqlx_mysql::MySqlValueRef<'r>,
            ) -> ::core::result::Result<Self, ::sqlx_core::error::BoxDynError> {
                <::nulid::Nulid as ::sqlx_core::decode::Decode<::sqlx_mysql::MySql>>::decode(value).map(#name)
            }
        }
    }
}

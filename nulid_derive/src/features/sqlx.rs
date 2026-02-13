//! SQLx support for Id-derived types.
//!
//! This module provides code generation for SQLx trait implementations
//! for types that derive `Id`, delegating to the inner `Nulid`'s SQLx support.
//! Supports PostgreSQL, SQLite, and MySQL/MariaDB databases.

use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

/// Generates SQLx trait implementations for the Id wrapper type.
///
/// This generates implementations for PostgreSQL, SQLite, and MySQL/MariaDB:
///
/// PostgreSQL:
/// - `Type<Postgres>`, `Encode`, `Decode`, and `PgHasArrayType`
///
/// SQLite:
/// - `Type<Sqlite>`, `Encode`, and `Decode`
///
/// MySQL/MariaDB:
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

        // ====================================================================
        // SQLite implementations
        // ====================================================================

        #[cfg(feature = "sqlx")]
        impl #impl_generics ::sqlx::Type<::sqlx::Sqlite> for #name #ty_generics #where_clause {
            fn type_info() -> ::sqlx::sqlite::SqliteTypeInfo {
                <::nulid::Nulid as ::sqlx::Type<::sqlx::Sqlite>>::type_info()
            }
        }

        #[cfg(feature = "sqlx")]
        impl<'q> ::sqlx::Encode<'q, ::sqlx::Sqlite> for #name #ty_generics #where_clause {
            fn encode_by_ref(
                &self,
                args: &mut ::std::vec::Vec<::sqlx::sqlite::SqliteArgumentValue<'q>>,
            ) -> ::core::result::Result<::sqlx::encode::IsNull, ::sqlx::error::BoxDynError> {
                <::nulid::Nulid as ::sqlx::Encode<::sqlx::Sqlite>>::encode_by_ref(&self.0, args)
            }
        }

        #[cfg(feature = "sqlx")]
        impl<'r> ::sqlx::Decode<'r, ::sqlx::Sqlite> for #name #where_clause {
            fn decode(
                value: ::sqlx::sqlite::SqliteValueRef<'r>,
            ) -> ::core::result::Result<Self, ::sqlx::error::BoxDynError> {
                <::nulid::Nulid as ::sqlx::Decode<::sqlx::Sqlite>>::decode(value).map(#name)
            }
        }

        // ====================================================================
        // MySQL/MariaDB implementations
        // ====================================================================

        #[cfg(feature = "sqlx")]
        impl #impl_generics ::sqlx::Type<::sqlx::MySql> for #name #ty_generics #where_clause {
            fn type_info() -> ::sqlx::mysql::MySqlTypeInfo {
                <::nulid::Nulid as ::sqlx::Type<::sqlx::MySql>>::type_info()
            }
        }

        #[cfg(feature = "sqlx")]
        impl #impl_generics ::sqlx::Encode<'_, ::sqlx::MySql> for #name #ty_generics #where_clause {
            fn encode_by_ref(
                &self,
                buf: &mut ::std::vec::Vec<u8>,
            ) -> ::core::result::Result<::sqlx::encode::IsNull, ::sqlx::error::BoxDynError> {
                <::nulid::Nulid as ::sqlx::Encode<::sqlx::MySql>>::encode_by_ref(&self.0, buf)
            }
        }

        #[cfg(feature = "sqlx")]
        impl<'r> ::sqlx::Decode<'r, ::sqlx::MySql> for #name #where_clause {
            fn decode(
                value: ::sqlx::mysql::MySqlValueRef<'r>,
            ) -> ::core::result::Result<Self, ::sqlx::error::BoxDynError> {
                <::nulid::Nulid as ::sqlx::Decode<::sqlx::MySql>>::decode(value).map(#name)
            }
        }
    }
}

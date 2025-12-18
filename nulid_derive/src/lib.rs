//! Derive macros for types that wrap `Nulid`.
//!
//! This crate provides derive macros to automatically implement common traits
//! for newtype wrappers around `Nulid`.
//!
//! # Examples
//!
//! ```ignore
//! use nulid::Nulid;
//! use nulid_derive::Id;
//!
//! #[derive(Id)]
//! pub struct UserId(Nulid);
//!
//! // Now you can use TryFrom:
//! let id = UserId::try_from("01HZQWER4TYUIOP9876QWERTY5")?;
//! let id2 = UserId::try_from("01HZQWER4TYUIOP9876QWERTY5".to_string())?;
//! ```

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields};

/// Derives common traits for types that wrap `Nulid`.
///
/// This macro implements the following traits for a newtype wrapper:
/// - `TryFrom<String>`
/// - `TryFrom<&str>`
/// - `From<Nulid>`
/// - `From<T> for Nulid` (where T is your wrapper type)
/// - `AsRef<Nulid>`
/// - `std::fmt::Display`
/// - `std::str::FromStr`
///
/// # Requirements
///
/// The type must be a tuple struct with exactly one field of type `Nulid`.
///
/// # Examples
///
/// ```ignore
/// use nulid::Nulid;
/// use nulid_derive::Id;
///
/// #[derive(Id, Debug, Clone, Copy, PartialEq, Eq, Hash)]
/// pub struct UserId(Nulid);
///
/// #[derive(Id)]
/// pub struct OrderId(pub Nulid);
///
/// // Usage:
/// let user_id = UserId::try_from("01HZQWER4TYUIOP9876QWERTY5")?;
/// let nulid = Nulid::from(user_id);
/// let user_id2 = UserId::from(nulid);
/// println!("{}", user_id); // Prints the Base32-encoded string
/// ```
#[proc_macro_derive(Id)]
pub fn derive_id(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    // Validate that this is a tuple struct with one field
    let Data::Struct(data_struct) = &input.data else {
        return syn::Error::new_spanned(&input, "Id can only be derived for structs")
            .to_compile_error()
            .into();
    };

    let Fields::Unnamed(fields) = &data_struct.fields else {
        return syn::Error::new_spanned(
            &data_struct.fields,
            "Id requires a tuple struct (e.g., struct UserId(Nulid))",
        )
        .to_compile_error()
        .into();
    };

    if fields.unnamed.len() != 1 {
        return syn::Error::new_spanned(&fields.unnamed, "Id requires exactly one field")
            .to_compile_error()
            .into();
    }

    let expanded = quote! {
        impl #impl_generics ::std::convert::TryFrom<::std::string::String> for #name #ty_generics #where_clause {
            type Error = ::nulid::Error;

            fn try_from(s: ::std::string::String) -> ::std::result::Result<Self, Self::Error> {
                use ::std::str::FromStr;
                ::nulid::Nulid::from_str(&s).map(#name)
            }
        }

        impl #impl_generics ::std::convert::TryFrom<&str> for #name #ty_generics #where_clause {
            type Error = ::nulid::Error;

            fn try_from(s: &str) -> ::std::result::Result<Self, Self::Error> {
                use ::std::str::FromStr;
                ::nulid::Nulid::from_str(s).map(#name)
            }
        }

        impl #impl_generics ::std::convert::From<::nulid::Nulid> for #name #ty_generics #where_clause {
            fn from(nulid: ::nulid::Nulid) -> Self {
                #name(nulid)
            }
        }

        impl #impl_generics ::std::convert::From<#name #ty_generics> for ::nulid::Nulid #where_clause {
            fn from(wrapper: #name #ty_generics) -> Self {
                wrapper.0
            }
        }

        impl #impl_generics ::std::convert::AsRef<::nulid::Nulid> for #name #ty_generics #where_clause {
            fn as_ref(&self) -> &::nulid::Nulid {
                &self.0
            }
        }

        impl #impl_generics ::std::fmt::Display for #name #ty_generics #where_clause {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                ::std::fmt::Display::fmt(&self.0, f)
            }
        }

        impl #impl_generics ::std::str::FromStr for #name #ty_generics #where_clause {
            type Err = ::nulid::Error;

            fn from_str(s: &str) -> ::std::result::Result<Self, Self::Err> {
                ::nulid::Nulid::from_str(s).map(#name)
            }
        }
    };

    TokenStream::from(expanded)
}

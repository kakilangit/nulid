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
use syn::{Data, DeriveInput, Fields, parse_macro_input};

/// Derives common traits for types that wrap `Nulid`.
///
/// This macro implements the following traits for a newtype wrapper:
/// - `TryFrom<String>`
/// - `TryFrom<&str>`
/// - `From<Nulid>`
/// - `From<T> for Nulid` (where T is your wrapper type)
/// - `AsRef<Nulid>`
/// - `std::fmt::Display`
/// - `std::fmt::Debug`
/// - `std::str::FromStr`
/// - `Copy` (which automatically provides `Clone`)
/// - `PartialEq` and `PartialEq<Nulid>` - Equality comparison with wrapper and inner type
/// - `Eq`
/// - `PartialOrd` and `PartialOrd<Nulid>` - Ordering comparison with wrapper and inner type
/// - `Ord`
/// - `Hash`
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
/// #[derive(Id)]
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
///
/// // Comparison works automatically
/// assert_eq!(user_id, user_id2);
/// assert!(user_id <= user_id2);
///
/// // Direct comparison with Nulid
/// assert_eq!(user_id, nulid);
/// assert!(user_id <= nulid);
/// ```
#[allow(clippy::too_many_lines)]
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

        impl #impl_generics ::std::fmt::Debug for #name #ty_generics #where_clause {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                f.debug_tuple(::std::stringify!(#name))
                    .field(&self.0)
                    .finish()
            }
        }

        #[allow(clippy::expl_impl_clone_on_copy)]
        impl #impl_generics ::std::clone::Clone for #name #ty_generics #where_clause {
            fn clone(&self) -> Self {
                *self
            }
        }

        impl #impl_generics ::std::marker::Copy for #name #ty_generics #where_clause {}

        impl #impl_generics ::std::cmp::PartialEq for #name #ty_generics #where_clause {
            fn eq(&self, other: &Self) -> bool {
                self.0 == other.0
            }
        }

        impl #impl_generics ::std::cmp::Eq for #name #ty_generics #where_clause {}

        impl #impl_generics ::std::cmp::PartialOrd for #name #ty_generics #where_clause {
            fn partial_cmp(&self, other: &Self) -> ::std::option::Option<::std::cmp::Ordering> {
                ::std::option::Option::Some(self.cmp(other))
            }
        }

        impl #impl_generics ::std::cmp::Ord for #name #ty_generics #where_clause {
            fn cmp(&self, other: &Self) -> ::std::cmp::Ordering {
                self.0.cmp(&other.0)
            }
        }

        impl #impl_generics ::std::hash::Hash for #name #ty_generics #where_clause {
            fn hash<H: ::std::hash::Hasher>(&self, state: &mut H) {
                self.0.hash(state);
            }
        }

        impl #impl_generics ::std::cmp::PartialEq<::nulid::Nulid> for #name #ty_generics #where_clause {
            fn eq(&self, other: &::nulid::Nulid) -> bool {
                self.0 == *other
            }
        }

        impl #impl_generics ::std::cmp::PartialOrd<::nulid::Nulid> for #name #ty_generics #where_clause {
            fn partial_cmp(&self, other: &::nulid::Nulid) -> ::std::option::Option<::std::cmp::Ordering> {
                self.0.partial_cmp(other)
            }
        }
    };

    TokenStream::from(expanded)
}

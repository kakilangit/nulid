//! Procedural macros for convenient NULID generation.
//!
//! This crate provides macros to simplify working with NULIDs in your code.
//!
//! # Examples
//!
//! ```ignore
//! use nulid::nulid;
//!
//! // Generate a new NULID (panics on error)
//! let id = nulid!();
//!
//! // Generate with explicit error handling
//! let id = nulid!(?);
//! ```

use proc_macro::TokenStream;
use quote::quote;

/// Generates a new NULID at compile time.
///
/// This macro provides a convenient way to generate NULIDs without verbose error handling.
///
/// # Variants
///
/// - `nulid!()` - Generates a NULID, panicking on error (use in contexts where failure is acceptable)
/// - `nulid!(?)` - Returns `Result<Nulid, Error>` for explicit error handling
///
/// # Examples
///
/// ```ignore
/// use nulid::nulid;
///
/// // Simple generation (panics on error)
/// let id = nulid!();
/// println!("Generated ID: {}", id);
///
/// // With error handling
/// fn create_id() -> nulid::Result<nulid::Nulid> {
///     let id = nulid!(?)?;
///     Ok(id)
/// }
///
/// // In a function that can handle errors
/// let id = nulid!(?).expect("Failed to generate NULID");
/// ```
///
/// # Panics
///
/// The `nulid!()` variant (without `?`) will panic if NULID generation fails,
/// which can happen if the system's random number generator is unavailable.
///
/// Use `nulid!(?)` if you need to handle errors gracefully.
#[proc_macro]
pub fn nulid(input: TokenStream) -> TokenStream {
    // Check if "?" was passed as an argument for fallible mode
    let fallible_mode = if input.is_empty() {
        false
    } else {
        // Parse as a single token
        let input_str = input.to_string();
        let trimmed = input_str.trim();

        if trimmed == "?" {
            true
        } else {
            return syn::Error::new(
                proc_macro2::Span::call_site(),
                "expected `?` or no argument; usage: nulid!() or nulid!(?)",
            )
            .to_compile_error()
            .into();
        }
    };

    let expanded = if fallible_mode {
        // Return Result for error handling
        quote! {
            ::nulid::Nulid::new()
        }
    } else {
        // Panic on error for convenience
        quote! {
            ::nulid::Nulid::new().expect("Failed to generate NULID")
        }
    };

    TokenStream::from(expanded)
}

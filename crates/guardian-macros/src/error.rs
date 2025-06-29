//! Error handling utilities for guardian-macros

use syn::{Error as Syn, spanned::Spanned};

/// Custom error type for guardian-macros
pub type Error = Syn;

/// Create a new compilation error with a message
pub fn fault<T: Spanned + quote::ToTokens>(tokens: T, message: &str) -> Error {
    Syn::new_spanned(tokens, message)
}

/// Create a new compilation error with a message and suggestion
pub fn fault_with_help<T: Spanned + quote::ToTokens>(tokens: T, message: &str, help: &str) -> Error {
    let mut error = Syn::new_spanned(&tokens, message);
    error.combine(Syn::new_spanned(&tokens, help));
    error
} 
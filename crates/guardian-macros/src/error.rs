//! Error handling utilities for guardian-macros

use syn::{Error as SynError, spanned::Spanned};

/// Custom error type for guardian-macros
pub type Error = SynError;

/// Create a new compilation error with a message
pub fn new_error<T: Spanned + quote::ToTokens>(tokens: T, message: &str) -> Error {
    SynError::new_spanned(tokens, message)
}

/// Create a new compilation error with a message and suggestion
pub fn new_error_with_help<T: Spanned + quote::ToTokens>(tokens: T, message: &str, help: &str) -> Error {
    let mut error = SynError::new_spanned(&tokens, message);
    error.combine(SynError::new_spanned(&tokens, help));
    error
} 
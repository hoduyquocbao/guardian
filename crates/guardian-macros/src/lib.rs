//! Procedural macros for Guardian-Store
//! 
//! Provides the #[frame] attribute macro for defining binary layouts
//! with single-word identifier philosophy.

use proc_macro::TokenStream;

mod definition;
mod generator;
mod error;

use definition::Layout;
use generator::generate_frame;

/// Procedural macro for defining binary frame layouts
/// 
/// This macro generates a struct and implementation for parsing
/// binary data according to a specified layout, following the
/// single-word identifier philosophy.
/// 
/// # Example
/// ```rust
/// use guardian_macros::frame;
/// 
/// #[frame(version = 1, endian = "be")]
/// pub struct Packet {
///     id: u32,
///     kind: u16,
///     name: str(16),
///     payload: rest,
/// }
/// ```
#[proc_macro_attribute]
pub fn frame(attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse the input into our internal Layout representation
    let layout = match Layout::parse(attr, item) {
        Ok(layout) => layout,
        Err(error) => return error.into_compile_error().into(),
    };
    
    // Generate the frame implementation
    match generate_frame(&layout) {
        Ok(tokens) => tokens.into(),
        Err(error) => error.into_compile_error().into(),
    }
} 
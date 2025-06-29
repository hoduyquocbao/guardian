//! Code generation for guardian-macros

use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

use crate::definition::{Layout, Kind, Endian};
use crate::error::Error;

/// Generate frame implementation from layout
pub fn generate_frame(layout: &Layout) -> Result<TokenStream, Error> {
    let struct_name = &layout.name;
    let attributes = &layout.attributes;
    let fields = &layout.fields;
    
    // Calculate minimum size for fixed fields
    let min_size = calculate_min_size(fields);
    
    // Generate accessor methods
    let mut accessor_methods = Vec::new();
    let mut offset = 0usize;
    
    for field in fields {
        let method = generate_accessor(field, offset)?;
        accessor_methods.push(method);
        
        // Update offset for next field
        offset += field_size(field);
    }
    
    // Generate version method if specified
    let version_method = if let Some(version) = attributes.version {
        quote! {
            pub fn version(&self) -> u8 {
                #version
            }
        }
    } else {
        quote! {}
    };
    
    // Generate the complete implementation
    let expanded = quote! {
        #[derive(Debug, Clone)]
        pub struct #struct_name<'a> {
            source: &'a [u8],
        }
        
        impl<'a> #struct_name<'a> {
            pub fn new(source: &'a [u8]) -> Result<Self, guardian_store::Error> {
                if source.len() < #min_size {
                    return Err(guardian_store::Error::Format("Insufficient data".to_string()));
                }
                
                Ok(Self { source })
            }
            
            #(#accessor_methods)*
            
            #version_method
            
            pub fn size(&self) -> usize {
                self.source.len()
            }
        }
    };
    
    Ok(expanded)
}

/// Generate accessor method for a field
fn generate_accessor(field: &crate::definition::Field, offset: usize) -> Result<TokenStream, Error> {
    let field_name = &field.name;
    let method_name = Ident::new(&field_name.to_string(), field_name.span());
    
    let access_pattern = match &field.kind {
        Kind::Integer { bits, signed, endian } => {
            generate_integer_access(offset, *bits, *signed, endian)?
        }
        Kind::Str { size } => {
            quote! {
                std::str::from_utf8(&self.source[#offset..#offset + #size])
                    .unwrap_or("")
            }
        }
        Kind::Bytes { size } => {
            quote! {
                &self.source[#offset..#offset + #size]
            }
        }
        Kind::Rest => {
            quote! {
                &self.source[#offset..]
            }
        }
    };
    
    let return_type = generate_return_type(&field.kind);
    
    Ok(quote! {
        pub fn #method_name(&self) -> #return_type {
            #access_pattern
        }
    })
}

/// Generate integer access pattern
fn generate_integer_access(offset: usize, bits: u8, signed: bool, endian: &Option<Endian>) -> Result<TokenStream, Error> {
    let bytes = (bits / 8) as usize;
    let endian_expr = match endian {
        Some(Endian::Big) => quote! { from_be_bytes },
        Some(Endian::Little) => quote! { from_le_bytes },
        None => quote! { from_be_bytes }, // Default to big endian
    };
    
    let type_name = if signed {
        match bits {
            8 => quote! { i8 },
            16 => quote! { i16 },
            32 => quote! { i32 },
            64 => quote! { i64 },
            _ => return Err(crate::error::new_error(offset, "Unsupported integer size")),
        }
    } else {
        match bits {
            8 => quote! { u8 },
            16 => quote! { u16 },
            32 => quote! { u32 },
            64 => quote! { u64 },
            _ => return Err(crate::error::new_error(offset, "Unsupported integer size")),
        }
    };
    
    // Generate byte array for the integer
    let mut byte_indices = Vec::new();
    for i in 0..bytes {
        byte_indices.push(quote! { self.source[#offset + #i] });
    }
    
    Ok(quote! {
        #type_name::#endian_expr([#(#byte_indices),*])
    })
}

/// Generate return type for a field
fn generate_return_type(kind: &Kind) -> TokenStream {
    match kind {
        Kind::Integer { bits, signed, .. } => {
            if *signed {
                match bits {
                    8 => quote! { i8 },
                    16 => quote! { i16 },
                    32 => quote! { i32 },
                    64 => quote! { i64 },
                    _ => quote! { i64 }, // Default fallback
                }
            } else {
                match bits {
                    8 => quote! { u8 },
                    16 => quote! { u16 },
                    32 => quote! { u32 },
                    64 => quote! { u64 },
                    _ => quote! { u64 }, // Default fallback
                }
            }
        }
        Kind::Str { .. } => quote! { &str },
        Kind::Bytes { .. } => quote! { &[u8] },
        Kind::Rest => quote! { &[u8] },
    }
}

/// Calculate minimum size for fixed fields
fn calculate_min_size(fields: &[crate::definition::Field]) -> usize {
    fields.iter()
        .filter_map(|field| {
            if matches!(field.kind, Kind::Rest) {
                None
            } else {
                Some(field_size(field))
            }
        })
        .sum()
}

/// Get size of a field
fn field_size(field: &crate::definition::Field) -> usize {
    match &field.kind {
        Kind::Integer { bits, .. } => (*bits / 8) as usize,
        Kind::Str { size } => *size,
        Kind::Bytes { size } => *size,
        Kind::Rest => 0, // Variable size
    }
} 
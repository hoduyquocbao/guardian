//! Code generation for guardian-macros

use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

use crate::definition::{Layout, Kind, Endian};
use crate::error::{fault, Error};

/// Generate frame implementation from layout
pub fn generate(layout: &Layout) -> Result<TokenStream, Error> {
    let struct_name = &layout.name;
    let attributes = &layout.attributes;
    let fields = &layout.fields;
    
    // Calculate minimum size for fixed fields
    let min = calculate_min(fields);
    
    // Generate accessor methods
    let mut accessors = Vec::new();
    let mut offset = 0usize;
    
    for field in fields {
        let method = generate_accessor(field, offset)?;
        accessors.push(method);
        
        // Update offset for next field
        offset += size(field);
    }
    
    // Generate version method if specified
    let version = if let Some(version) = attributes.version {
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
            pub fn new(source: &'a [u8]) -> Result<Self, std::io::Error> {
                if source.len() < #min {
                    return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Insufficient data"));
                }
                
                Ok(Self { source })
            }
            
            #(#accessors)*
            
            #version
            
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
    
    let access = match &field.kind {
        Kind::Integer { bits, signed, endian } => {
            generate_int(offset, *bits, *signed, endian)?
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
    
    let returns = generate_returns(&field.kind);
    
    Ok(quote! {
        pub fn #method_name(&self) -> #returns {
            #access
        }
    })
}

/// Generate integer access pattern
fn generate_int(offset: usize, bits: u8, signed: bool, endian: &Option<Endian>) -> Result<TokenStream, Error> {
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
            _ => return Err(fault(offset, "Unsupported integer size")),
        }
    } else {
        match bits {
            8 => quote! { u8 },
            16 => quote! { u16 },
            32 => quote! { u32 },
            64 => quote! { u64 },
            _ => return Err(fault(offset, "Unsupported integer size")),
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
fn generate_returns(kind: &Kind) -> TokenStream {
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
fn calculate_min(fields: &[crate::definition::Field]) -> usize {
    fields.iter()
        .filter_map(|field| {
            if matches!(field.kind, Kind::Rest) {
                None
            } else {
                Some(size(field))
            }
        })
        .sum()
}

/// Get size of a field
fn size(field: &crate::definition::Field) -> usize {
    match &field.kind {
        Kind::Integer { bits, .. } => (*bits / 8) as usize,
        Kind::Str { size } => *size,
        Kind::Bytes { size } => *size,
        Kind::Rest => 0, // Variable size
    }
} 
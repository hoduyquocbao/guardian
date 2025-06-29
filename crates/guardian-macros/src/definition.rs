//! Layout definition parsing for guardian-macros

use proc_macro::TokenStream;
use proc_macro2::TokenStream as Tokens;
use syn::{
    parse2,
    Ident, ItemStruct, Type, TypePath,
};

use crate::error::{fault, Error};

/// Endianness specification
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Endian {
    Big,
    Little,
}

/// Frame attributes configuration
#[derive(Debug, Clone)]
pub struct Attributes {
    pub version: Option<u8>,
    pub endian: Endian,
    pub check: bool,
}

impl Default for Attributes {
    fn default() -> Self {
        Self {
            version: None,
            endian: Endian::Big,
            check: true,
        }
    }
}

/// Field kind classification
#[derive(Debug, Clone)]
pub enum Kind {
    Integer {
        bits: u8,
        signed: bool,
        endian: Option<Endian>,
    },
    Str {
        size: usize,
    },
    Bytes {
        size: usize,
    },
    Rest,
}

/// Field definition
#[derive(Debug, Clone)]
pub struct Field {
    pub name: Ident,
    pub kind: Kind,
}

/// Layout specification
#[derive(Debug, Clone)]
pub struct Layout {
    pub name: Ident,
    pub attributes: Attributes,
    pub fields: Vec<Field>,
}

impl Layout {
    /// Parse attribute and item tokens into a Layout
    pub fn parse(attr: TokenStream, item: TokenStream) -> Result<Self, Error> {
        let attributes = Self::parse_attrs(attr)?;
        let item_tokens: Tokens = item.into();
        let item_struct = parse2::<ItemStruct>(item_tokens.clone())
            .map_err(|e| fault(&item_tokens, &format!("Failed to parse struct: {}", e)))?;
        
        let mut fields = Vec::new();
        for field in item_struct.fields {
            let field_def = Self::parse_field(field, &attributes.endian)?;
            fields.push(field_def);
        }
        
        Ok(Layout {
            name: item_struct.ident,
            attributes,
            fields,
        })
    }
    
    /// Parse frame attributes
    fn parse_attrs(_attr: TokenStream) -> Result<Attributes, Error> {
        // Simplified attribute parsing for now
        // TODO: Implement proper attribute parsing
        Ok(Attributes::default())
    }
    
    /// Parse a field definition
    fn parse_field(field: syn::Field, default_endian: &Endian) -> Result<Field, Error> {
        let name = field.ident
            .clone()
            .ok_or_else(|| fault(&field, "Field must have a name"))?;
        
        let kind = Self::parse_type(&field.ty, default_endian)?;
        
        Ok(Field { name, kind })
    }
    
    /// Parse field type to determine kind
    fn parse_type(ty: &Type, default_endian: &Endian) -> Result<Kind, Error> {
        match ty {
            Type::Path(TypePath { path, .. }) => {
                let segments = &path.segments;
                if segments.len() == 1 {
                    let segment = &segments[0];
                    let ident = &segment.ident;
                    let ident_str = ident.to_string();
                    
                    // Handle integer types
                    if let Some((bits, signed, endian_override)) = Self::parse_int(&ident_str) {
                        return Ok(Kind::Integer {
                            bits,
                            signed,
                            endian: endian_override.or(Some(*default_endian)),
                        });
                    }
                    
                    // Handle str(n) syntax
                    if ident_str.starts_with("str") {
                        return Self::parse_str(ident);
                    }
                    
                    // Handle bytes(n) syntax
                    if ident_str.starts_with("bytes") {
                        return Self::parse_bytes(ident);
                    }
                    
                    // Handle rest keyword
                    if ident_str == "rest" {
                        return Ok(Kind::Rest);
                    }
                }
                
                Err(fault(ty, "Unsupported field type"))
            }
            _ => Err(fault(ty, "Unsupported field type")),
        }
    }
    
    /// Parse integer type with optional endian suffix
    fn parse_int(ident: &str) -> Option<(u8, bool, Option<Endian>)> {
        let (base, endian) = if ident.ends_with("_be") {
            (&ident[..ident.len() - 3], Some(Endian::Big))
        } else if ident.ends_with("_le") {
            (&ident[..ident.len() - 3], Some(Endian::Little))
        } else {
            (ident, None)
        };
        
        let (bits, signed) = match base {
            "u8" => (8, false),
            "i8" => (8, true),
            "u16" => (16, false),
            "i16" => (16, true),
            "u32" => (32, false),
            "i32" => (32, true),
            "u64" => (64, false),
            "i64" => (64, true),
            _ => return None,
        };
        
        Some((bits, signed, endian))
    }
    
    /// Parse str(n) type
    fn parse_str(ident: &Ident) -> Result<Kind, Error> {
        let ident_str = ident.to_string();
        if !ident_str.starts_with("str(") || !ident_str.ends_with(")") {
            return Err(fault(ident, "Expected str(n) format"));
        }
        
        let size_str = &ident_str[4..ident_str.len() - 1];
        let size: usize = size_str.parse()
            .map_err(|_| fault(ident, "Invalid size in str(n)"))?;
        
        Ok(Kind::Str { size })
    }
    
    /// Parse bytes(n) type
    fn parse_bytes(ident: &Ident) -> Result<Kind, Error> {
        let ident_str = ident.to_string();
        if !ident_str.starts_with("bytes(") || !ident_str.ends_with(")") {
            return Err(fault(ident, "Expected bytes(n) format"));
        }
        
        let size_str = &ident_str[6..ident_str.len() - 1];
        let size: usize = size_str.parse()
            .map_err(|_| fault(ident, "Invalid size in bytes(n)"))?;
        
        Ok(Kind::Bytes { size })
    }
} 
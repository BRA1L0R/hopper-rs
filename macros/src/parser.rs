use crate::errors::Error;
use syn::{Ident, Type, DataStruct, DeriveInput, Data};


/// # Parses derive macro input.
/// ## Success
///
/// In case of success, returns tuple with:
/// Vector of field idents ( Vec<syn::Ident>  );
/// Vector of field types  ( Vec<syn::Type>   );
/// Ident (name) of the parsed struct (syn::Ident);
///
/// ---
///
/// ## Error
///
/// In case of wrong input, it returns a compile time error.
macro_rules! parse_input {
    ($input:ident) => {
        if let Ok(v) = crate::parser::parse_derive_input(&syn::parse_macro_input!($input as syn::DeriveInput)) {
            v
        } else {
            return r#"compile_error!("Unsupported data structure.\nDon't use enums or unnamed fields (tuple structs)")"#.parse().unwrap();
        }
    };
}

/// # Parses derive macro input.
///
/// Please prefer the use of crate::parser::parse_input!
/// when in a _proc_macro_derive_ function.
///
/// ## Success
///
/// In case of success, returns tuple with:
/// Vector of field idents ( Vec<syn::Ident> );
/// Vector of field types  ( Vec<syn::Type>  );
/// Ident (name) of the parsed struct (syn::Ident);
///
/// ---
///
/// ## Error
///
/// In case of wrong input, it returns crate::errors::Error::InvalidStructErr.
pub fn parse_derive_input(input: &DeriveInput) -> Result<(Vec<Ident>, Vec<Type>, Ident), Error> {
    let fields = if let Data::Struct(DataStruct {
        fields: ref named, ..
    }) = input.data
    {
        named
    } else {
        return Err(Error::InvalidStructErr);
    };

    let mut names = Vec::with_capacity(fields.len());
    let mut types = Vec::with_capacity(fields.len());

    for x in fields {
        names.push(x.ident.clone().unwrap());
        types.push(x.ty.clone());
    }

    Ok((names, types, input.ident.clone()))
}


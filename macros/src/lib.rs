use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, parse_macro_input, DataStruct, Data};

/// Derive macro generating an impl of the trait `crate::protocol::data::Deserialize`.
/// 
/// Do not use on enums or structs with unnamed fields (tuple structs).
/// Every field type needs to implement `crate::protocol::data::Deserialize`.
#[proc_macro_derive(Deserialize)]
pub fn derive_deserialize(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let fields = if let Data::Struct(DataStruct { fields: ref named, ..}) = input.data {
        named
    } else {
        return r#"compile_error!("Unsupported data structure.\nDon't use enums or unnamed fields (tuple structs)")"#.parse().unwrap();
    };

    let typename = input.ident;

    let mut names = Vec::with_capacity(fields.len());
    let mut types = Vec::with_capacity(fields.len());

    for x in fields {
        names.push(x.ident.as_ref().unwrap());
        types.push(x.ty.clone());
    }

    quote!{
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl<R: ::std::io::Read> crate::protocol::data::Deserialize<R> for #typename {
            fn deserialize(reader: &mut R) -> ::std::result::Result<Self, crate::protocol::error::ProtoError> {
                Ok(
                    Self{
                        #(
                            #names: #types::deserialize(reader)?,
                        )*
                    }
                )
            }
        }
    }.into()
}


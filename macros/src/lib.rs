use proc_macro::TokenStream;
use quote::quote;

mod errors;
#[macro_use]
mod parser;

/// Derive macro generating an impl of the trait `crate::protocol::data::Deserialize`.
///
/// Do not use on enums or structs with unnamed fields (tuple structs).
/// Every field type needs to implement `crate::protocol::data::Deserialize`.
#[proc_macro_derive(Deserialize)]
pub fn derive_deserialize(input: TokenStream) -> TokenStream {
    let (names, types, typename) = parse_input!(input);

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

/// Derive macro generating an impl of the trait `crate::protocol::data::Serialize`.
///
/// Do not use on enums or structs with unnamed fields (tuple structs).
/// Every field type needs to implement `crate::protocol::data::Serialize`.
#[proc_macro_derive(Serialize)]
pub fn derive_serialize(input: TokenStream) -> TokenStream {
    let (names, _, typename) = parse_input!(input);

    quote!{
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl<W: ::std::io::Write> crate::protocol::data::Serialize<W> for #typename {
            fn serialize(&self, writer: &mut W) -> ::std::result::Result<(), crate::protocol::error::ProtoError> {
                #(
                    self.#names.serialize(writer)?;
                )*

                Ok(())
            }
        }
    }.into()
}

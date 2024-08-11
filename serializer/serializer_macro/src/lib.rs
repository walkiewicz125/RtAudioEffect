extern crate proc_macro;
macro_rules! debug_println {
    ($($arg:tt)*) => (if ::std::cfg!(debug_assertions) { ::std::println!($($arg)*); })
}
#[proc_macro_derive(ByteMessage)]
pub fn byte_message_macro_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = match syn::parse(input) {
        Ok(ast) => ast,
        Err(e) => return e.to_compile_error().into(),
    };

    match byte_message_impl::impl_byte_message_macro(&ast) {
        Ok(tokens) => {
            debug_println!(
                "ByteMessage implementation for: {}:\n<<<\n{}\n>>>",
                ast.ident,
                tokens
            );
            return tokens;
        }
        Err(e) => {
            return e.to_compile_error().into();
        }
    }
}

mod enum_impl;
mod struct_impl;
mod utils;

mod byte_message_impl {
    extern crate proc_macro2;

    use crate::enum_impl::impl_enum;
    use crate::struct_impl::impl_struct;
    use crate::utils::SerializerCode;

    use quote::quote;
    use syn::spanned::Spanned;
    use syn::{Data, Error};

    pub fn impl_byte_message_macro(
        ast: &syn::DeriveInput,
    ) -> Result<proc_macro::TokenStream, Error> {
        let type_name = &ast.ident;
        let serializer_code: SerializerCode;

        match &ast.data {
            Data::Struct(data_struct) => {
                serializer_code = impl_struct(data_struct)?;
            }
            Data::Enum(data_enum) => {
                serializer_code = impl_enum(type_name, data_enum)?;
            }
            Data::Union(_) => {
                return Err(Error::new(ast.span(), "Unions are not supported"));
            }
        };

        let byte_reader = quote! {
            struct ByteReader {
                bytes: Vec<u8>,
                offset: usize,
            }

            impl ByteReader {
                fn new(bytes: Vec<u8>) -> Self {
                    ByteReader { bytes, offset: 0 }
                }

                fn advance(&mut self, size: usize) {
                    self.offset += size;
                }

                fn peek(&self) -> &[u8] {
                    &self.bytes[self.offset..]
                }

                fn are_all_bytes_consumed(&self) -> bool {
                    self.offset == self.bytes.len()
                }

                fn bytes_consumed(&self) -> usize {
                    self.offset
                }

                fn length(&self) -> usize {
                    self.bytes.len()
                }
            }
        };

        let (serializing, deserializing) = serializer_code;

        let generator = quote! {
            impl ByteMessage for #type_name {
                fn to_bytes(&self) -> Vec<u8> {
                    let mut data = Vec::<u8>::new();
                    #serializing
                    data
                }

                fn from_bytes(&mut self, bytes: Vec<u8>) -> Result<u32, String>{
                    #byte_reader
                    let mut reader = ByteReader::new(bytes);
                    #deserializing
                    if ! reader.are_all_bytes_consumed() {
                        return Err(format!("Not all bytes were consumed. Consumed: {}, Total: {}", reader.bytes_consumed(), reader.length()));
                    }

                    Ok(reader.offset as u32)
                }
            }
        };

        Ok(generator.into())
    }
}

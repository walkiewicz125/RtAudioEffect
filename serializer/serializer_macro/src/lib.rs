extern crate proc_macro;

#[proc_macro_derive(ByteMessage)]
pub fn byte_message_macro_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = syn::parse(input).unwrap();

    // Build the trait implementation
    byte_message_impl::impl_byte_message_macro(&ast)
}

mod byte_message_impl {
    extern crate proc_macro2;

    use proc_macro2::TokenStream;
    use quote::quote;
    use syn::{spanned::Spanned, Error};

    pub fn impl_byte_message_macro(ast: &syn::DeriveInput) -> proc_macro::TokenStream {
        let name = &ast.ident;

        let code;
        match &ast.data {
            syn::Data::Struct(data_struct) => match impl_struct(data_struct) {
                Ok(serializing_code) => {
                    code = serializing_code;
                }
                Err(e) => {
                    return e.into_compile_error().into();
                }
            },
            syn::Data::Enum(data_enum) => match impl_enum(name, data_enum) {
                Ok(serializing_code) => {
                    code = serializing_code;
                }
                Err(e) => {
                    return e.into_compile_error().into();
                }
            },
            syn::Data::Union(_) => {
                return syn::Error::new(ast.span(), "Unions are not supported")
                    .into_compile_error()
                    .into();
            }
        }

        let serializing_fields = code.serialize;
        let deserializing_fields = code.deserialize;

        let generator = quote! {
            impl ByteMessage for #name {
                fn to_bytes(&self) -> Vec<u8> {
                    let mut data = Vec::<u8>::new();
                    #(#serializing_fields)*
                    data
                }

                fn from_bytes(&mut self, bytes: Vec<u8>) -> Result<u32, String>{
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

                    let mut reader = ByteReader::new(bytes);

                    #(#deserializing_fields)*

                    if ! reader.are_all_bytes_consumed() {
                        return Err(format!("Not all bytes were consumed. Consumed: {}, Total: {}", reader.bytes_consumed(), reader.length()));
                    }

                    Ok(reader.offset as u32)
                }
            }
        };
        // print!("Generated impl: {}", quote! { #generator });
        generator.into()
    }

    struct SerializingCode {
        serialize: Vec<TokenStream>,
        deserialize: Vec<TokenStream>,
    }

    impl SerializingCode {
        fn empty() -> Self {
            Self {
                serialize: Vec::new(),
                deserialize: Vec::new(),
            }
        }
    }

    fn impl_struct(data_struct: &syn::DataStruct) -> Result<SerializingCode, Error> {
        let mut code = SerializingCode::empty();

        if let syn::Fields::Named(named) = &data_struct.fields {
            for field in named.named.iter() {
                let field_name = &field.ident.clone().unwrap();

                let field_serialization = quote! {
                    data.extend_from_slice(self.#field_name.to_bytes().as_slice());
                };
                code.serialize.push(field_serialization);

                let field_deserialization = quote! {
                    let size = self.#field_name.from_bytes(reader.peek().to_vec())?;
                    reader.advance(size as usize);
                };
                code.deserialize.push(field_deserialization);
            }
            return Ok(code);
        } else {
            return Err(Error::new(
                data_struct.fields.span(),
                "Only named fields are supported",
            ));
        }
    }

    fn impl_enum(
        enum_name: &syn::Ident,
        data_enum: &syn::DataEnum,
    ) -> Result<SerializingCode, Error> {
        let mut code = SerializingCode::empty();

        for (discriminator_usize, variant) in data_enum.variants.iter().enumerate() {
            let discriminator = discriminator_usize as u32;
            let variant_name = &variant.ident;

            if let Some(_) = &variant.discriminant {
                return Err(Error::new(
                    variant.span(),
                    "Discriminants are not supported",
                ));
            }

            match &variant.fields {
                syn::Fields::Named(named) => {
                    let mut fields_names = Vec::<TokenStream>::new();
                    let mut inner_code_serializing = Vec::<TokenStream>::new();
                    let mut inner_code_deserializing = Vec::<TokenStream>::new();
                    for field in named.named.iter() {
                        let field_name;
                        if let Some(field_name_ident) = &field.ident {
                            field_name = field_name_ident;
                        } else {
                            return Err(Error::new(field.span(), "Named fields are required"));
                        }

                        fields_names.push(quote! { #field_name });
                        inner_code_serializing.push(quote! {
                            data.extend_from_slice(#field_name.to_bytes().as_slice());
                        });
                        inner_code_deserializing.push(quote! {#field_name:{
                            let mut #field = Default::default();
                            let size = #field_name.from_bytes(reader.peek().to_vec())?;
                            reader.advance(size as usize);
                            #field_name}
                        });

                        println!(
                            "Filed serialization named: <<{}>>",
                            inner_code_serializing.last().unwrap()
                        );
                        println!(
                            "Filed deserialization named: <<{}>>",
                            inner_code_deserializing.last().unwrap()
                        );
                    }
                    code.serialize.push(quote! {
                        #enum_name::#variant_name{#(#fields_names),*} => {
                            let value = &(#discriminator as u32).to_ne_bytes();
                            data.extend_from_slice(value);
                            #(#inner_code_serializing)*
                        }
                    });
                    code.deserialize.push(quote! {
                        #discriminator => {
                            *self = #enum_name::#variant_name{
                                #(#inner_code_deserializing),*
                            }
                        }
                    });

                    println!(
                        "code.serialize named: <<{}>>",
                        code.serialize.last().unwrap()
                    );
                    println!(
                        "code.deserialize named: <<{}>>",
                        code.deserialize.last().unwrap()
                    );
                }
                syn::Fields::Unnamed(unnamed) => {
                    let mut fields_names = Vec::<TokenStream>::new();
                    let mut inner_code_serializing = Vec::<TokenStream>::new();
                    let mut inner_code_deserializing = Vec::<TokenStream>::new();
                    for (field_index, field) in unnamed.unnamed.iter().enumerate() {
                        let field_name =
                            syn::Ident::new(&format!("field_{}", field_index), field.span());
                        fields_names.push(quote! { #field_name });

                        inner_code_serializing.push(quote! {
                            data.extend_from_slice(#field_name.to_bytes().as_slice());
                        });
                        inner_code_deserializing.push(quote! {{
                            let mut field : #field = Default::default();
                            let size = field.from_bytes(reader.peek().to_vec())?;
                            reader.advance(size as usize);
                            field}
                        });

                        println!(
                            "Filed serialization unnamed: <<{}>>",
                            inner_code_serializing.last().unwrap()
                        );
                        println!(
                            "Filed deserialization unnamed: <<{}>>",
                            inner_code_deserializing.last().unwrap()
                        );
                    }
                    code.serialize.push(quote! {
                        #enum_name::#variant_name(#(#fields_names),*) => {
                            let value = &(#discriminator as u32).to_ne_bytes();
                            data.extend_from_slice(value);
                            #(#inner_code_serializing)*
                        }
                    });
                    code.deserialize.push(quote! {
                        #discriminator => {
                            *self = #enum_name::#variant_name(
                                #(#inner_code_deserializing),*
                            )
                        }
                    });

                    println!(
                        "code.serialize unnamed: <<{}>>",
                        code.serialize.last().unwrap()
                    );
                    println!(
                        "code.deserialize unnamed: <<{}>>",
                        code.deserialize.last().unwrap()
                    );
                }
                syn::Fields::Unit => {
                    code.serialize.push(quote! {
                        #enum_name::#variant_name => {
                            let value = &(#discriminator as u32).to_ne_bytes();
                            data.extend_from_slice(value);
                        }
                    });
                    code.deserialize.push(quote! {
                        #discriminator => {
                            *self = #enum_name::#variant_name;
                        }
                    });
                }
            }
        }

        let enum_field_serialization = code.serialize;
        let enum_fields_deserialization = code.deserialize;

        let enum_serialization = quote! {
            match self {
                #(#enum_field_serialization)*
            }
        };

        let enum_deserialization = quote! {
            let discriminator = u32::from_ne_bytes(
                reader.peek()[0..4]
                    .try_into()
                    .map_err(|e| format!("Error parsing discriminator: {}", e))?,
            );
            reader.advance(4);

            match discriminator {
                #(#enum_fields_deserialization)*
                _ => {
                    return Err(format!("Invalid discriminator: {}", discriminator));
                }
            }
        };

        return Ok(SerializingCode {
            serialize: vec![enum_serialization],
            deserialize: vec![enum_deserialization],
        });
    }
}

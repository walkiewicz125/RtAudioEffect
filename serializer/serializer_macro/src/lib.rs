extern crate proc_macro;
extern crate proc_macro2;
use proc_macro::TokenStream;

#[proc_macro_derive(Serializable)]
pub fn serializable_macro_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();

    // Build the trait implementation
    serializable::impl_serializable_macro(&ast)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

mod serializable {
    use proc_macro2::TokenStream;
    use quote::quote;
    use syn::{spanned::Spanned, Error, Type};

    extern crate proc_macro2;

    struct SerializingCode {
        serialize: Vec<TokenStream>,
        deserialize: Vec<TokenStream>,
    }

    fn impl_struct(data_struct: &syn::DataStruct) -> Result<SerializingCode, Error> {
        let mut serializing_fields = Vec::new();
        let mut deserializing_fields = Vec::new();

        if let syn::Fields::Named(named) = &data_struct.fields {
            for field in named.named.iter() {
                let field_name = &field.ident.clone().unwrap();
                let field_serialization = quote! {
                    data.extend_from_slice(self.#field_name.get_bytes().as_slice());
                };
                serializing_fields.push(field_serialization);

                let field_deserialization = quote! {
                    let field_size = self.#field_name.from_bytes(&bytes[ind as usize..]);
                    size += field_size;
                    ind += field_size;
                };

                deserializing_fields.push(field_deserialization);
            }

            Ok(SerializingCode {
                serialize: serializing_fields,
                deserialize: deserializing_fields,
            })
        } else {
            return Err(Error::new(
                data_struct.fields.span(),
                "Only named fields are supported",
            ));
        }
    }

    fn impl_enum(data_enum: &syn::DataEnum) -> Result<SerializingCode, Error> {
        let mut matching_arms_serialization = Vec::new();
        let mut matching_arms_deserializing = Vec::new();

        let mut error_opt: Option<syn::Error> = None;

        let aggregate_errors = |error_opt: &mut Option<syn::Error>, new_error| {
            match error_opt.take() {
                Some(mut e) => {
                    e.combine(new_error);
                    error_opt.replace(e);
                }
                None => {
                    error_opt.replace(new_error);
                }
            };
        };

        let mut discriminant: u8 = 0;
        for variant in data_enum.variants.iter() {
            if let Some(_discriminant) = &variant.discriminant {
                aggregate_errors(
                    &mut error_opt,
                    syn::Error::new(variant.span(), "Discriminant are not supported"),
                );
            }
            match &variant.fields {
                syn::Fields::Named(fields_named) => {
                    aggregate_errors(
                        &mut error_opt,
                        syn::Error::new(variant.span(), "Named are not supported"),
                    );
                }
                syn::Fields::Unnamed(fields_unnamed) => {
                    if fields_unnamed.unnamed.len() != 1 {
                        aggregate_errors(
                            &mut error_opt,
                            syn::Error::new(
                                variant.span(),
                                "Only one field is supported for unnamed fields",
                            ),
                        );
                    } else {
                        let variant_name = &variant.ident;
                        let field_serialization = quote! {
                            Self::#variant_name(field) => {
                                data.push(#discriminant);
                                data.extend_from_slice(field.get_bytes().as_slice());
                            }
                        };

                        let field_deserialization: TokenStream;
                        if let Type::Verbatim(ty) = &fields_unnamed.unnamed[0].ty {
                            field_deserialization = quote! {
                                #discriminant => {
                                    let field_size = self.#variant_name.from_bytes(&bytes[ind as usize..]);
                                    size += field_size;
                                    ind += field_size;

                                    *self = Self::#variant_name;
                                    size += 1;
                                }
                            };
                        } else {
                            aggregate_errors(
                                &mut error_opt,
                                syn::Error::new(
                                    fields_unnamed.span(),
                                    "Only types that implement Serializable are supported",
                                ),
                            );
                        }

                        matching_arms_serialization.push(field_serialization);
                        matching_arms_deserializing.push(field_deserialization);
                    }
                }
                syn::Fields::Unit => {
                    let variant_name = &variant.ident;
                    let field_serialization = quote! {
                        Self::#variant_name => {
                            data.push(#discriminant);
                        }
                    };
                    let field_deserialization = quote! {
                        #discriminant => {
                            *self = Self::#variant_name;
                            size += 1;
                        }
                    };

                    matching_arms_serialization.push(field_serialization);
                    matching_arms_deserializing.push(field_deserialization);
                }
            }

            discriminant += 1;
        }

        let serializing_fields = vec![quote! {
            match self {
                #(#matching_arms_serialization)*
            }
        }];

        let deserializing_fields = vec![quote! {
            let discriminator = bytes[ind as usize];
            ind += 1;
            match discriminator {
                #(#matching_arms_deserializing)*
                _ => {
                    panic!("Invalid discriminator value: {}", discriminator);
                }
            }
        }];

        if let Some(error) = error_opt {
            return Err(error);
        } else {
            Ok(SerializingCode {
                serialize: serializing_fields,
                deserialize: deserializing_fields,
            })
        }
    }

    pub fn impl_serializable_macro(ast: &syn::DeriveInput) -> Result<TokenStream, Error> {
        let serializing_fields;
        let deserializing_fields;

        let serializing_code = {
            match &ast.data {
                syn::Data::Struct(data_struct) => impl_struct(data_struct),
                syn::Data::Enum(data_enum) => impl_enum(data_enum),
                syn::Data::Union(_) => {
                    return Err(Error::new(
                        ast.span(),
                        "Union types are not supported for serialization",
                    ));
                }
            }
        }?;

        serializing_fields = serializing_code.serialize;
        deserializing_fields = serializing_code.deserialize;

        let name = &ast.ident;
        let gen = quote! {
            impl Serializable for #name {
                fn get_bytes(&self) -> Vec<u8> {
                    let mut data = Vec::<u8>::new();
                    #(#serializing_fields)*
                    println!("Serialized data: {:?}", data);
                    data
                }

                fn from_bytes(&mut self, bytes: &[u8]) -> u32 {
                    let mut ind: u32 = 0;
                    let mut size: u32 = 0;
                    #(#deserializing_fields)*

                    size
                }

                fn try_from_bytes(bytes: &[u8]) -> Result<Self, String> {
                    let mut instance = Self::default();
                    instance.from_bytes(bytes);

                    Ok(instance)
                }
            }

            impl From<&[u8]> for #name {
                fn from(bytes: &[u8]) -> Self {
                    let mut instance = Self::default();
                    instance.from_bytes(bytes);
                    instance
                }
            }
        };

        Ok(gen.into())
    }
}

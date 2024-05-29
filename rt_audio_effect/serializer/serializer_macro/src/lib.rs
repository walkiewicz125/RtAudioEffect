extern crate proc_macro;

use quote::quote;
use syn::{self, spanned::Spanned};

#[proc_macro_derive(Serializable)]
pub fn serializable_macro_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = syn::parse(input).unwrap();

    // Build the trait implementation
    impl_serializable_macro(&ast)
}

fn impl_serializable_macro(ast: &syn::DeriveInput) -> proc_macro::TokenStream {
    let name = &ast.ident;

    let mut serializing_fields = Vec::new();
    let mut deserializing_fields = Vec::new();

    if let syn::Data::Struct(data_struct) = &ast.data {
        println!("Parsing Struct: {:?}", ast.ident);
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

                println!("Field name: {:?}", field_name);
            }
        }

        let gen = quote! {
            impl Serializable for #name {
                fn get_bytes(&self) -> Vec<u8> {
                    let mut data = Vec::<u8>::new();
                    #(#serializing_fields)*
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

        return gen.into();
    }

    if let syn::Data::Enum(data_enum) = &ast.data {
        println!("Parsing Enum: {:?}", ast.ident);
        let enum_name = &ast.ident;
        for variant in data_enum.variants.iter() {
            let variant_name = &variant.ident.clone();
            println!("Variant name: {:?}", variant_name);

            if variant.fields.is_empty() {
                let variant_serialization = quote! {
                    #enum_name::#variant_name => {
                        panic!("Cannot serialize empty variant");
                    }
                };
                serializing_fields.push(variant_serialization);
            } else {
                for field in variant.fields.iter() {
                    println!("Field name: {:?}", field.ident);

                    let variant_serialization = quote! {
                        #enum_name::#variant_name(inner) => {
                            data.extend_from_slice(inner.get_bytes().as_slice());
                        }
                    };
                    serializing_fields.push(variant_serialization);
                }
            }
        }

        let gen = quote! {
            impl Serializable for #name {
                fn get_bytes(&self) -> Vec<u8> {
                    let mut data = Vec::<u8>::new();
                    match self {
                        #(#serializing_fields)*
                    }
                    data
                }

                fn from_bytes(&mut self, bytes: &[u8]) -> u32 {
                    0
                }

                fn try_from_bytes(bytes: &[u8]) -> Result<Self, String> {
                    Err("Not implemented".to_string())
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

        return gen.into();
    }

    return syn::Error::new(ast.span(), format!("Unsupported type: {:?}", ast.ident))
        .to_compile_error()
        .into();
}

use proc_macro::TokenStream;

#[proc_macro_derive(WireDataSerializer)]
pub fn wire_data_serializer_macro_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();

    // Build the trait implementation
    serializer_private::impl_macro(&ast)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

mod serializer_private {
    use proc_macro2::TokenStream;
    use quote::quote;
    use syn::{spanned::Spanned, Error};

    extern crate proc_macro2;

    struct SerializingCode {
        serialize: Vec<TokenStream>,
        deserialize: Vec<TokenStream>,
    }

    pub fn impl_macro(ast: &syn::DeriveInput) -> Result<TokenStream, Error> {
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
            impl WireDataSerializer for #name {
                fn get_bytes(&self) -> Vec<u8> {
                    let mut data = Vec::<u8>::new();
                    #(#serializing_fields)*
                    data
                }

                fn from_bytes(data_view: &mut DataView) -> Result<Self, String> {
                    // #(#deserializing_fields)*
                    Err("Not implemented".to_string())
                }
            }
        };

        Ok(gen.into())
    }

    fn impl_struct(data_struct: &syn::DataStruct) -> Result<SerializingCode, Error> {
        Ok(SerializingCode {
            serialize: data_struct
                .fields
                .iter()
                .map(|field| {
                    let field_name = field.ident.as_ref().unwrap();
                    let field_type = &field.ty;

                    quote! {
                        let mut bytes = self.#field_name.get_bytes();
                        data.append(&mut bytes);
                    }
                })
                .collect(),
            deserialize: data_struct
                .fields
                .iter()
                .map(|field| {
                    let field_name = field.ident.as_ref().unwrap();
                    let field_type = &field.ty;

                    quote! {
                        let #field_name = <#field_type>::from_bytes(data_view)?;
                    }
                })
                .collect(),
        })
    }

    fn impl_enum(data_enum: &syn::DataEnum) -> Result<SerializingCode, Error> {
        Err(Error::new(
            data_enum.enum_token.span(),
            "Enums are not supported",
        ))
    }
}

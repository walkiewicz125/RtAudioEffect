use crate::utils::SerializerCode;
use proc_macro2::Ident;
use proc_macro2::TokenStream;
use quote::quote;
use syn::FieldsUnnamed;
use syn::{spanned::Spanned, DataEnum, Error, Fields, FieldsNamed};

pub fn impl_enum(enum_name: &Ident, data_enum: &DataEnum) -> Result<SerializerCode, Error> {
    let mut code = SerializerCode::default();
    let (serializing, deserializing) = &mut code;

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
            Fields::Named(named) => {
                let named_serialization =
                    impl_enum_named(enum_name, variant_name, discriminator, named)?;

                serializing.extend(named_serialization.0);
                deserializing.extend(named_serialization.1);
            }
            Fields::Unnamed(unnamed) => {
                let unnamed_serialization =
                    impl_enum_unnamed(enum_name, variant_name, discriminator, unnamed)?;
                serializing.extend(unnamed_serialization.0);
                deserializing.extend(unnamed_serialization.1);
            }
            Fields::Unit => {
                serializing.extend(quote! {
                    #enum_name::#variant_name => {
                        let value = &(#discriminator as u32).to_ne_bytes();
                        data.extend_from_slice(value);
                    }
                });
                deserializing.extend(quote! {
                    #discriminator => {
                        *self = #enum_name::#variant_name;
                    }
                });
            }
        }
    }

    let enum_serialization = quote! {
        match self {
            #serializing
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
            #deserializing
            _ => {
                return Err(format!("Invalid discriminator: {}", discriminator));
            }
        }
    };

    return Ok((enum_serialization, enum_deserialization));
}

fn impl_enum_named(
    enum_name: &Ident,
    variant_name: &Ident,
    discriminator: u32,
    named_fields: &FieldsNamed,
) -> Result<SerializerCode, Error> {
    let mut fields_names = Vec::<TokenStream>::new();
    let mut fields_serializing = Vec::<TokenStream>::new();
    let mut fields_deserializing = Vec::<TokenStream>::new();
    for field in named_fields.named.iter() {
        let field_name = &field
            .ident
            .as_ref()
            .ok_or(Error::new(field.span(), "Named fields are required"))?;

        fields_names.push(quote! { #field_name });

        fields_serializing.push(quote! {
            data.extend_from_slice(#field_name.to_bytes().as_slice());
        });

        fields_deserializing.push(quote! {#field_name:{
            let mut #field = Default::default();
            let size = #field_name.from_bytes(reader.peek().to_vec())?;
            reader.advance(size as usize);
            #field_name}
        });
    }

    let named_serialization = quote! {
        #enum_name::#variant_name{#(#fields_names),*} => {
            let value = &(#discriminator).to_ne_bytes();
            data.extend_from_slice(value);
            #(#fields_serializing)*
        }
    };

    let named_deserialization = quote! {
        #discriminator => {
            *self = #enum_name::#variant_name{
                #(#fields_deserializing),*
            }
        }
    };

    Ok((named_serialization, named_deserialization))
}

fn impl_enum_unnamed(
    enum_name: &Ident,
    variant_name: &Ident,
    discriminator: u32,
    unnamed_fields: &FieldsUnnamed,
) -> Result<SerializerCode, Error> {
    let mut fields_names = Vec::<TokenStream>::new();
    let mut fields_serializing = Vec::<TokenStream>::new();
    let mut fields_deserializing = Vec::<TokenStream>::new();
    for (field_index, field) in unnamed_fields.unnamed.iter().enumerate() {
        let field_name = syn::Ident::new(&format!("field_{}", field_index), field.span());
        fields_names.push(quote! { #field_name });

        fields_serializing.push(quote! {
            data.extend_from_slice(#field_name.to_bytes().as_slice());
        });
        fields_deserializing.push(quote! {{
            let mut field : #field = Default::default();
            let size = field.from_bytes(reader.peek().to_vec())?;
            reader.advance(size as usize);
            field}
        });
    }

    let unnamed_serialization = quote! {
        #enum_name::#variant_name(#(#fields_names),*) => {
            let value = &(#discriminator).to_ne_bytes();
            data.extend_from_slice(value);
            #(#fields_serializing)*
        }
    };

    let unnamed_deserialization = quote! {
        #discriminator => {
            *self = #enum_name::#variant_name(
                #(#fields_deserializing),*
            )
        }
    };

    Ok((unnamed_serialization, unnamed_deserialization))
}

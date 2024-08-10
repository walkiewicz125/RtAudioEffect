use crate::utils::SerializerCode;
use quote::quote;
use syn::{spanned::Spanned, Error};

pub fn impl_struct(data_struct: &syn::DataStruct) -> Result<SerializerCode, Error> {
    let mut code = SerializerCode::default();
    let (serializing, deserializing) = &mut code;

    if let syn::Fields::Named(named) = &data_struct.fields {
        for field in named.named.iter() {
            let field_name = &field.ident.clone().unwrap();

            serializing.extend(quote! {
                data.extend_from_slice(self.#field_name.to_bytes().as_slice());
            });

            deserializing.extend(quote! {
                let size = self.#field_name.from_bytes(reader.peek().to_vec())?;
                reader.advance(size as usize);
            });
        }
        return Ok(code);
    } else {
        return Err(Error::new(
            data_struct.fields.span(),
            "Only named fields are supported",
        ));
    }
}

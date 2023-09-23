use proc_macro2::{Literal, TokenStream};
use quote::quote;
use syn::{Data, DeriveInput, Error};

const BUILD_GENOME_ATTR_IDENT: &'static str = "build_genome";

pub(crate) fn derive_build_genome(input: DeriveInput) -> syn::Result<TokenStream> {
    let ident = input.ident.clone();
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let data = if let Data::Struct(data) = input.data {
        Ok(data)
    } else {
        Err(Error::new_spanned(
            input.clone(),
            "Only structs are allowed",
        ))
    }?;

    let mut tokens = Vec::new();
    for field in data.fields {
        let field_ident = field.ident.as_ref().ok_or(Error::new_spanned(
            field.clone(),
            "Only named fields are supported",
        ))?;
        let field_literal = Literal::string(field_ident.to_string().as_str());
        for attr in field.attrs {
            if attr.path().is_ident(BUILD_GENOME_ATTR_IDENT) {
                attr.parse_nested_meta(|meta| {
                    let path = &meta.path;
                    if path.is_ident("nested") {
                        tokens.push(quote!(
                            self.#field_ident.build_genome(builder.nested(#field_literal));
                        ));
                        Ok(())
                    } else if path.is_ident("gen") {
                        tokens.push(quote!(
                            builder.add(#field_literal, crate::genome::Gen {
                                value: self.#field_ident,
                            });
                        ));
                        Ok(())
                    } else {
                        Err(Error::new_spanned(attr.clone(), "Wrong attribute argument"))
                    }
                })?;
            }
        }
    }

    Ok(quote! {
      impl #impl_generics crate::genome::BuildGenome for #ident #ty_generics #where_clause {
        fn build_genome(&self, builder: crate::genome::GenomeBuilder) {
          #(#tokens)*
        }
      }
    })
}

//! Derive the [`HasIdMut`] trait.
use crate::common;
use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, Field, Fields};

pub(crate) fn impl_has_id_mut(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let Data::Struct(obj) = &ast.data else {
        panic!("derive `HasIdMut` can only be appied to structs");
    };

    let fields = match &obj.fields {
        Fields::Unnamed(field) => &field.unnamed,
        Fields::Named(field) => &field.named,
        Fields::Unit => panic!("can not derive `HasId` on a unit struct"),
    };

    let id_fields = fields
        .iter()
        .filter(|field| {
            let id_attrs = common::attrs_with_ident(&field.attrs, "id");
            if id_attrs.len() > 1 {
                panic!("multiple `id` attrs")
            }

            id_attrs.len() == 1
        })
        .collect::<Vec<&Field>>();

    if id_fields.len() == 0 {
        panic!("no fields marked as id");
    } else if id_fields.len() > 1 {
        panic!("multiple fields marked as id");
    }

    let id_field = id_fields[0];
    let id_field_ident = &id_field
        .ident
        .as_ref()
        .expect("could not get ident of id field");

    let gen = quote! {
        impl HasIdMut for #name {
            fn id_mut(&mut self) -> &mut Self::Id {
                &mut self.#id_field_ident
            }
        }
    };

    gen.into()
}

#[cfg(test)]
#[path = "./derive_mut_test.rs"]
mod derive_mut_test;

//! Common functions.
use syn::{Attribute, Meta};

/// Filters [`Attribute`]s to those with a certain identity.
pub fn attrs_with_ident<'a>(attrs: &'a Vec<Attribute>, ident: &str) -> Vec<&'a Attribute> {
    attrs
        .into_iter()
        .filter(|attr| {
            let Meta::Path(path) = attr.parse_meta().expect("could not parse meta") else {
                return false;
            };

            let Some(a_ident) = path.get_ident() else {
                return false;
            };

            a_ident == ident
        })
        .collect::<Vec<&Attribute>>()
}

#[cfg(test)]
#[path = "./common_test.rs"]
mod common_test;

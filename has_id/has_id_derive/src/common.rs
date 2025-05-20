//! Common functions.
use syn::Attribute;

/// Filters [`Attribute`]s to those with a certain identity.
pub fn attrs_with_ident<'a>(attrs: &'a Vec<Attribute>, ident: &str) -> Vec<&'a Attribute> {
    attrs
        .into_iter()
        .filter(|attr| attr.path().is_ident(ident))
        .collect::<Vec<&Attribute>>()
}

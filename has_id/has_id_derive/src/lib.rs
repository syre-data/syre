//! Derive functionality for the `HasId` trait.
mod common;
mod derive;
use proc_macro::TokenStream;

// ************
// *** base ***
// ************

#[proc_macro_derive(HasId, attributes(id))]
pub fn has_id_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).expect("could not parse input");

    derive::impl_has_id(&ast)
}

// *************
// *** serde ***
// *************

#[cfg(feature = "serde")]
mod derive_serde;

#[cfg(feature = "serde")]
#[proc_macro_derive(HasIdSerde, attributes(id))]
pub fn has_id_serde_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).expect("could not parse input");

    derive_serde::impl_has_id(&ast)
}

#[cfg(test)]
#[path = "lib_test.rs"]
mod lib_test;

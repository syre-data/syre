//! Derive functionality for the `settings_manager` crate.
mod common;
mod settings;

use proc_macro::TokenStream;

#[proc_macro_derive(Settings, attributes(settings))]
pub fn settings_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).expect("could not parse input");
    settings::impl_settings(&ast)
}

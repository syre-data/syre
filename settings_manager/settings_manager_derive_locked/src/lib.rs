//! Derive functionality for the `settings_manager` crate.
mod common;
mod settings;

use proc_macro::TokenStream;

#[proc_macro_derive(LockedSettings, attributes(locked_settings))]
pub fn locked_settings_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).expect("could not parse input");
    settings::impl_locked_settings(&ast)
}

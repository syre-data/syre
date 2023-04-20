use extendr_api::prelude::*;
pub mod database;

extendr_module! {
    mod resources;
    use database;
}

#[cfg(test)]
#[path = "./mod_test.rs"]
mod mod_test;

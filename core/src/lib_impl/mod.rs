//! Implementations for language bindings and libraries.
#[cfg(feature = "clap")]
pub mod clap;

#[cfg(feature = "extendr")]
pub mod extendr;

#[cfg(feature = "pyo3")]
pub mod pyo3;

#[cfg(feature = "yew")]
pub mod yew;

#[cfg(test)]
#[path = "./mod_test.rs"]
mod mod_test;

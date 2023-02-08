#![feature(mutex_unlock)]
//! User interface for Thot Desktop.
mod app;
mod commands;
mod common;
mod components;
mod error;
mod hooks;
mod navigation;
mod pages;
mod routes;
mod widgets;

use app::App;
pub use error::Result;

fn main() {
    yew::Renderer::<App>::new().render();
}

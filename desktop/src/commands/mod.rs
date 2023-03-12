//! Resources for [`commands`](thot_desktop_tauri::commands).
pub mod asset;
pub mod authenticate;
pub mod common;
pub mod container;
pub mod graph;
pub mod project;
pub mod script;
pub mod settings;
pub mod user;

#[cfg(test)]
#[path = "./mod_test.rs"]
mod mod_test;

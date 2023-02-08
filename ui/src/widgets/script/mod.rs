//! Script and ScriptAssociation components.
pub mod create_script;

// Re-exports
pub use create_script::CreateScript;

#[cfg(test)]
#[path = "./mod_test.rs"]
mod mod_test;

//! Display messages to the user.
pub mod message;
pub mod messages;

// Re-exports
pub use message::Message;
pub use messages::Messages;

#[cfg(test)]
#[path = "./mod_test.rs"]
mod mod_test;

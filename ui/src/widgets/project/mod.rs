//! Project related UI widgets.
pub mod project_card;
pub mod project_deck;
pub mod project_properties;

// Reexports
pub use project_card::ProjectCard;
pub use project_deck::ProjectDeck;

#[cfg(test)]
#[path = "./mod_test.rs"]
mod mod_test;

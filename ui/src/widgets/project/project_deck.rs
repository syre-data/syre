use crate::components::card_deck::{CardDeck, CardDeckProps};
use crate::types::ToKey;
use thot_core::project::Project;
use yew::virtual_dom::Key;

pub type ProjectDeckProps = CardDeckProps<Project>;
pub type ProjectDeck = CardDeck<Project>;

impl ToKey for Project {
    fn key(&self) -> Key {
        self.rid.clone().into()
    }
}

#[cfg(test)]
#[path = "./project_deck_test.rs"]
mod project_deck_test;

use crate::components::card_deck::{CardDeck, CardDeckProps};
use crate::types::ToKey;
use syre_core::project::Project;
use yew::virtual_dom::Key;

pub type ProjectDeckProps = CardDeckProps<Project>;
pub type ProjectDeck = CardDeck<Project>;

impl ToKey for Project {
    fn key(&self) -> Key {
        self.rid.clone().into()
    }
}

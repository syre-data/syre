use super::card::{Card, CardUi};
use crate::types::ToKey;
use yew::prelude::*;
use yew::virtual_dom::Key;

#[derive(Properties, PartialEq)]
pub struct CardDeckProps<T: Clone + PartialEq + CardUi> {
    pub items: Vec<T>,

    #[prop_or_default]
    pub onclick_card: Option<Callback<Key>>,
}

#[function_component(CardDeck)]
pub fn card_deck<T>(props: &CardDeckProps<T>) -> Html
where
    T: 'static + Clone + PartialEq + CardUi + ToKey,
{
    let cards = props
        .items
        .iter()
        .map(|item| {
            let key = item.key();
            let onclick = props.onclick_card.clone().map(move |onclick_card| {
                let key = key.clone();
                let onclick_card = onclick_card.clone();

                Callback::from(move |_: MouseEvent| {
                    onclick_card.emit(key.clone());
                })
            });

            html! {
                <Card<T>
                    key={item.key()}
                    item={item.clone()}
                    {onclick} />
            }
        })
        .collect::<Html>();

    html! {
        <div class={classes!("card-deck")}>{ cards }</div>
    }
}

#[cfg(test)]
#[path = "./card_deck_test.rs"]
mod card_deck_test;

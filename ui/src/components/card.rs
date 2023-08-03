//! A UI card.
use yew::prelude::*;

/// Card interface.
pub trait CardUi {
    /// Card title.
    /// Displayed in an `h2` tag.
    fn title(&self) -> &str;

    /// Card body.
    fn body(&self) -> Html;

    /// Card footer.
    fn footer(&self) -> Option<Html> {
        None
    }
}

/// Properties for a Card.
#[derive(Properties, PartialEq)]
pub struct CardProps<T: PartialEq + CardUi> {
    /// Item to display.
    pub item: T,

    /// Callback to exececute when the card is clicked.
    #[prop_or_default]
    pub onclick: Option<Callback<MouseEvent>>,
}

/// Card component.
#[function_component(Card)]
pub fn card<T>(props: &CardProps<T>) -> Html
where
    T: 'static + PartialEq + CardUi,
{
    let card_ref = use_node_ref();

    let footer = match props.item.footer() {
        None => html! { "" },
        Some(f) => f,
    };

    html! {
        <div ref={card_ref}
            class={classes!("card", "clickable")}
            onclick={props.onclick.clone()}>

            <h2 class={classes!("title")}>{ props.item.title() }</h2>
            <div class={classes!("body")}>{ props.item.body() }</div>
            <div class={classes!("footer")}>{ footer }</div>
        </div>
    }
}

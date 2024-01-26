//! Display messages to user.
use yew::prelude::*;

// ****************
// *** Messages ***
// ****************

#[derive(Properties, PartialEq)]
pub struct MessagesProps {
    /// messages to display.
    #[prop_or_default]
    pub children: Children,
}

#[function_component(Messages)]
pub fn messages(props: &MessagesProps) -> Html {
    html! {
        <div class={classes!("thot-ui-messages")}>
            { props.children.iter().collect::<Html>() }
        </div>
    }
}

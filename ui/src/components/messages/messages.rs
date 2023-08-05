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
    // /// Messages to display.
    // pub messages: Vec<Message>,
}

// @todo: Allow raw `Message`s to be passed in.
#[function_component(Messages)]
pub fn messages(props: &MessagesProps) -> Html {
    html! {
        <div class={classes!("thot-ui-messages")}>
            // { props.messages.iter().map(|m| Into::<Html>::into(m.clone())).collect::<Html>() }
            { props.children.iter().collect::<Html>() }
        </div>
    }
}

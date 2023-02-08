//! Displays messages to the user.
use crate::app::AppStateReducer;
use thot_ui::components::{Message, Messages as MessagesUi};
use yew::prelude::*;

#[function_component(Messages)]
pub fn messages() -> Html {
    let app_state =
        use_context::<AppStateReducer>().expect("could not find `AppStateReducer` context");

    html! {
        // <MessagesUi messages={app_state.messages.clone()} />
        <MessagesUi>
            { app_state.messages.iter().map(|m| html! {
                    <Message kind={m.kind.clone()} message={m.message.clone()} />
                }).collect::<Html>() }
        </MessagesUi>
    }
}

#[cfg(test)]
#[path = "./messages_test.rs"]
mod messages_test;

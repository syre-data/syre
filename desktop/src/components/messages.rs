//! Displays messages to the user.
use crate::app::{AppStateAction, AppStateReducer};
use thot_ui::components::{Message, Messages as MessagesUi};
use uuid::Uuid;
use yew::prelude::*;

#[function_component(Messages)]
pub fn messages() -> Html {
    let app_state =
        use_context::<AppStateReducer>().expect("could not find `AppStateReducer` context");

    let onclose = {
        let app_state = app_state.clone();

        move |id: &Uuid| {
            let app_state = app_state.clone();
            let id = id.clone();

            Callback::from(move |_| {
                app_state.dispatch(AppStateAction::RemoveMessage(id));
            })
        }
    };

    html! {
        // <MessagesUi messages={app_state.messages.clone()} />
        <MessagesUi>
            { app_state.messages.iter().map(|m| html! {
                <Message
                    class={classes!("clickable")}
                    kind={m.kind.clone()}
                    message={m.message.clone()}
                    details={m.details.clone()}
                    onclose={onclose(m.id())} />
            }).collect::<Html>() }
        </MessagesUi>
    }
}

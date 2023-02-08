//! Display error messages
use crate::types::MessageType;
use yew::prelude::*;

// ***************
// *** Message ***
// ***************

#[derive(Properties, PartialEq)]
pub struct MessageProps {
    pub message: AttrValue,
    pub kind: MessageType,
}

#[function_component(Message)]
pub fn message(props: &MessageProps) -> Html {
    let class = match props.kind {
        MessageType::Info => "message-info",
        MessageType::Success => "message-success",
        MessageType::Error => "message-error",
    };

    html! {
        <div class={classes!(class)}>
            { &props.message }
        </div>
    }
}

#[cfg(test)]
#[path = "./message_test.rs"]
mod message_test;

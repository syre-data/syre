//! Display error messages
use crate::types::MessageType;
use yew::prelude::*;

// ***************
// *** Message ***
// ***************

#[derive(Properties, PartialEq)]
pub struct MessageProps {
    #[prop_or_default]
    pub class: Classes,

    pub message: AttrValue,
    pub kind: MessageType,

    #[prop_or_default]
    pub onclick: Callback<()>,
}

#[function_component(Message)]
pub fn message(props: &MessageProps) -> Html {
    let onclick = {
        let onclick = props.onclick.clone();

        Callback::from(move |_: MouseEvent| {
            onclick.emit(());
        })
    };

    let kind_class = match props.kind {
        MessageType::Info => "message-info",
        MessageType::Success => "message-success",
        MessageType::Error => "message-error",
    };

    let class = classes!("thot-ui-message", kind_class, props.class.clone());

    html! {
        <div {class} {onclick}>
            { &props.message }
        </div>
    }
}

#[cfg(test)]
#[path = "./message_test.rs"]
mod message_test;

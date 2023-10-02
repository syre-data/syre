//! Display error messages
use crate::types::MessageType;
use yew::prelude::*;
use yew_icons::{Icon, IconId};

// ***************
// *** Message ***
// ***************

#[derive(Properties, PartialEq)]
pub struct MessageProps {
    #[prop_or_default]
    pub class: Classes,

    pub message: AttrValue,

    #[prop_or_default]
    pub details: Option<AttrValue>,

    pub kind: MessageType,

    #[prop_or_default]
    pub onclick: Callback<()>,
}

#[function_component(Message)]
pub fn message(props: &MessageProps) -> Html {
    let show_details = use_state(|| false);

    let onclick = {
        let onclick = props.onclick.clone();

        Callback::from(move |_: MouseEvent| {
            onclick.emit(());
        })
    };

    let toggle_details = {
        let show_details = show_details.clone();

        Callback::from(move |e: MouseEvent| {
            e.stop_propagation();
            show_details.set(!*show_details);
        })
    };

    let kind_class = match props.kind {
        MessageType::Info => "message-info",
        MessageType::Success => "message-success",
        MessageType::Error => "message-error",
    };

    let class = classes!("thot-ui-message", kind_class, props.class.clone());
    let mut details_class = classes!("details");
    if *show_details {
        details_class.push("open");
    } else {
        details_class.push("closed");
    }

    let details_icon = if *show_details {
        IconId::FontAwesomeSolidAngleUp
    } else {
        IconId::FontAwesomeSolidAngleDown
    };

    html! {
        <div {class} {onclick}>
            <div class={classes!("message")}>
                { &props.message }
            </div>
            if {props.details.is_some()} {
                <div class={details_class}>
                    <span onclick={toggle_details}>
                        { "Details" }
                        <Icon icon_id={details_icon} />
                    </span>
                    if *show_details {
                        <div class={classes!("details-body")}>
                            { props.details.as_ref().unwrap() }
                        </div>
                    }
                </div>
            }
        </div>
    }
}

//! Shadow box.
use yew::prelude::*;
use yew_icons::{Icon, IconId};

#[derive(PartialEq, Properties)]
pub struct PageOverlayProps {
    /// Html id of the host element.
    pub host_id: AttrValue,

    #[prop_or_default]
    pub class: Classes,

    #[prop_or_default]
    pub children: Children,

    #[prop_or(Callback::noop())]
    pub onclose: Callback<MouseEvent>,
}

const CONTAINER_STYLE: &str = "
    position: absolute;
    top: 0;
    bottom: 0;
    left: 0;
    right: 0;
";

#[function_component(PageOverlay)]
pub fn page_overlay(props: &PageOverlayProps) -> Html {
    let window = web_sys::window().expect("could not get window");
    let document = window.document().expect("window should have a document");
    let host = document
        .get_element_by_id(&props.host_id)
        .expect("could not get host element");

    let out = html! {
        <div class={"syre-ui-page-overlay-wrapper"} style={CONTAINER_STYLE}>
            <div class={"syre-ui-page-overlay"} style={"position: relative;"}>
                <button onclick={props.onclose.clone()}>
                    <Icon icon_id={IconId::FontAwesomeSolidXmark} />
                </button>

                <div class={"page-overlay-content"}>
                    { for props.children.iter() }
                </div>
            </div>
        </div>
    };

    create_portal(out, host.into())
}

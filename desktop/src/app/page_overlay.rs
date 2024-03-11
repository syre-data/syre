//! Shadow box for the application.
use syre_ui::components::PageOverlay as PageOverlayUi;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct PageOverlayProps {
    #[prop_or_default]
    pub class: Classes,

    #[prop_or_default]
    pub children: Children,

    #[prop_or(Callback::noop())]
    pub onclose: Callback<MouseEvent>,
}

#[function_component(PageOverlay)]
pub fn shadow_box(props: &PageOverlayProps) -> Html {
    html! {
        <PageOverlayUi
            host_id={"app-main-page-overlay"}
            children={props.children.clone()}
            onclose={&props.onclose} />
    }
}

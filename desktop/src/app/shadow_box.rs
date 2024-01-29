//! Shadow box for the application.
use syre_ui::components::ShadowBox as ShadowBoxUi;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct ShadowBoxProps {
    #[prop_or_default]
    pub class: Classes,

    #[prop_or_default]
    pub children: Children,

    #[prop_or_default]
    pub title: Option<AttrValue>,

    #[prop_or(Callback::noop())]
    pub onclose: Callback<MouseEvent>,
}

#[function_component(ShadowBox)]
pub fn shadow_box(props: &ShadowBoxProps) -> Html {
    html! {
        <ShadowBoxUi
            host_id={"app-main-shadow-box"}
            children={props.children.clone()}
            title={&props.title}
            onclose={&props.onclose} />
    }
}

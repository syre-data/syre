//! Shadow box.
use yew::prelude::*;

#[derive(PartialEq, Properties)]
pub struct ShadowBoxProps {
    /// Html id of the host element.
    pub host_id: AttrValue,

    #[prop_or_default]
    pub class: Classes,

    #[prop_or_default]
    pub children: Children,

    #[prop_or_default]
    pub title: Option<AttrValue>,

    #[prop_or(Callback::noop())]
    pub onclose: Callback<MouseEvent>,
}

const CONTAINER_STYLE: &str = "
    position: fixed;
    top: 0;
    bottom: 0;
    left: 0;
    right: 0;

    display: flex;
    justify-content: center;
    align-items: center;
    background-color: rgba(0, 0, 0, 0.5);
";

const CLOSE_BTN_STYLES: &str = "
    position: absolute;
    top: 0;
    right: 0;

    padding: 0;
    margin: 0;
    width: 2rem;
    height: 2rem;
    border-radius: 1rem;
    transform: translate(50%, -50%);
";

#[function_component(ShadowBox)]
pub fn shadow_box(props: &ShadowBoxProps) -> Html {
    let window = web_sys::window().expect("could not get window");
    let document = window.document().expect("window should have a document");
    let host = document
        .get_element_by_id(&props.host_id)
        .expect("could not get host element");

    let out = html! {
        <div class={classes!("thot-ui-shadow-box-wrapper")} style={CONTAINER_STYLE}>
            <div class={classes!("thot-ui-shadow-box")} style={"position: relative;"}>
                <button
                    style={CLOSE_BTN_STYLES}
                    onclick={props.onclose.clone()}>{ "X" }</button>

                if props.title.is_some() {
                    <div class={classes!("shadow-box-header")}>
                        <h2>{ props.title.clone().unwrap() }</h2>
                    </div>
                }
                <div class={classes!("shadow-box-content")}>
                    { for props.children.iter() }
                </div>
            </div>
        </div>
    };

    create_portal(out, host.into())
}

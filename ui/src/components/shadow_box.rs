//! Shadow box.
use crate::hooks::preferred_theme::PreferredTheme;
use crate::hooks::use_preferred_theme;
use yew::prelude::*;

#[derive(PartialEq, Properties)]
pub struct ShadowBoxProps {
    #[prop_or_default]
    pub class: Classes,

    #[prop_or_default]
    pub children: Children,

    #[prop_or_default]
    pub title: Option<String>,

    #[prop_or(Callback::noop())]
    pub onclose: Callback<MouseEvent>,
}

const CONTAINER_STYLES_LIGHT: &str = "
    position: absolute;
    top: 0;
    bottom: 0;
    left: 0;
    right: 0;

    display: flex;
    justify-content: center;
    align-items: center;
    background-color: rgba(0, 0, 0, 0.5);
";

const CONTAINER_STYLES_DARK: &str = "
    position: absolute;
    top: 0;
    bottom: 0;
    left: 0;
    right: 0;

    display: flex;
    justify-content: center;
    align-items: center;
    background-color: rgba(0, 0, 0, 0.5);
";

const CONTENT_STYLES_LIGHT: &str = "
    position: relative;
";

const CONTENT_STYLES_DARK: &str = "
    position: relative;
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
    let theme = use_preferred_theme().unwrap_or(PreferredTheme::Dark);

    let container_styles = match theme {
        PreferredTheme::Light => CONTAINER_STYLES_LIGHT,
        PreferredTheme::Dark => CONTAINER_STYLES_DARK,
    };

    let content_styles = match theme {
        PreferredTheme::Light => CONTENT_STYLES_LIGHT,
        PreferredTheme::Dark => CONTENT_STYLES_DARK,
    };

    html! {
        <div class={classes!("thot-ui-shadow-box-wrapper")}style={container_styles}>
            <div class={classes!("thot-ui-shadow-box")} style={content_styles}>
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
    }
}

#[cfg(test)]
#[path = "./shadow_box_test.rs"]
mod shadow_box_test;

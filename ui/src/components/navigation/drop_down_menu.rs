//! Drop-down menu item.
//! Toggles its children's visibility on click.
use yew::prelude::*;

/// [`DropDownMenu`] properties.
#[derive(PartialEq, Properties)]
pub struct DropdownMenuProps {
    #[prop_or_default]
    pub children: Children,

    #[prop_or_default]
    pub icon: Option<String>,

    pub title: String,

    /// Initial state of the menu.
    #[prop_or(true)]
    pub collapsed: bool,
}

/// Drop-down menu.
/// Each child appears as an element.
#[function_component(DropdownMenu)]
pub fn drop_down_menu(props: &DropdownMenuProps) -> Html {
    let collapsed = use_state(|| props.collapsed);
    let toggle_collapse = {
        let collapsed = collapsed.clone();

        Callback::from(move |_e: web_sys::MouseEvent| {
            collapsed.set(!*collapsed);
        })
    };

    html! {
        <div>
            <div class={classes!("drop-down-menu-title", "clickable")}
                onclick={toggle_collapse}>

                if let Some(icon) = props.icon.clone() {
                    <span class={classes!("title-icon")}>{
                        icon
                    }</span>
                }

                <span class={classes!("title")}>{
                    &props.title
                }</span>

                <span class={classes!("collapsed-indicator")}>
                    if *collapsed {
                        // { "v" }
                        { "\u{02c5}" }
                    } else {
                        // { "^" }
                        { "\u{02c4}" }
                    }
                </span>
            </div>

            if !*collapsed {
                <ul>
                    {props.children.iter().map(|child| html! {
                        <li>{ child }</li>
                    }).collect::<Html>()}
                </ul>
            }
        </div>
    }
}

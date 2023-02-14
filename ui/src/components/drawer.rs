//! Drawer component.
use yew::prelude::*;

#[derive(PartialEq)]
pub enum DrawerPosition {
    Top,
    Right,
    Bottom,
    Left,
}

#[derive(Properties, PartialEq)]
pub struct DrawerProps {
    #[prop_or_default]
    pub class: Classes,

    #[prop_or_default]
    pub children: Children,

    // @todo: Allow this to be optional, creating a
    // state for internal management if not provided.
    /// Toggle the open state of the [`Drawer`].
    pub open: UseStateHandle<bool>,

    /// Location of the [`Drawer`] on screen.
    pub position: DrawerPosition,
}

#[function_component(Drawer)]
pub fn drawer(props: &DrawerProps) -> Html {
    let toggle_open = {
        let open = props.open.clone();

        Callback::from(move |_: MouseEvent| {
            open.set(!*open);
        })
    };

    let (open_symbol, close_symbol) = match props.position {
        DrawerPosition::Top => ('\u{25BE}', '\u{25B4}'),
        DrawerPosition::Right => ('\u{25B8}', '\u{25C2}'),
        DrawerPosition::Bottom => ('\u{25B4}', '\u{25BE}'),
        DrawerPosition::Left => ('\u{25C2}', '\u{25B8}'),
    };

    let class = classes!(
        "thot-ui-drawer",
        (*props.open).then(|| "open"),
        props.class.clone()
    );

    let style = r"
        display: flex;
    ";

    let contents_style = if !*props.open { "display: none;" } else { "" };

    html! {
        <div {class} {style}>
            <div class={classes!("drawer-toggle")}
                onclick={toggle_open}>

                { if *props.open {
                    { open_symbol }
                } else {
                    { close_symbol }
                }}
            </div>
            <div class={classes!("drawer-contents")}
                style={contents_style} >

                { for props.children.iter() }
            </div>
        </div>
    }
}

#[cfg(test)]
#[path = "./drawer_test.rs"]
mod drawer_test;

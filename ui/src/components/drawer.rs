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
    let class = classes!(
        "thot-ui-drawer",
        (*props.open).then(|| "open"),
        props.class.clone()
    );

    let contents_style = if !*props.open { "display: none;" } else { "" };

    html! {
        <div {class}>
            <div class={classes!("drawer-contents")}
                style={contents_style} >

                { for props.children.iter() }
            </div>
        </div>
    }
}

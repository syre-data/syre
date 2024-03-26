//! Drawer component.
use wasm_bindgen::prelude::Closure;
use wasm_bindgen::JsCast;
use yew::prelude::*;

#[derive(PartialEq, Clone)]
pub enum ResizeHandle {
    None,
    Top,
    Right,
    Bottom,
    Left,
}

impl Default for ResizeHandle {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Properties, PartialEq)]
pub struct DrawerProps {
    #[prop_or_default]
    pub root_ref: NodeRef,

    #[prop_or_default]
    pub class: Classes,

    #[prop_or_default]
    pub children: Children,

    /// Toggle the open state of the [`Drawer`].
    pub open: bool,

    /// Location of the [`Drawer`] on screen for resizing.
    #[prop_or_default]
    pub resize: ResizeHandle,
}

#[function_component(Drawer)]
pub fn drawer(props: &DrawerProps) -> Html {
    let root_ref = props.root_ref.clone();
    let resizer_ref = use_node_ref();

    use_effect_with(props.resize.clone(), {
        let root_ref = root_ref.clone();
        let resizer_ref = resizer_ref.clone();
        move |resize| {
            if resize == &ResizeHandle::None {
                return;
            }

            let document = web_sys::window().unwrap().document().unwrap();
            let root_elm = root_ref
                .cast::<web_sys::HtmlDivElement>()
                .expect("could not cast root node to div element");

            let resizer_elm = resizer_ref
                .cast::<web_sys::HtmlDivElement>()
                .expect("could not cast resizer node to div element");

            let trigger_resize = Closure::<dyn Fn(MouseEvent)>::new({
                let resize = resize.clone();
                // let remove_resize = remove_resize.as_ref().unchecked_ref();
                move |e: MouseEvent| {
                    e.stop_propagation();

                    let resize_cb = Closure::<dyn Fn(MouseEvent)>::new({
                        let resize = resize.clone();
                        let root_elm = root_elm.clone();
                        move |e: MouseEvent| match resize {
                            ResizeHandle::None => panic!("resize should not be called"),

                            ResizeHandle::Top => {
                                let root_bb = root_elm.get_bounding_client_rect();
                                let root_base = root_bb.y() + root_bb.height();
                                let height = e.y() - root_base as i32;
                                root_elm
                                    .style()
                                    .set_property("height", &format!("{height}px"))
                                    .unwrap();
                            }

                            ResizeHandle::Right => {
                                let root_bb = root_elm.get_bounding_client_rect();
                                let root_base = root_bb.x();
                                let width = e.x() - root_base as i32;
                                root_elm
                                    .style()
                                    .set_property("width", &format!("{width}px"))
                                    .unwrap();
                            }

                            ResizeHandle::Bottom => {
                                let root_bb = root_elm.get_bounding_client_rect();
                                let root_base = root_bb.y() as i32;
                                let height = root_base as i32 - e.y();
                                root_elm
                                    .style()
                                    .set_property("height", &format!("{height}px"))
                                    .unwrap();
                            }

                            ResizeHandle::Left => {
                                let root_bb = root_elm.get_bounding_client_rect();
                                let root_base = root_bb.x() + root_bb.width();
                                let width = root_base as i32 - e.x();
                                root_elm
                                    .style()
                                    .set_property("width", &format!("{width}px"))
                                    .unwrap();
                            }
                        }
                    });

                    document
                        .add_event_listener_with_callback(
                            "mousemove",
                            resize_cb.as_ref().unchecked_ref(),
                        )
                        .unwrap();

                    let remove_resize = Closure::<dyn Fn(MouseEvent)>::new({
                        let document = document.clone();
                        move |_e: MouseEvent| {
                            document
                                .remove_event_listener_with_callback(
                                    "mousemove",
                                    resize_cb.as_ref().unchecked_ref(),
                                )
                                .unwrap();
                        }
                    });

                    document
                        .add_event_listener_with_callback(
                            "mouseup",
                            remove_resize.as_ref().unchecked_ref(),
                        )
                        .unwrap();

                    remove_resize.forget();
                }
            });

            resizer_elm.set_onmousedown(Some(trigger_resize.as_ref().unchecked_ref()));
            trigger_resize.forget();
        }
    });

    let class = classes!(
        "syre-ui-drawer",
        props.open.then(|| "open"),
        props.class.clone()
    );

    let mut style = vec![];
    let mut resizer_style = vec!["flex-shrink: 0;"];
    let mut resizer_class = classes!("resize-handle");
    let mut contents_style = vec![];
    match &props.resize {
        ResizeHandle::None => {}

        ResizeHandle::Bottom => {
            style.push("display: flex;".to_string());
            resizer_style.push("align-self: flex-end;");
            resizer_class.push("vertical top");
            contents_style.push("flex-grow: 1;")
        }

        ResizeHandle::Left => {
            style.push("display: flex;".to_string());
            resizer_style.push("align-self: flex-start;");
            resizer_class.push("horizontal right");
            contents_style.push("flex-grow: 1;")
        }

        ResizeHandle::Top => {
            style.push("display: flex;".to_string());
            resizer_style.push("align-self: flex-start;");
            resizer_class.push("vertical bottom");
            contents_style.push("flex-grow: 1;")
        }

        ResizeHandle::Right => {
            style.push("display: flex;".to_string());
            resizer_style.push("align-self: flex-end;");
            resizer_class.push("horizontal left");
            contents_style.push("flex-grow: 1;")
        }
    }

    if !props.open {
        style.push("display: none;".to_string());
    };

    if let Some(root_elm) = root_ref.cast::<web_sys::HtmlDivElement>() {
        if let Ok(width) = root_elm.style().get_property_value("width") {
            if !width.trim().is_empty() {
                style.push(format!("width: {width};"));
            }
        }

        if let Ok(height) = root_elm.style().get_property_value("height") {
            if !height.trim().is_empty() {
                style.push(format!("height: {height};"));
            }
        }
    }

    let resize_handle = html! {
        <div ref={resizer_ref}
            class={resizer_class}
            style={resizer_style.join(" ")}>
        </div>
    };

    html! {
        <div ref={root_ref}
            {class}
            style={style.join(" ")}>

            if &props.resize == &ResizeHandle::Left || &props.resize == &ResizeHandle::Top {
                { resize_handle.clone() }
            }

            <div class={"drawer-contents"}
                style={contents_style.join(" ")} >

                { for props.children.iter() }
            </div>

            if &props.resize == &ResizeHandle::Right || &props.resize == &ResizeHandle::Bottom {
                { resize_handle }
            }
        </div>
    }
}

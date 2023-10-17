//! Container tree UI.
use super::Container as ContainerUi;
use crate::app::{AppStateAction, AppStateReducer, AuthStateReducer, ShadowBox};
use crate::commands::container::{NewChildArgs, UpdatePropertiesArgs};
use crate::common::invoke;
use crate::components::canvas::{
    CanvasStateAction, CanvasStateReducer, GraphStateAction, GraphStateReducer,
};
use std::str::FromStr;
use thot_core::project::Container;
use thot_core::types::{Creator, ResourceId, UserId};
use thot_ui::types::Message;
use thot_ui::widgets::suspense::Loading;
use wasm_bindgen::prelude::Closure;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yew_icons::{Icon, IconId};

// **********************
// *** New Child Name ***
// **********************

/// Properties for a [`NewChildName`].
#[derive(Properties, PartialEq)]
struct NewChildNameProps {
    /// Callback to run on submission.
    #[prop_or_default]
    pub onsubmit: Option<Callback<String>>,
}

/// Component to get name for a new child.
#[tracing::instrument(level = "debug", skip(props))]
#[function_component(NewChildName)]
fn new_child_name(props: &NewChildNameProps) -> Html {
    let input_ref = use_node_ref();
    let is_input_valid = use_state(|| false);

    let onsubmit = {
        let cb = props.onsubmit.clone();
        let input_ref = input_ref.clone();

        Callback::from(move |e: web_sys::SubmitEvent| {
            e.prevent_default();

            if let Some(cb) = cb.clone() {
                let input = input_ref
                    .cast::<web_sys::HtmlInputElement>()
                    .expect("could not cast node ref as input");

                cb.emit(input.value().trim().into());
            }
        })
    };

    let oninput = {
        let input_ref = input_ref.clone();
        let is_input_valid = is_input_valid.clone();

        Callback::from(move |_: web_sys::InputEvent| {
            let input = input_ref
                .cast::<web_sys::HtmlInputElement>()
                .expect("could not cast node ref as input");

            is_input_valid.set(!input.value().trim().is_empty());
        })
    };

    html! {
        <form {onsubmit}>
             <input ref={input_ref} {oninput} placeholder="Name" />
             <button disabled={!*is_input_valid}>{ "Add child" }</button>
        </form>
    }
}

// **********************
// *** Container Tree ***
// **********************

static CONNECTOR_CLASS: &str = "container-tree-node-connector";
static VISIBILITY_CONTROL_CLASS: &str = "container-tree-visibility-control";
static VISIBILITY_CONTROL_SIZE: u8 = 20;
static EYE_ICON_SIZE: u8 = 16;

/// Properties for a [`ContainerTree`].
#[derive(Properties, PartialEq, Debug)]
pub struct ContainerTreeProps {
    /// The root `Container`.
    pub root: ResourceId,
}

/// Container tree component.
#[tracing::instrument(level = "debug")]
#[function_component(ContainerTree)]
pub fn container_tree(props: &ContainerTreeProps) -> HtmlResult {
    let auth_state =
        use_context::<AuthStateReducer>().expect("`AuthStateReducer` context not found");

    let app_state = use_context::<AppStateReducer>().expect("`AppStateReducer` context not found");
    let canvas_state =
        use_context::<CanvasStateReducer>().expect("`CanvasStateReducer` context not found");

    let graph_state = use_context::<GraphStateReducer>().expect("`GraphReducer` context not found");

    let show_add_child_form = use_state(|| false);
    let new_child_parent = use_state(|| None);
    let root_ref = use_node_ref();
    let children_ref = use_node_ref();
    let connectors_ref = use_node_ref();

    // -----------------
    // --- add child ---
    // -----------------

    let onadd_child = {
        let show_add_child_form = show_add_child_form.clone();
        let new_child_parent = new_child_parent.clone();

        Callback::from(move |parent: ResourceId| {
            new_child_parent.set(Some(parent));
            show_add_child_form.set(true);
        })
    };

    let close_add_child = {
        let show_add_child_form = show_add_child_form.clone();

        Callback::from(move |_: web_sys::MouseEvent| {
            show_add_child_form.set(false);
        })
    };

    let add_child = {
        let app_state = app_state.clone();
        let graph_state = graph_state.clone();
        let new_child_parent = new_child_parent.clone();
        let show_add_child_form = show_add_child_form.clone();
        let uid = auth_state
            .user
            .as_ref()
            .expect("`AuthState.user` should be set")
            .rid
            .clone();

        Callback::from(move |name: String| {
            let app_state = app_state.clone();
            let graph_state = graph_state.clone();
            let uid = uid.clone();

            show_add_child_form.set(false);
            let parent = (*new_child_parent)
                .clone()
                .expect("new child parent not set");

            spawn_local(async move {
                // create child
                let Ok(mut child) = invoke::<Container>(
                    "new_child",
                    NewChildArgs {
                        name,
                        parent: parent.clone(),
                    },
                )
                .await
                else {
                    app_state.dispatch(AppStateAction::AddMessage(Message::error(
                        "Could not create child",
                    )));
                    return;
                };

                // REMOVE Taken care of by file listener now.
                // graph_state.dispatch(GraphStateAction::InsertChildContainer(
                //     parent,
                //     child.clone(),
                // ));

                // // set creator
                // child.properties.creator = Creator::User(Some(UserId::Id(uid)));

                // graph_state.dispatch(GraphStateAction::UpdateContainerProperties(
                //     UpdatePropertiesArgs {
                //         rid: child.rid,
                //         properties: child.properties,
                //     },
                // ));
            });
        })
    };

    // ----------
    // --- ui ---
    // ----------

    // add connectors
    {
        let canvas_state = canvas_state.clone();
        let root_ref = root_ref.clone();
        let children_ref = children_ref.clone();
        let connectors_ref = connectors_ref.clone();

        use_effect(move || {
            create_connectors(
                root_ref.clone(),
                children_ref.clone(),
                connectors_ref.clone(),
                canvas_state.clone(),
            );
        });
    }
    {
        let canvas_state = canvas_state.clone();
        let root_ref = root_ref.clone();
        let children_ref = children_ref.clone();
        let connectors_ref = connectors_ref.clone();

        use_effect_with((), move |_| {
            let window = web_sys::window().expect("could not get window");
            let create_connectors_cb = Closure::<dyn Fn()>::new(move || {
                create_connectors(
                    root_ref.clone(),
                    children_ref.clone(),
                    connectors_ref.clone(),
                    canvas_state.clone(),
                )
            });

            window
                .add_event_listener_with_callback(
                    "resize",
                    create_connectors_cb.as_ref().unchecked_ref(),
                )
                .expect("could not add `resize` listener to `window`");

            create_connectors_cb.forget();
        });
    }

    let container_fallback = html! { <Loading text={"Loading container"} /> };
    Ok(html! {
        <div class={classes!("container-tree")}>
            <Suspense fallback={container_fallback}>
                <svg ref={connectors_ref}
                    class="container-tree-node-connectors">

                    <defs>
                        <g id={"visible-marker"}>
                            <Icon icon_id={IconId::FontAwesomeRegularEye}
                                width={format!("{EYE_ICON_SIZE}")}
                                height={format!("{EYE_ICON_SIZE}")} />
                        </g>

                        <g id={"hidden-marker"}>
                            <Icon icon_id={IconId::FontAwesomeRegularEyeSlash}
                                width={format!("{EYE_ICON_SIZE}")}
                                height={format!("{EYE_ICON_SIZE}")} />
                        </g>
                    </defs>
                </svg>

                <ContainerUi
                    r#ref={root_ref}
                    rid={props.root.clone()}
                    {onadd_child} />

                <div ref={children_ref} class={classes!("children")}>
                    { graph_state.graph
                        .children(&props.root)
                        .expect("`Container` children not found")
                        .iter()
                        .map(|rid| html! {
                            if canvas_state.is_visible(rid) {
                                <ContainerTree root={rid.clone()} />
                            } else {
                                <div class={classes!("child-node-marker")}
                                    data-rid={rid.clone()}>

                                    { &graph_state.graph.get(&rid)
                                        .expect("child `Container` not found")
                                        .properties
                                        .name
                                    }
                                </div>
                            }
                        })
                        .collect::<Html>()
                    }
                </div>
            </Suspense>

            if *show_add_child_form {
                <ShadowBox title="Add child" onclose={close_add_child}>
                    <NewChildName onsubmit={add_child} />
                </ShadowBox>
            }
        </div>
    })
}

// ***************
// *** helpers ***
// ***************

/// Creates connectors between `Container` nodes in a tree.
fn create_connectors(
    root: NodeRef,
    children: NodeRef,
    connectors: NodeRef,
    canvas_state: CanvasStateReducer,
) {
    if root.get().is_none() || children.get().is_none() || connectors.get().is_none() {
        // element not loaded
        return;
    }

    let root_elm = root
        .cast::<web_sys::HtmlElement>()
        .expect("could not cast root node to element");

    let children_elm = children
        .cast::<web_sys::HtmlElement>()
        .expect("could cast children node to element");

    let connectors_elm = connectors
        .cast::<web_sys::HtmlElement>()
        .expect("could not cast connectors node to element");

    let children = children_elm
        .query_selector_all(":scope > .container-tree")
        .expect("could not query children");

    let children_markers = children_elm
        .query_selector_all(":scope > .child-node-marker")
        .expect("could not query children markers");

    // clear connectors
    let current_elements = connectors_elm
        .query_selector_all(":scope > :not(defs)")
        .expect("could not query current elements");

    for index in 0..current_elements.length() {
        let Some(child_elm) = current_elements.get(index) else {
            continue;
        };

        let _ = connectors_elm.remove_child(&child_elm);
    }

    // root position
    let root_bottom = root_elm.offset_top() + root_elm.client_height();
    let root_center = root_elm.offset_left() + root_elm.client_width() / 2;

    // add connectors to children
    for index in 0..children.length() {
        let Some(child_elm) = children.get(index) else {
            continue;
        };

        let child_elm = child_elm
            .dyn_ref::<web_sys::HtmlElement>()
            .expect("could not cast child node to element");

        let child_node_elm = child_elm
            .query_selector(":scope > .container-node")
            .expect("could not get child node")
            .expect("child node not found");

        let child_node_elm = child_node_elm
            .dyn_ref::<web_sys::HtmlElement>()
            .expect("could not cast child node to element");

        let child_top = child_elm.offset_top() + child_node_elm.offset_top();
        let child_center = child_elm.offset_left()
            + child_node_elm.offset_left()
            + child_node_elm.client_width() / 2;

        let midgap = (root_bottom + child_top) / 2;
        let points_list = format!("{root_center},{root_bottom} {root_center},{midgap} {child_center},{midgap} {child_center},{child_top}");

        let connector = create_connector_element(&points_list);
        connectors_elm
            .append_child(&connector)
            .expect("could not add connector to document");

        // visibility toggle
        let child_id = child_node_elm
            .dataset()
            .get("rid")
            .expect("could not get `ResourceId` of child element");

        let visibility =
            create_visibility_control_element(child_center, midgap, child_id, canvas_state.clone());

        connectors_elm
            .append_child(&visibility)
            .expect("could not add visibility control to document");
    }

    // add connectors to children markers
    for index in 0..children_markers.length() {
        let Some(child_elm) = children_markers.get(index) else {
            continue;
        };

        let child_elm = child_elm
            .dyn_ref::<web_sys::HtmlElement>()
            .expect("could not cast child node to element");

        let child_top = child_elm.offset_top();
        let child_center = child_elm.offset_left() + child_elm.client_width() / 2;

        let midgap = (root_bottom + child_top) / 2;
        let points_list = format!("{root_center},{root_bottom} {root_center},{midgap} {child_center},{midgap} {child_center},{child_top}");

        let connector = create_connector_element(&points_list);
        connectors_elm
            .append_child(&connector)
            .expect("could not add connector to document");

        // visibility toggle
        let child_id = child_elm
            .dataset()
            .get("rid")
            .expect("could not get `ResourceId` of child marker element");

        let visibility =
            create_visibility_control_element(child_center, midgap, child_id, canvas_state.clone());

        connectors_elm
            .append_child(&visibility)
            .expect("could not add visibility control to document");
    }
}

fn create_connector_element(points: &str) -> web_sys::SvgPolylineElement {
    let window = web_sys::window().expect("could not get window");
    let document = window.document().expect("window should have a document");

    let connector = document
        .create_element_ns(Some("http://www.w3.org/2000/svg"), "polyline")
        .expect("could not create polyline");

    let connector = connector
        .dyn_ref::<web_sys::SvgPolylineElement>()
        .expect("could not cast element as polyline");

    connector
        .set_attribute("points", &points)
        .expect("could not set `points` on connector");

    connector
        .set_attribute("class", CONNECTOR_CLASS)
        .expect("could not set `class` on connector");

    connector.to_owned()
}

fn create_visibility_control_element(
    cx: i32,
    cy: i32,
    rid: String,
    canvas_state: CanvasStateReducer,
) -> web_sys::SvgsvgElement {
    let window = web_sys::window().expect("could not get window");
    let document = window.document().expect("window should have a document");

    let vis_circle = document
        .create_element_ns(Some("http://www.w3.org/2000/svg"), "circle")
        .expect("could not create `circle`");

    let vis_circle = vis_circle
        .dyn_ref::<web_sys::SvgCircleElement>()
        .expect("could not cast element as `circle`");

    vis_circle
        .set_attribute("cx", &format!("{}", VISIBILITY_CONTROL_SIZE / 2))
        .expect("could not set `cx` on visibility control");

    vis_circle
        .set_attribute("cy", &format!("{}", VISIBILITY_CONTROL_SIZE / 2))
        .expect("could not set `cy` on visibility control");

    vis_circle
        .set_attribute("r", &format!("{}", VISIBILITY_CONTROL_SIZE / 2))
        .expect("could not set `r` on visibility control");

    let child_id = ResourceId::from_str(&rid).expect("could not parse child's `ResourceId`");
    let eye_icon = document
        .create_element_ns(Some("http://www.w3.org/2000/svg"), "use")
        .expect("could not create `use`");

    let eye_icon = eye_icon
        .dyn_ref::<web_sys::SvgUseElement>()
        .expect("could not cast element as `use`");

    let icon_id = if canvas_state.is_visible(&child_id) {
        "#visible-marker"
    } else {
        "#hidden-marker"
    };

    eye_icon
        .set_attribute("href", icon_id)
        .expect("could not set `href` on visiblilty icon");

    eye_icon
        .set_attribute(
            "x",
            &format!("{}", (VISIBILITY_CONTROL_SIZE - EYE_ICON_SIZE) / 2),
        )
        .expect("could not set `x` on visibility icon");

    eye_icon
        .set_attribute(
            "y",
            &format!("{}", (VISIBILITY_CONTROL_SIZE - EYE_ICON_SIZE) / 2),
        )
        .expect("could not set `y` on visibility icon");

    // eye_icon
    //     .set_attribute("width", "100%")
    //     .expect("could not set `width` on visibility icon");

    // eye_icon
    //     .set_attribute("height", "100%")
    //     .expect("could not set `height` on visibility icon");

    let visibility = document
        .create_element_ns(Some("http://www.w3.org/2000/svg"), "svg")
        .expect("could not create `svg`");

    let visibility = visibility
        .dyn_ref::<web_sys::SvgsvgElement>()
        .expect("could not cast element as `svg`");

    visibility
        .set_attribute(
            "viewBox",
            &format!("0 0 {VISIBILITY_CONTROL_SIZE} {VISIBILITY_CONTROL_SIZE}"),
        )
        .expect("could not set `x` on visibility control");

    visibility
        .set_attribute("width", &format!("{VISIBILITY_CONTROL_SIZE}"))
        .expect("could not set `width` on visibility control");

    visibility
        .set_attribute("height", &format!("{VISIBILITY_CONTROL_SIZE}"))
        .expect("could not set `height` on visibility control");

    visibility
        .set_attribute("class", VISIBILITY_CONTROL_CLASS)
        .expect("could not set `class` on visibility control");

    visibility
        .set_attribute(
            "x",
            &format!("{}", cx - (VISIBILITY_CONTROL_SIZE / 2) as i32),
        )
        .expect("could not set `x` on visibility control");

    visibility
        .set_attribute(
            "y",
            &format!("{}", cy - (VISIBILITY_CONTROL_SIZE / 2) as i32),
        )
        .expect("could not set `y` on visibility control");

    visibility
        .append_child(vis_circle)
        .expect("could not append `circle` as child");

    visibility
        .append_child(&eye_icon)
        .expect("could not append icon as child");

    let toggle_visibility = Closure::<dyn Fn(MouseEvent)>::new(move |e: MouseEvent| {
        e.stop_propagation();

        canvas_state.dispatch(CanvasStateAction::SetVisibility(
            child_id.clone(),
            !canvas_state.is_visible(&child_id),
        ));
    });

    visibility.set_onclick(Some(toggle_visibility.as_ref().unchecked_ref()));
    toggle_visibility.forget();

    visibility.to_owned()
}

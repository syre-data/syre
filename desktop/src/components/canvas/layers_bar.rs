//! Layers.
use crate::components::canvas::selection_action::{selection_action, SelectionAction};
use crate::components::canvas::{CanvasStateAction, CanvasStateReducer, GraphStateReducer};
use thot_core::types::ResourceId;
use yew::prelude::*;
use yew_icons::{Icon, IconId};

const EXPAND_ICON_SIZE: u8 = 16;

/// Properties for a [`Layer`].
#[derive(Properties, PartialEq)]
struct LayerProps {
    pub root: ResourceId,

    /// Initial expansion state.
    #[prop_or(false)]
    pub expanded: bool,
}

#[function_component(Layer)]
fn layer(props: &LayerProps) -> Html {
    let canvas_state = use_context::<CanvasStateReducer>().unwrap();
    let graph_state = use_context::<GraphStateReducer>().unwrap();
    let expanded_state = use_state(|| props.expanded);

    let root = graph_state.graph.get(&props.root).unwrap();
    let children = graph_state.graph.children(&props.root).unwrap();
    let selected = canvas_state.selected.contains(&props.root);
    let multiple_selected = canvas_state.selected.len() > 1;

    let toggle_expanded_state = {
        let expanded_state = expanded_state.clone();
        Callback::from(move |e: MouseEvent| {
            e.stop_propagation();
            expanded_state.set(!*expanded_state);
        })
    };

    let onclick_layer = {
        let canvas_state = canvas_state.clone();
        let root = props.root.clone();
        Callback::from(move |e: MouseEvent| {
            e.prevent_default();
            e.stop_propagation();
            let root = root.clone();

            match selection_action(selected, multiple_selected, e) {
                SelectionAction::SelectOnly => {
                    canvas_state.dispatch(CanvasStateAction::ClearSelected);
                    canvas_state.dispatch(CanvasStateAction::SelectContainer(root));
                }

                SelectionAction::Select => {
                    canvas_state.dispatch(CanvasStateAction::SelectContainer(root));
                }

                SelectionAction::Unselect => {
                    canvas_state.dispatch(CanvasStateAction::Unselect(root));
                }
            }
        })
    };

    // TODO Double click layer to center container.
    // TODO Assets?

    let mut classes = classes!("layer");
    if *expanded_state {
        classes.push("expanded")
    }
    if selected {
        classes.push("selected");
    }

    html! {
        <div class={classes}>
            <div class={classes!("layer-title")}
                onclick={onclick_layer}>

                if children.len() > 0 {
                    <span class={classes!("layer-expand")}
                        onclick={toggle_expanded_state}>
                        if *expanded_state {
                            <Icon icon_id={IconId::FontAwesomeSolidCaretDown}
                                width={format!("{EXPAND_ICON_SIZE}")}
                                height={format!("{EXPAND_ICON_SIZE}")} />
                        } else {
                            <Icon icon_id={IconId::FontAwesomeSolidCaretRight}
                                width={format!("{EXPAND_ICON_SIZE}")}
                                height={format!("{EXPAND_ICON_SIZE}")} />
                        }
                    </span>
                }
                <span class="name">{ &root.properties.name }</span>
            </div>
            if children.len() > 0 {
                <div class={classes!("children")}>
                    { children.iter().map(|child| html!{
                        <Layer key={format!("layer-{child}")}
                            root={child.clone()} />
                    }).collect::<Html>() }
                </div>
            }
        </div>
    }
}

#[function_component(LayersBar)]
pub fn layers_bar() -> Html {
    let graph_state = use_context::<GraphStateReducer>().unwrap();
    let root = graph_state.graph.root();
    html! {
        <div class={classes!("layers-bar")}>
            <Layer root={root.clone()} expanded={true} />
        </div>
    }
}

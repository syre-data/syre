//! Layers.
use crate::components::canvas::selection_action::{selection_action, SelectionAction};
use crate::components::canvas::{CanvasStateAction, CanvasStateReducer, GraphStateReducer};
use thot_core::project::Asset as CoreAsset;
use thot_core::types::ResourceId;
use thot_ui::widgets::common::asset as asset_ui;
use yew::prelude::*;
use yew_icons::{Icon, IconId};

const EXPAND_ICON_SIZE: u8 = 16;
const RESOURCE_ICON_SIZE: u8 = 16;

/// Properties for [`Assets`].
#[derive(Properties, PartialEq)]
struct AssetProps {
    pub asset: CoreAsset,
}

#[function_component(Asset)]
fn asset(props: &AssetProps) -> Html {
    let canvas_state = use_context::<CanvasStateReducer>().unwrap();

    let selected = canvas_state.selected.contains(&props.asset.rid);
    let multiple_selected = canvas_state.selected.len() > 1;
    let onclick = {
        let canvas_state = canvas_state.clone();
        let asset = props.asset.rid.clone();
        let selected = selected.clone();
        Callback::from(move |e: MouseEvent| {
            e.prevent_default();
            e.stop_propagation();
            let asset = asset.clone();

            match selection_action(selected, multiple_selected, e) {
                SelectionAction::SelectOnly => {
                    canvas_state.dispatch(CanvasStateAction::SelectAssetOnly(asset));
                }

                SelectionAction::Select => {
                    canvas_state.dispatch(CanvasStateAction::SelectAsset(asset));
                }

                SelectionAction::Unselect => {
                    canvas_state.dispatch(CanvasStateAction::Unselect(asset));
                }
            }
        })
    };

    let mut class = classes!("layer-asset");
    if selected {
        class.push("selected");
    }

    html! {
        <div {class} {onclick}>

            <span class={classes!("resource-icon")} style={ asset_ui::asset_icon_color(&props.asset) }>
                <Icon icon_id={asset_ui::asset_icon_id(&props.asset)}
                    width={RESOURCE_ICON_SIZE.to_string()}
                    height={RESOURCE_ICON_SIZE.to_string()} />
            </span>

            <span class={classes!("name")}>
                { asset_ui::asset_display_name(&props.asset) }
            </span>
        </div>
    }
}

/// Properties for [`Assets`].
#[derive(Properties, PartialEq)]
struct AssetsProps {
    pub assets: Vec<CoreAsset>,

    /// Initial expansion state.
    #[prop_or(false)]
    pub expanded: bool,
}

#[function_component(Assets)]
fn assets(props: &AssetsProps) -> Html {
    let expanded_state = use_state(|| props.expanded);

    let toggle_expanded_state = {
        let expanded_state = expanded_state.clone();
        Callback::from(move |e: MouseEvent| {
            e.stop_propagation();
            expanded_state.set(!*expanded_state);
        })
    };

    let mut class = classes!("layer", "assets");
    if *expanded_state {
        class.push("expanded")
    }

    html! {
        <div {class}>
            <div class={classes!("layer-title")}>
                <span class={classes!("layer-expand")}
                    onclick={toggle_expanded_state}>
                    if *expanded_state {
                        <Icon icon_id={IconId::FontAwesomeSolidCaretDown}
                            width={EXPAND_ICON_SIZE.to_string()}
                            height={EXPAND_ICON_SIZE.to_string()} />
                    } else {
                        <Icon icon_id={IconId::FontAwesomeSolidCaretRight}
                            width={EXPAND_ICON_SIZE.to_string()}
                            height={EXPAND_ICON_SIZE.to_string()} />
                    }
                </span>

                <span class={classes!("resource-icon")}>
                    <Icon icon_id={IconId::BootstrapFiles}
                        width={RESOURCE_ICON_SIZE.to_string()}
                        height={RESOURCE_ICON_SIZE.to_string()} />
                </span>

                <span class={classes!("name")}>{ "Assets" }</span>
            </div>

            <div class={classes!("layer-assets")}>
                { props.assets.iter().map(|asset| html! {
                    <Asset asset={asset.clone()} />
                }).collect::<Html>()}
            </div>
        </div>
    }
}

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

    let onclick = {
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

    let ondblclick = {
        let root = props.root.clone();
        let project = canvas_state.project.clone();
        Callback::from(move |e: MouseEvent| {
            e.stop_propagation();

            let window = web_sys::window().unwrap();
            let document = window.document().unwrap();
            let container = document
                .query_selector(&format!(
                    ".project-canvas[data-rid=\"{project}\"] .container-node[data-rid=\"{root}\"]"
                ))
                .unwrap()
                .unwrap();

            let mut scroll_opts = web_sys::ScrollIntoViewOptions::new();
            scroll_opts.behavior(web_sys::ScrollBehavior::Smooth);
            scroll_opts.block(web_sys::ScrollLogicalPosition::Center);
            scroll_opts.inline(web_sys::ScrollLogicalPosition::Center);
            container.scroll_into_view_with_scroll_into_view_options(&scroll_opts);
        })
    };

    let mut class = classes!("layer");
    if *expanded_state {
        class.push("expanded")
    }
    if selected {
        class.push("selected");
    }

    html! {
        <div {class}>
            <div class={classes!("layer-title")}
                {onclick}
                {ondblclick} >

                if children.len() > 0 {
                    <span class={classes!("layer-expand")}
                        onclick={toggle_expanded_state}>
                        if *expanded_state {
                            <Icon icon_id={IconId::FontAwesomeSolidCaretDown}
                                width={EXPAND_ICON_SIZE.to_string()}
                                height={EXPAND_ICON_SIZE.to_string()} />
                        } else {
                            <Icon icon_id={IconId::FontAwesomeSolidCaretRight}
                                width={EXPAND_ICON_SIZE.to_string()}
                                height={EXPAND_ICON_SIZE.to_string()} />
                        }
                    </span>
                }
                <span class={classes!("resource-icon")}>
                    <Icon icon_id={IconId::FontAwesomeRegularFolder}
                        width={RESOURCE_ICON_SIZE.to_string()}
                        height={RESOURCE_ICON_SIZE.to_string()} />
                </span>
                <span class={classes!("name")}>{ &root.properties.name }</span>
            </div>
            <div class={classes!("resources")}>
                if root.assets.len() > 0 {
                    <Assets assets={root.assets.values().map(|asset| asset.clone()).collect::<Vec<_>>()} />
                }

                if children.len() > 0 {
                    <div class={classes!("children")}>
                        { children.iter().map(|child| html!{
                            <Layer key={format!("layer-{child}")}
                                root={child.clone()} />
                        }).collect::<Html>() }
                    </div>
                }
            </div>
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

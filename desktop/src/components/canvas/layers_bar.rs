//! Layers.
use crate::components::canvas::selection_action::{selection_action, SelectionAction};
use crate::components::canvas::{CanvasStateAction, CanvasStateReducer, GraphStateReducer};
use thot_core::project::Asset as CoreAsset;
use thot_core::types::ResourceId;
use thot_ui::widgets::common::asset as asset_ui;
use yew::prelude::*;
use yew_icons::{Icon, IconId};

const ICON_SIZE: u8 = 16;

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

    let name = asset_ui::asset_display_name(&props.asset);
    let mut class = classes!("layer-asset");
    if selected {
        class.push("selected");
    }

    html! {
        <div {class} {onclick}>
            <div class={"info-group"}>
                <span class={"resource-icon"} style={asset_ui::asset_icon_color(&props.asset)}>
                    <Icon icon_id={asset_ui::asset_icon_id(&props.asset)}
                        width={ICON_SIZE.to_string()}
                        height={ICON_SIZE.to_string()} />
                </span>

                <span class={"name"} title={name.clone()}>
                    { name }
                </span>
            </div>
            <div class={"controls-group"}>
                if let Some(flags) = canvas_state.flags.get(&props.asset.rid) {
                    <span class={"alert-icon c-warning"}
                        title={flags.iter().map(|msg| format!("\u{2022} {msg}")).collect::<Vec<_>>().join("\n")}>

                        <Icon icon_id={IconId::BootstrapExclamationTriangle}
                            width={ICON_SIZE.to_string()}
                            height={ICON_SIZE.to_string()} />
                    </span>
                }
            </div>
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

    let mut assets = props.assets.clone();
    assets.sort_by(|a, b| {
        let a_name = asset_ui::asset_display_name(a);
        let b_name = asset_ui::asset_display_name(b);
        a_name.cmp(&b_name)
    });

    let mut class = classes!("layer", "assets");
    if *expanded_state {
        class.push("expanded")
    }

    html! {
        <div {class}>
            <div class={"layer-title"}>
                <span class={"layer-expand"}
                    onclick={toggle_expanded_state}>
                    if *expanded_state {
                        <Icon icon_id={IconId::FontAwesomeSolidCaretDown}
                            width={ICON_SIZE.to_string()}
                            height={ICON_SIZE.to_string()} />
                    } else {
                        <Icon icon_id={IconId::FontAwesomeSolidCaretRight}
                            width={ICON_SIZE.to_string()}
                            height={ICON_SIZE.to_string()} />
                    }
                </span>

                <span class={"resource-icon"}>
                    <Icon icon_id={IconId::BootstrapFiles}
                        width={ICON_SIZE.to_string()}
                        height={ICON_SIZE.to_string()} />
                </span>

                <span class={"name"}>{ "Assets" }</span>
            </div>

            <div class={"layer-assets"}>
                { assets.into_iter().map(|asset| html! {
                    <Asset {asset} />
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

    let onclick_toggle_visibility = {
        let root = root.rid.clone();
        use_callback(canvas_state.clone(), move |e: MouseEvent, canvas_state| {
            e.stop_propagation();

            canvas_state.dispatch(CanvasStateAction::SetVisibility(
                root.clone(),
                !canvas_state.is_visible(&root),
            ))
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
            <div class={"layer-title"}
                {onclick}
                {ondblclick} >

                <div class={"info-group"}>
                    if children.len() > 0 || root.assets.len() > 0 {
                        <span class={"layer-expand"}
                            onclick={toggle_expanded_state}>
                            if *expanded_state {
                                <Icon icon_id={IconId::FontAwesomeSolidCaretDown}
                                    width={ICON_SIZE.to_string()}
                                    height={ICON_SIZE.to_string()} />
                            } else {
                                <Icon icon_id={IconId::FontAwesomeSolidCaretRight}
                                    width={ICON_SIZE.to_string()}
                                    height={ICON_SIZE.to_string()} />
                            }
                        </span>
                    }
                    <span class={"resource-icon"}>
                        <Icon icon_id={IconId::FontAwesomeRegularFolder}
                            width={ICON_SIZE.to_string()}
                            height={ICON_SIZE.to_string()} />
                    </span>
                    <span class={"name"}
                        title={root.properties.name.clone()}>
                        { &root.properties.name }
                    </span>
                </div>
                <div class={"controls-group"}>
                    if let Some(flags) = canvas_state.flags.get(&props.root) {
                        <span class={"alert-icon"}
                            title={flags.iter().map(|msg| format!("\u{2022} {msg}")).collect::<Vec<_>>().join("\n")}>

                            <Icon icon_id={IconId::BootstrapExclamationTriangle}
                                width={ICON_SIZE.to_string()}
                                height={ICON_SIZE.to_string()} />
                        </span>
                    }
                    <span class={"visibility-toggle"}
                        onclick={onclick_toggle_visibility}>
                        if canvas_state.is_visible(&root.rid) {
                            <Icon icon_id={IconId::FontAwesomeRegularEye}
                                width={ICON_SIZE.to_string()}
                                height={ICON_SIZE.to_string()} />
                        } else {
                            <Icon icon_id={IconId::FontAwesomeRegularEyeSlash}
                                width={ICON_SIZE.to_string()}
                                height={ICON_SIZE.to_string()} />
                        }
                    </span>
                </div>
            </div>
            <div class={"resources"}>
                if root.assets.len() > 0 {
                    <Assets assets={root.assets.values().map(|asset| asset.clone()).collect::<Vec<_>>()} />
                }

                if children.len() > 0 {
                    <div class={"children"}>
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
        <div class={"layers-bar"}>
            <Layer root={root.clone()} expanded={true} />
        </div>
    }
}

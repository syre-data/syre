//! UI for a `Container` preview within a [`ContainerTree`](super::ContainerTree).
use crate::types::ContainerPreview;
use crate::widgets::asset::AssetsPreview;
use crate::widgets::container::script_associations::ScriptAssociationsPreview;
use crate::widgets::metadata::MetadataPreview;
use crate::widgets::Tags;
use thot_core::project::container::{AssetMap, ScriptMap};
use thot_core::project::{Asset as CoreAsset, StandardProperties};
use thot_core::types::{ResourceId, ResourceMap};
use yew::prelude::*;

// *********************
// *** Settings Menu ***
// *********************

/// Menu items available in the [`Container`]'s settings menu.
#[derive(PartialEq, Clone, Debug)]
pub enum ContainerSettingsMenuEvent {
    /// Add an [`Asset`](thot_core::project::Asset).
    AddAsset,

    /// Analyze Container tree.
    Analyze,
}

/// Properties for [`ContainerSettingsMenu`].
#[derive(PartialEq, Properties)]
struct ContainerSettingsMenuProps {
    /// Callback when a menu item is clicked.
    pub onclick: Callback<ContainerSettingsMenuEvent>,
}

/// Container settings menu.
#[function_component(ContainerSettingsMenu)]
fn container_settings_menu(props: &ContainerSettingsMenuProps) -> Html {
    let onclick = {
        let onclick = props.onclick.clone();

        move |event: ContainerSettingsMenuEvent| {
            let onclick = onclick.clone();

            Callback::from(move |e: MouseEvent| {
                e.stop_propagation();
                onclick.emit(event.clone());
            })
        }
    };

    html! {
        <div class={classes!("container-settings-menu")}>
            <ul>
                <li class={classes!("clickable")}
                    onclick={onclick(ContainerSettingsMenuEvent::AddAsset)}>
                    { "Add Assets" }
                </li>
            </ul>
        </div>
    }
}

// *****************
// *** Container ***
// *****************

#[derive(Properties, PartialEq)]
pub struct ContainerProps {
    pub rid: ResourceId,
    pub properties: StandardProperties,
    pub assets: AssetMap,
    pub scripts: ScriptMap,
    pub script_names: ResourceMap<String>,

    #[prop_or_default]
    pub class: Classes,

    #[prop_or(ContainerPreview::None)]
    pub preview: ContainerPreview,

    /// Callback to run when the `Container` is clicked.
    #[prop_or_default]
    pub onclick: Callback<()>,

    /// Callback to run when the `Container` is double clicked.
    #[prop_or_default]
    pub ondblclick: Callback<()>,

    /// Callback to run when an Asset is cilcked.
    #[prop_or_default]
    pub onclick_asset: Option<Callback<ResourceId>>,

    /// Callback to run when an Asset is double cilcked.
    #[prop_or_default]
    pub ondblclick_asset: Option<Callback<ResourceId>>,

    /// Callback to run when Assets are added to the Container.
    #[prop_or_default]
    pub onadd_assets: Option<Callback<String>>,

    /// Callback to run when the add child button is clicked.
    #[prop_or_default]
    pub onadd_child: Option<Callback<ResourceId>>,

    /// Callback when container settings button is clicked.
    /// If not provided, button is not shown.
    ///
    /// # Fields
    /// 1. [`ResourceId`] of the [`Container`](thot_core::project::Container)
    ///     the event was called on.
    /// 2. [`SettingsMenuEvent`] indicating which action was requested.
    #[prop_or_default]
    pub on_settings_event: Option<Callback<ContainerSettingsMenuEvent>>,

    /// Callback when container properties edit button is clicked.
    #[prop_or_default]
    pub onclick_add_assets: Option<Callback<ResourceId>>,

    /// Callback when container script edit button is clicked.
    #[prop_or_default]
    pub onclick_edit_scripts: Option<Callback<ResourceId>>,

    /// Callback when visibility toggle button is clicked.
    #[prop_or_default]
    pub onclick_toggle_visibility: Option<Callback<ResourceId>>,

    /// Callback to run when a user drops a file on the Container.
    #[prop_or_default]
    pub ondrop: Option<Callback<web_sys::DragEvent>>,
}

/// A Container node within a Container tree.
#[function_component(Container)]
pub fn container(props: &ContainerProps) -> Html {
    let show_settings_menu = use_state(|| false);

    let assets = props
        .assets
        .iter()
        .map(|(_rid, asset): (&ResourceId, &CoreAsset)| asset.clone())
        .collect::<Vec<CoreAsset>>();

    let prevent_drag_default = Callback::from(|e: web_sys::DragEvent| {
        e.prevent_default();
    });

    let onclick = {
        let onclick = props.onclick.clone();

        Callback::from(move |_: MouseEvent| {
            onclick.emit(());
        })
    };

    let ondblclick = {
        let ondblclick = props.ondblclick.clone();

        Callback::from(move |_: MouseEvent| {
            ondblclick.emit(());
        })
    };

    let onclick_settings = {
        let show_settings_menu = show_settings_menu.clone();

        Callback::from(move |e: MouseEvent| {
            e.stop_propagation();
            show_settings_menu.set(!*show_settings_menu);
        })
    };

    let onclick_add_assets = {
        let onclick_add_assets = props.onclick_add_assets.clone();
        let rid = props.rid.clone();

        Callback::from(move |e: MouseEvent| {
            e.stop_propagation();
            if let Some(onclick_add_assets) = onclick_add_assets.clone() {
                onclick_add_assets.emit(rid.clone());
            }
        })
    };

    let onclick_edit_scripts = {
        let onclick_edit_scripts = props.onclick_edit_scripts.clone();
        let rid = props.rid.clone();

        Callback::from(move |e: MouseEvent| {
            e.stop_propagation();
            if let Some(onclick_edit_scripts) = onclick_edit_scripts.clone() {
                onclick_edit_scripts.emit(rid.clone());
            }
        })
    };

    let onclick_toggle_visibility = {
        let onclick_toggle_visibility = props.onclick_toggle_visibility.clone();
        let rid = props.rid.clone();

        Callback::from(move |e: MouseEvent| {
            e.stop_propagation();
            if let Some(onclick_toggle_visibility) = onclick_toggle_visibility.clone() {
                onclick_toggle_visibility.emit(rid.clone());
            }
        })
    };

    let onadd_child = {
        let onadd_child = props.onadd_child.clone();
        let rid = props.rid.clone();

        Callback::from(move |e: MouseEvent| {
            e.stop_propagation();
            if let Some(onadd_child) = onadd_child.clone() {
                onadd_child.emit(rid.clone());
            }
        })
    };

    // inject closing setings menu on click to `on_settings_event` callback
    let on_settings_event = props.on_settings_event.clone().map(|on_settings_event| {
        let show_settings_menu = show_settings_menu.clone();

        Callback::from(move |event: ContainerSettingsMenuEvent| {
            show_settings_menu.set(false); // close settigns menu
            on_settings_event.emit(event); // trigger callback
        })
    });

    let class = classes!("container-node", props.class.clone());

    html! {
        <div {class}
            {onclick}
            {ondblclick}
            ondragenter={prevent_drag_default.clone()}
            ondragover={prevent_drag_default}
            ondrop={props.ondrop.clone()} >

            if let Some(on_settings_event) = on_settings_event {
                <div class={classes!("container-settings-control")}>
                    <button
                        class={classes!("container-settings-toggle")}
                        onclick={onclick_settings}>{ "\u{22ee}" }</button>

                    if *show_settings_menu {
                        <ContainerSettingsMenu onclick={on_settings_event} />
                    }
                </div>
            }

            <div class={classes!("container-name")}>
                if let Some(name) = props.properties.name.as_ref() {
                    { &name }
                } else {
                    { "(no name)" }
                }
            </div>

            <div class={classes!("container-preview")}>
                { match props.preview {
                    ContainerPreview::None => { html! { <></> } },
                    ContainerPreview::Type => { html! {
                        if let Some(kind) = props.properties.kind.as_ref() {
                            { &kind }
                        } else {
                            { "(no type)" }
                        }
                    }},
                    ContainerPreview::Description => { html! {
                        if let Some(description) = props.properties.description.as_ref() {
                            { &description }
                        } else {
                            { "(no description)" }
                        }
                    }},
                    ContainerPreview::Tags => { html! {
                        <Tags value={props.properties.tags.clone()} />
                    }},
                    ContainerPreview::Metadata => { html! {
                        <MetadataPreview value={props.properties.metadata.clone()} />
                    }},
                    ContainerPreview::Assets => { html! {
                        <AssetsPreview
                            {assets}
                            onclick_asset={&props.onclick_asset}
                            ondblclick_asset={&props.ondblclick_asset} />
                    }},
                    ContainerPreview::Scripts => { html! {
                        <ScriptAssociationsPreview
                            scripts={props.scripts.clone()}
                            names={props.script_names.clone()}
                        />
                    }},
                }}
            </div>

            <div class={classes!("container-controls")}>
                <button
                    class={classes!("container-control")}
                    onclick={onclick_add_assets}>

                    { "[]" }
                </button>

                <button
                    class={classes!("container-control")}
                    onclick={onclick_edit_scripts}>

                    { "</>" }
                </button>

                <button
                    class={classes!("container-control")}
                    onclick={onclick_toggle_visibility}>

                    { "<o>" }
                </button>
            </div>
            <div class={classes!("add-child-container-control")}>
                <button onclick={onadd_child}>{ "+" }</button>
            </div>
       </div>
    }
}

#[cfg(test)]
#[path = "./container_test.rs"]
mod container_test;

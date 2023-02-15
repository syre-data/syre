//! UI for a `Container` preview within a [`ContainerTree`](super::ContainerTree).
use crate::components::form::{InlineInput, InlineTextarea};
use crate::types::ContainerPreview;
use crate::widgets::asset::AssetsPreview;
use crate::widgets::container::script_associations::ScriptAssociationsPreview;
use crate::widgets::metadata::MetadataPreview;
use crate::widgets::TagsEditor;
use thot_core::project::container::{AssetMap, ScriptMap};
use thot_core::project::{Asset as CoreAsset, Metadata, StandardProperties};
use thot_core::types::ResourceId;
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
    /// [`ResourceId`] of the associated [`Container`](thot_core::project::Container).
    pub rid: ResourceId,

    /// Callback when a menu item is clicked.
    pub onclick: Option<Callback<(ResourceId, ContainerSettingsMenuEvent)>>,
}

/// Container settings menu.
#[function_component(ContainerSettingsMenu)]
fn container_settings_menu(props: &ContainerSettingsMenuProps) -> Html {
    let onclick = {
        let rid = props.rid.clone();
        let onclick = props.onclick.clone();

        move |event: ContainerSettingsMenuEvent| {
            let rid = rid.clone();
            let onclick = onclick.clone();

            Callback::from(move |_: MouseEvent| {
                if let Some(onclick) = onclick.clone() {
                    onclick.emit((rid.clone(), event.clone()));
                }
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

    /// Callback to run when the `name` is changed.
    #[prop_or_default]
    pub onchange_name: Option<Callback<String>>,

    /// Callback to run when the `kind` is changed.
    #[prop_or_default]
    pub onchange_kind: Option<Callback<String>>,

    /// Callback to run when the `description` is changed.
    #[prop_or_default]
    pub onchange_description: Option<Callback<String>>,

    /// Callback to run when the `tags` are changed.
    #[prop_or_default]
    pub onchange_tags: Option<Callback<Vec<String>>>,

    /// Callback to run when `metadata` changes.
    #[prop_or_default]
    pub onchange_metadata: Option<Callback<Metadata>>,

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
    pub on_settings_event: Option<Callback<(ResourceId, ContainerSettingsMenuEvent)>>,

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

    let stop_propagation = Callback::from(|e: MouseEvent| {
        e.stop_propagation();
    });

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

        Callback::from(
            move |(rid, event): (ResourceId, ContainerSettingsMenuEvent)| {
                show_settings_menu.set(false); // close settigns menu
                on_settings_event.emit((rid, event)); // trigger callback
            },
        )
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
                        <ContainerSettingsMenu
                            rid={props.rid.clone()}
                            onclick={on_settings_event} />
                    }
                </div>
            }

            <div
                class={classes!("container-name")}
                ondblclick={stop_propagation.clone()}>

                <InlineInput<String>
                    placeholder={"Name"}
                    value={props.properties.name.clone()}
                    onchange={&props.onchange_name}>

                    { "(no name)" }
                </InlineInput<String>>
            </div>

            <div
                class={classes!("container-preview")}
                ondblclick={stop_propagation.clone()}>

                { match props.preview {
                    ContainerPreview::None => { html! { <></> } },
                    ContainerPreview::Type => { html! {
                        <InlineInput<String>
                            placeholder={"Type"}
                            value={props.properties.kind.clone()}
                            onchange={&props.onchange_kind}>

                            { "(no type)" }
                        </InlineInput<String>>
                    }},
                    ContainerPreview::Description => { html! {
                        <InlineTextarea placeholder={"Description"}
                            value={props.properties.description.clone()}
                            onchange={&props.onchange_description}>

                            { "(no description)" }
                        </InlineTextarea>
                    }},
                    ContainerPreview::Tags => { html! {
                        <TagsEditor value={props.properties.tags.clone()}
                            onchange={&props.onchange_tags}>
                        </TagsEditor>
                    }},
                    ContainerPreview::Metadata => { html! {
                        <MetadataPreview value={props.properties.metadata.clone()}
                            onchange={&props.onchange_metadata}>
                        </MetadataPreview>
                    }},
                    ContainerPreview::Assets => { html! {
                        <AssetsPreview
                            {assets}
                            onclick_asset={&props.onclick_asset}
                            ondblclick_asset={&props.ondblclick_asset} />
                    }},
                    ContainerPreview::Scripts => { html! {
                        <ScriptAssociationsPreview
                            scripts={props.scripts.clone()} />
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

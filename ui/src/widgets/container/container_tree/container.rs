//! UI for a `Container` preview within a [`ContainerTree`](super::ContainerTree).
use crate::types::ContainerPreview;
use crate::widgets::asset::AssetsPreview;
use crate::widgets::container::script_associations::ScriptAssociationsPreview;
use crate::widgets::metadata::MetadataPreview;
use crate::widgets::Tags;
use std::collections::HashSet;
use thot_core::project::container::{AssetMap, ScriptMap};
use thot_core::project::{Asset as CoreAsset, StandardProperties};
use thot_core::types::{ResourceId, ResourceMap};
use yew::prelude::*;

// ************
// *** Menu ***
// ************

// @todo: Possible items:
// + Analyze: Analyze subtree.
/// Menu items available in the [`Container`]'s menu.
#[derive(PartialEq, Clone, Debug)]
pub enum ContainerMenuEvent {
    /// Open the `Container`'s folder.
    OpenFolder,

    /// Add a [`Asset`](thot_core::project::Asset)s to a `Container` using custom options.
    AddAssets,

    /// Duplicate the `Contiainer` tree.
    DuplicateTree,

    /// Remove the `Contiainer` tree.
    Remove,
}

/// Properties for [`ContainerMenu`].
#[derive(PartialEq, Properties)]
struct ContainerMenuProps {
    #[prop_or_default]
    pub r#ref: NodeRef,

    /// Callback when a menu item is clicked.
    pub onclick: Callback<ContainerMenuEvent>,

    /// Indicates whether the Container is root
    #[prop_or(false)]
    pub is_root: bool,
}

/// Container menu.
#[function_component(ContainerMenu)]
fn container_menu(props: &ContainerMenuProps) -> Html {
    let onclick = {
        let onclick = props.onclick.clone();

        move |event: ContainerMenuEvent| {
            let onclick = onclick.clone();

            Callback::from(move |e: MouseEvent| {
                e.stop_propagation();
                onclick.emit(event.clone());
            })
        }
    };

    html! {
        <div ref={props.r#ref.clone()}
            class={classes!("container-menu")}>

            <ul>
                <li class={classes!("clickable")}
                    onclick={onclick(ContainerMenuEvent::OpenFolder)}>
                    { "Open folder" }
                </li>

                <li class={classes!("clickable")}
                    onclick={onclick(ContainerMenuEvent::AddAssets)}>
                    { "Add data" }
                </li>

                { if props.is_root { html!{} } else { html!{
                    <>
                    <li class={classes!("clickable")}
                        onclick={onclick(ContainerMenuEvent::DuplicateTree)}>
                        { "Duplicate Tree" }
                    </li>
                    <li class={classes!("clickable")}
                        onclick={onclick(ContainerMenuEvent::Remove)}>
                        { "Remove Tree" }
                    </li>
                    </>
                }}}
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
    pub r#ref: NodeRef,

    #[prop_or_default]
    pub class: Classes,

    #[prop_or(true)]
    pub visible: bool,

    #[prop_or(false)]
    pub is_root: bool,

    #[prop_or(ContainerPreview::Assets)]
    pub preview: ContainerPreview,

    #[prop_or_default]
    pub active_assets: HashSet<ResourceId>,

    /// Callback to run when the `Container` is clicked.
    #[prop_or_default]
    pub onclick: Callback<MouseEvent>,

    /// Callback to run when the `Container` is double clicked.
    #[prop_or_default]
    pub ondblclick: Callback<MouseEvent>,

    /// Callback to run when an Asset is cilcked.
    #[prop_or_default]
    pub onclick_asset: Option<Callback<(ResourceId, MouseEvent)>>,

    /// Callback to run when an Asset is double cilcked.
    #[prop_or_default]
    pub ondblclick_asset: Option<Callback<(ResourceId, MouseEvent)>>,

    /// Callback when an [`Asset`](CoreAsset) is to be deleted.
    #[prop_or_default]
    pub onclick_asset_remove: Option<Callback<ResourceId>>,

    /// Callback to run when Assets are added to the Container.
    #[prop_or_default]
    pub onadd_assets: Option<Callback<String>>,

    /// Callback to run when the add child button is clicked.
    #[prop_or_default]
    pub onadd_child: Option<Callback<ResourceId>>,

    /// Callback when container button is clicked.
    /// If not provided, button is not shown.
    ///
    /// # Fields
    /// 1. [`ResourceId`] of the [`Container`](thot_core::project::Container)
    ///     the event was called on.
    /// 2. [`ContainerMenuEvent`] indicating which action was requested.
    #[prop_or_default]
    pub on_menu_event: Option<Callback<ContainerMenuEvent>>,

    #[prop_or_default]
    pub ondragenter: Callback<DragEvent>,

    #[prop_or_default]
    pub ondragover: Callback<DragEvent>,

    #[prop_or_default]
    pub ondragleave: Callback<DragEvent>,

    #[prop_or_default]
    pub ondrop: Callback<DragEvent>,
}

/// A Container node within a Container tree.
#[function_component(Container)]
pub fn container(props: &ContainerProps) -> Html {
    let show_menu = use_state(|| false);
    let dragover_counter = use_state(|| 0);
    let menu_ref = use_node_ref();

    let assets = props
        .assets
        .iter()
        .map(|(_rid, asset): (&ResourceId, &CoreAsset)| asset.clone())
        .collect::<Vec<CoreAsset>>();

    let ondragenter = {
        let ondragenter = props.ondragenter.clone();
        let dragover_counter = dragover_counter.clone();

        Callback::from(move |e: DragEvent| {
            e.prevent_default();

            if *dragover_counter == 0 {
                ondragenter.emit(e);
            }

            dragover_counter.set(*dragover_counter + 1);
        })
    };

    let ondragover = {
        let ondragover = props.ondragover.clone();

        Callback::from(move |e: DragEvent| {
            e.prevent_default();
            ondragover.emit(e);
        })
    };

    let ondragleave = {
        let ondragleave = props.ondragleave.clone();
        let dragover_counter = dragover_counter.clone();

        Callback::from(move |e: DragEvent| {
            e.prevent_default();
            if *dragover_counter == 1 {
                ondragleave.emit(e);
            }

            dragover_counter.set(*dragover_counter - 1);
        })
    };

    let ondrop = {
        let dragover_counter = dragover_counter.clone();
        let ondrop = props.ondrop.clone();

        Callback::from(move |e: DragEvent| {
            e.prevent_default();
            dragover_counter.set(0);
            ondrop.emit(e);
        })
    };

    let onclick_menu = {
        let show_menu = show_menu.clone();

        Callback::from(move |e: MouseEvent| {
            e.stop_propagation();
            show_menu.set(!*show_menu);
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

    // inject closing setings menu on click to `on_menu_event` callback
    let on_menu_event = props.on_menu_event.clone().map(|on_menu_event| {
        let show_menu = show_menu.clone();

        Callback::from(move |event: ContainerMenuEvent| {
            show_menu.set(false); // close settigns menu
            on_menu_event.emit(event); // trigger callback
        })
    });

    let mut class = classes!("container-node", props.class.clone());
    if *dragover_counter > 0 {
        class.push("dragover-active");
    }

    html! {
        <div ref={props.r#ref.clone()}
            {class}
            onclick={props.onclick.clone()}
            ondblclick={props.ondblclick.clone()}
            {ondragenter}
            {ondragover}
            {ondragleave}
            {ondrop}
            data-rid={props.rid.clone()} >

            if let Some(on_menu_event) = on_menu_event {
                <div class={classes!("container-menu-control")}>
                    <button
                        class={classes!("container-menu-toggle")}
                        onclick={onclick_menu}>{ "\u{22ee}" }</button>

                    if *show_menu {
                        <ContainerMenu
                            r#ref={menu_ref}
                            onclick={on_menu_event}
                            is_root={props.is_root} />
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
                            active={props.active_assets.clone()}
                            onclick_asset={&props.onclick_asset}
                            ondblclick_asset={&props.ondblclick_asset}
                            onclick_asset_remove={&props.onclick_asset_remove}
                            />
                    }},

                    ContainerPreview::Scripts => { html! {
                        <ScriptAssociationsPreview
                            scripts={props.scripts.clone()}
                            names={props.script_names.clone()} />
                    }},
                }}
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

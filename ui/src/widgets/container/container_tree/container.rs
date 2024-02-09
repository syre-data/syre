//! UI for a `Container` preview within a [`ContainerTree`](super::ContainerTree).
use crate::constants;
use crate::types::ContainerPreview;
use crate::widgets::asset::AssetsPreview;
use crate::widgets::container::script_associations::ScriptAssociationsPreview;
use crate::widgets::metadata::MetadataPreview;
use crate::widgets::Tags;
use std::collections::{HashMap, HashSet};
use syre_core::project::container::{AssetMap, ScriptMap};
use syre_core::project::{Asset, ContainerProperties, ScriptAssociation};
use syre_core::types::{ResourceId, ResourceMap};
use yew::prelude::*;
use yew_icons::{Icon, IconId};

// ************
// *** Menu ***
// ************

// TODO Possible items:
// + Analyze: Analyze subtree.
/// Menu items available in the [`Container`]'s menu.
#[derive(PartialEq, Clone, Debug)]
pub enum ContainerMenuEvent {
    /// Open the `Container`'s folder.
    OpenFolder,

    /// Add a [`Asset`](syre_core::project::Asset)s to a `Container` using custom options.
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
    pub class: Classes,

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

    let mut class = props.class.clone();
    class.push("container-menu");

    html! {
        <ul ref={props.r#ref.clone()} {class}>
            <li onclick={onclick(ContainerMenuEvent::OpenFolder)}>
                { "Open folder" }
            </li>

            <li onclick={onclick(ContainerMenuEvent::AddAssets)}>
                { "Add data" }
            </li>

            { if props.is_root { html!{} } else { html!{
                <>
                <li onclick={onclick(ContainerMenuEvent::DuplicateTree)}>
                    { "Duplicate tree" }
                </li>
                <li onclick={onclick(ContainerMenuEvent::Remove)}>
                    { "Remove tree" }
                </li>
                </>
            }}}
        </ul>
    }
}

// *****************
// *** Container ***
// *****************

#[derive(PartialEq, Default)]
pub struct Flags {
    pub container: Vec<String>,
    pub assets: HashMap<ResourceId, Vec<String>>,
}

#[derive(Properties, PartialEq)]
pub struct ContainerProps {
    pub rid: ResourceId,
    pub properties: ContainerProperties,
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

    #[prop_or_default]
    pub flags: Flags,

    #[prop_or(ContainerPreview::Assets)]
    pub preview: ContainerPreview,

    #[prop_or_default]
    pub active_assets: HashSet<ResourceId>,

    #[prop_or_default]
    pub onmousedown: Callback<MouseEvent>,

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

    /// Callback when an [`Asset`] is to be deleted.
    #[prop_or_default]
    pub onclick_asset_remove: Option<Callback<ResourceId>>,

    /// Callback to run when Assets are added to the Container.
    #[prop_or_default]
    pub onadd_assets: Option<Callback<String>>,

    /// Callback to run when the add child button is clicked.
    #[prop_or_default]
    pub onadd_child: Option<Callback<ResourceId>>,

    /// Callback when a script association changes from the preview.
    #[prop_or_default]
    pub onchange_script_association: Callback<ScriptAssociation>,

    /// Callback when a script association changes from the preview.
    #[prop_or_default]
    pub onremove_script_association: Callback<ResourceId>,

    /// Callback when container button is clicked.
    /// If not provided, button is not shown.
    ///
    /// # Fields
    /// 1. [`ResourceId`] of the [`Container`](syre_core::project::Container)
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
    let dragover_counter = use_state(|| 0);
    let menu_ref = use_node_ref();

    let assets = props
        .assets
        .iter()
        .map(|(_rid, asset): (&ResourceId, &Asset)| asset.clone())
        .collect::<Vec<Asset>>();

    let ondragenter = use_callback(
        (props.ondragenter.clone(), dragover_counter.clone()),
        move |e: DragEvent, (ondragenter, dragover_counter)| {
            e.prevent_default();

            if **dragover_counter == 0 {
                ondragenter.emit(e);
            }

            dragover_counter.set(**dragover_counter + 1);
        },
    );

    let ondragover = use_callback(props.ondragover.clone(), move |e: DragEvent, ondragover| {
        e.prevent_default();
        ondragover.emit(e);
    });

    let ondragleave = use_callback(
        (props.ondragleave.clone(), dragover_counter.clone()),
        move |e: DragEvent, (ondragleave, dragover_counter)| {
            e.prevent_default();
            if **dragover_counter == 1 {
                ondragleave.emit(e);
            }

            dragover_counter.set(**dragover_counter - 1);
        },
    );

    let ondrop = use_callback(props.ondrop.clone(), {
        let dragover_counter = dragover_counter.setter();
        move |e: DragEvent, ondrop| {
            e.prevent_default();
            dragover_counter.set(0);
            ondrop.emit(e);
        }
    });

    let onadd_child = use_callback(
        (props.rid.clone(), props.onadd_child.clone()),
        move |e: MouseEvent, (rid, onadd_child)| {
            e.stop_propagation();
            if let Some(onadd_child) = onadd_child.clone() {
                onadd_child.emit(rid.clone());
            }
        },
    );

    // inject closing setings menu on click to `on_menu_event` callback
    let on_menu_event = props.on_menu_event.clone().map(|on_menu_event| {
        Callback::from(move |event: ContainerMenuEvent| {
            on_menu_event.emit(event);
        })
    });

    let mut class = classes!("container-node", props.class.clone());
    if *dragover_counter > 0 {
        class.push("dragover-active");
    }

    html! {
        <div ref={props.r#ref.clone()}
            {class}
            onmousedown={props.onmousedown.clone()}
            onclick={props.onclick.clone()}
            ondblclick={props.ondblclick.clone()}
            {ondragenter}
            {ondragover}
            {ondragleave}
            {ondrop}
            data-rid={props.rid.clone()} >

            if let Some(on_menu_event) = on_menu_event {
                <div class={"container-menu-control dropdown-group"}>
                    <span class={"container-menu-toggle"}>
                        { "\u{22ee}" }
                    </span>

                    <ContainerMenu
                        class={"dropdown-menu"}
                        r#ref={menu_ref}
                        onclick={on_menu_event}
                        is_root={props.is_root} />
                </div>
            }

            <div class={"header"}>
                <div class={"container-name"}>
                    { &props.properties.name }
                </div>
                if props.flags.container.len() > 0 {
                    <span class={"alert-icon c-warning"}
                        title={props.flags.container.iter().map(|msg| format!("\u{2022} {msg}")).collect::<Vec<_>>().join("\n")}>

                        <Icon icon_id={IconId::BootstrapExclamationTriangle}
                            class={"syre-ui-icon"} />
                    </span>
                }
            </div>

            <div class={"body"}>
                <div class={"container-preview"}>
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
                                flags={props.flags.assets.clone()}
                                active={props.active_assets.clone()}
                                onclick_asset={&props.onclick_asset}
                                ondblclick_asset={&props.ondblclick_asset}
                                onclick_asset_remove={&props.onclick_asset_remove} />
                        }},

                        ContainerPreview::Scripts => { html! {
                            <ScriptAssociationsPreview
                                scripts={props.scripts.clone()}
                                names={props.script_names.clone()}
                                onchange={&props.onchange_script_association}
                                onremove={&props.onremove_script_association} />
                        }},
                    }}
                </div>
            </div>
            <div class={"add-child-container-control"}>
                <button onclick={onadd_child}>{ "+" }</button>
            </div>
       </div>
    }
}

//! Main [`Container`](CoreContainer) editor.
use super::{AssetsList, PropertiesEditor, ScriptAssociationsEditor};
use crate::components::navigation::{TabBar, TabKey};
use crate::widgets::MetadataEditor;
use indexmap::IndexMap;
use std::hash::Hash;
use thot_core::project::container::ScriptMap;
use thot_core::project::{Container as CoreContainer, Metadata, StandardProperties};
use yew::prelude::*;
use yew::virtual_dom::Key;

// ************
// *** View ***
// ************

/// Tabs for [`ContainerEditor`].
#[derive(Clone, PartialEq, Eq, Hash)]
pub enum EditorView {
    Properties,
    Metadata,
    Scripts,
    Assets,
}

impl Into<String> for EditorView {
    fn into(self) -> String {
        match self {
            EditorView::Properties => "Properties",
            EditorView::Metadata => "Metadata",
            EditorView::Scripts => "Scripts",
            EditorView::Assets => "Assets",
        }
        .to_string()
    }
}

impl Into<Key> for EditorView {
    fn into(self) -> Key {
        Into::<String>::into(self).into()
    }
}

// ************************
// *** Container Editor ***
// ************************

/// Arguments for [`ContainerEditor`].
#[derive(Properties, PartialEq)]
pub struct ContainerEditorProps {
    /// Initial value.
    #[prop_or_else(CoreContainer::new)]
    pub container: CoreContainer,

    /// Callback when properties are changed.
    #[prop_or_default]
    pub onchange_properties: Option<Callback<StandardProperties>>,

    #[prop_or_default]
    /// Callback when metadata is changed.
    pub onchange_metadata: Option<Callback<Metadata>>,

    #[prop_or_default]
    /// Callback when script associations are changed.
    pub onchange_scripts: Option<Callback<ScriptMap>>,
}

/// [`Container`](CoreContainer) editor.
#[function_component(ContainerEditor)]
pub fn container_editor(props: &ContainerEditorProps) -> Html {
    let active_view = use_state(|| EditorView::Properties);

    let views = vec![
        EditorView::Properties,
        EditorView::Metadata,
        EditorView::Scripts,
        EditorView::Assets,
    ];

    let mut tabs = IndexMap::with_capacity(views.len());
    for view in views {
        tabs.insert(view.clone(), view.into());
    }

    let onclick_tab = {
        let active_view = active_view.clone();

        Callback::from(move |view| {
            active_view.set(view);
        })
    };

    html! {
        <div class={classes!("container-editor")} style={"display: flex;"}>
            <div class={classes!("sidebar")}>
                <TabBar<EditorView> active={(*active_view).clone()} {tabs} {onclick_tab} />
            </div>
            <div class={classes!("content")}>
                { match *active_view {
                    EditorView::Properties => html! {
                        <PropertiesEditor
                            properties={props.container.properties.clone()}
                            onchange={props.onchange_properties.clone()} />
                    },
                    EditorView::Metadata => html! {
                        <MetadataEditor
                            value={props.container.properties.metadata.clone()}
                            onchange={props.onchange_metadata.clone()} />
                    },
                    EditorView::Scripts => html! {
                        <ScriptAssociationsEditor associations={props.container.scripts.clone()} />
                    },
                    EditorView::Assets => html! {
                        <AssetsList assets={props.container.assets.clone()}/>
                    },
                }}
            </div>
        </div>
    }
}

#[cfg(test)]
#[path = "./main_test.rs"]
mod main_test;

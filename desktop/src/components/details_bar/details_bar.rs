//! Project workspace details bar.
use super::asset_editor::AssetEditor;
use super::container_editor::ContainerEditor;
use super::project_actions::ProjectActions;
use super::script_associations_editor::ScriptAssociationsEditor;
use crate::components::canvas::CanvasStateReducer;
use thot_core::project::Asset as CoreAsset;
use thot_core::types::ResourceId;
use yew::prelude::*;

#[derive(Clone, PartialEq)]
pub enum DetailsBarWidget {
    /// Asset editor.
    AssetEditor(ResourceId),

    /// Script associations editor.
    ///
    /// # Fields
    /// 1. `Container`.
    /// 2. `onsave` callback.
    ScriptsAssociationsEditor(ResourceId, Option<Callback<()>>),

    /// Container editor.
    ContainerEditor(ResourceId),
}

#[function_component(DetailsBar)]
pub fn details_bar() -> Html {
    let canvas_state =
        use_context::<CanvasStateReducer>().expect("`WorkspaceStateReducer` context not found");

    html! {
        <div class={classes!("project-canvas-details-bar")}>
            if let Some(widget) = canvas_state.details_bar_widget.clone() {
                { match widget {
                    DetailsBarWidget::AssetEditor(rid) => html! {
                        <AssetEditor {rid} />
                    },

                    DetailsBarWidget::ScriptsAssociationsEditor(container, onsave) => html! {
                        <ScriptAssociationsEditor {container} {onsave} />
                    },

                    DetailsBarWidget::ContainerEditor(rid) => html! {
                        <ContainerEditor {rid} />
                    }
                }}
            } else {{
                // default
                html! {
                    <ProjectActions />
                }}
            }
        </div>
    }
}

#[cfg(test)]
#[path = "./details_bar_test.rs"]
mod details_bar_test;

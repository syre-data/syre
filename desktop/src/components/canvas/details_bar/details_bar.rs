//! Project workspace details bar.
use super::asset_bulk_editor::AssetBulkEditor;
use super::asset_editor::AssetEditor;
use super::container_bulk_editor::ContainerBulkEditor;
use super::container_editor::ContainerEditor;
use super::mixed_bulk_editor::MixedBulkEditor;
use super::project_actions::ProjectActions;
use crate::components::canvas::CanvasStateReducer;
use std::collections::HashSet;
use thot_core::types::ResourceId;
use yew::prelude::*;

#[derive(Clone, PartialEq)]
pub enum DetailsBarWidget {
    /// `Asset` editor.
    AssetEditor(ResourceId),

    /// `Container` editor.
    ContainerEditor(ResourceId),

    /// `Container` bulk editor.
    ContainerBulkEditor(HashSet<ResourceId>),

    /// `Asset` bulk editor.
    AssetBulkEditor(HashSet<ResourceId>),

    /// Mixed bulk editor.
    MixedBulkEditor(HashSet<ResourceId>),
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

                    DetailsBarWidget::ContainerEditor(rid) => html! {
                        <ContainerEditor {rid} />
                    },

                    DetailsBarWidget::ContainerBulkEditor(containers) => html! {
                        <ContainerBulkEditor {containers} />
                    },

                    DetailsBarWidget::AssetBulkEditor(assets) => html! {
                        <AssetBulkEditor {assets} />
                    },

                    DetailsBarWidget::MixedBulkEditor(resources) => html! {
                        <MixedBulkEditor {resources} />
                    },
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

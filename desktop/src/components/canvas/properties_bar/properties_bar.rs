//! Project workspace details bar.
use super::analysis_actions::ProjectAnalysisActions;
use super::asset_bulk_editor::AssetBulkEditor;
use super::asset_editor::AssetEditor;
use super::container_bulk_editor::ContainerBulkEditor;
use super::container_editor::ContainerEditor;
use super::mixed_bulk_editor::MixedBulkEditor;
use super::project_actions::ProjectActions;
use crate::components::canvas::CanvasStateReducer;
use std::collections::HashSet;
use syre_core::types::ResourceId;
use yew::prelude::*;

#[derive(Clone, PartialEq, Debug)]
pub enum PropertiesBarWidget {
    ProjectAnalysisActions,

    ProjectActions,

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

impl Default for PropertiesBarWidget {
    fn default() -> Self {
        Self::ProjectAnalysisActions
    }
}

#[function_component(PropertiesBar)]
pub fn properties_bar() -> Html {
    let canvas_state = use_context::<CanvasStateReducer>().unwrap();
    html! {
        <div class={classes!("project-canvas-details-bar")}>
            { match &canvas_state.properties_bar_widget{
                PropertiesBarWidget::ProjectAnalysisActions => html! {
                    <ProjectAnalysisActions />
                },

                PropertiesBarWidget::ProjectActions => html! {
                    <ProjectActions />
                },

                PropertiesBarWidget::AssetEditor(rid) => html! {
                    <AssetEditor rid={rid.clone()} />
                },

                PropertiesBarWidget::ContainerEditor(rid) => html! {
                    <ContainerEditor rid={rid.clone()} />
                },

                PropertiesBarWidget::ContainerBulkEditor(containers) => html! {
                    <ContainerBulkEditor containers={containers.clone()} />
                },

                PropertiesBarWidget::AssetBulkEditor(assets) => html! {
                    <AssetBulkEditor assets={assets.clone()} />
                },

                PropertiesBarWidget::MixedBulkEditor(resources) => html! {
                    <MixedBulkEditor resources={resources.clone()} />
                },
            }}
        </div>
    }
}

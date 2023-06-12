//! Bulk editor for Containers.
use super::super::GraphStateReducer;
use std::collections::HashSet;
use thot_core::types::ResourceId;
use thot_ui::widgets::bulk_editor::StandardPropertiesBulkEditor;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct ContainerBulkEditorProps {
    pub containers: HashSet<ResourceId>,
}

#[function_component(ContainerBulkEditor)]
pub fn container_bulk_editor(props: &ContainerBulkEditorProps) -> Html {
    let graph_state =
        use_context::<GraphStateReducer>().expect("`GraphStateReducer` context not found");

    let containers = props
        .containers
        .iter()
        .map(|rid| graph_state.graph.get(rid).expect("`Container` not found"));

    let properties = containers.map(|c| c.properties.clone()).collect::<Vec<_>>();

    html! {
        <StandardPropertiesBulkEditor {properties} />
    }
}

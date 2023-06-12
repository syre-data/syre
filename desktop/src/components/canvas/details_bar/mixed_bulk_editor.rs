//! Bulk editor for mixed resources.
use std::collections::HashSet;
use thot_core::types::ResourceId;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct MixedBulkEditorProps {
    pub resources: HashSet<ResourceId>,
}

#[function_component(MixedBulkEditor)]
pub fn mixed_bulk_editor(props: &MixedBulkEditorProps) -> Html {
    html! {
        <div>{ "Mixed Bulk Editor" }</div>
    }
}

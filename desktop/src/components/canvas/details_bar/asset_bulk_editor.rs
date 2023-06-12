//! Bulk editor for Assets.
use std::collections::HashSet;
use thot_core::types::ResourceId;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct AssetBulkEditorProps {
    pub assets: HashSet<ResourceId>,
}

#[function_component(AssetBulkEditor)]
pub fn asset_bulk_editor(props: &AssetBulkEditorProps) -> Html {
    html! {
        <div>{ "Asset Bulk Editor" }</div>
    }
}

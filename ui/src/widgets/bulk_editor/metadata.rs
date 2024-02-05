//! Bulk metadata editor.
use serde_json::Value as JsValue;
use std::collections::HashMap;
use yew::prelude::*;
use yew_icons::{Icon, IconId};

#[derive(Properties, PartialEq)]
pub struct MetadataBulkEditorProps {
    metadata: HashMap<String, JsValue>,
}

#[function_component(MetadataBulkEditor)]
pub fn metadata_bulk_editor(props: &MetadataBulkEditorProps) -> Html {
    html! {
        <div class={"syre-ui-metadata-bulk-editor"}>
            <div class={"metadata-header"}>
                <button class={"add-button"} type="button" onclick={onadd}>
                    <Icon class={"syre-ui-icon syre-ui-add-remove-icon"}
                        icon_id={IconId::HeroiconsSolidPlus}/>
                </button>
            </div>
            <div class={"add-metadatum-controls"}>
                if *add_metadatum_visible {
                    <MetadatumBuilder
                        {name_filter}
                        onsave={add_metadatum}
                        oncancel={oncancel_add_metadatum} />
                }
            </div>
            <ol class={"metadata-editor"}>
                { props.value.clone().into_iter().map(|(name, value)| html! {
                    <li key={name.clone()}>
                        <MetadatumEditor
                            name={name.clone()}
                            {value}
                            onchange={onchange(name.clone())}/>

                        <button class={"remove-button"} type="button" onclick={remove_metadatum(name)}>
                            <Icon class={"syre-ui-icon syre-ui-add-remove-icon"}
                                icon_id={IconId::HeroiconsSolidMinus}/>
                        </button>
                    </li>
                }).collect::<Html>() }
            </ol>
        </div>

    }
}

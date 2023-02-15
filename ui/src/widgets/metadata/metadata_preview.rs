//! Metadata preview.
use super::InlineMetadatumEditor;
use thot_core::project::Metadata;
use yew::prelude::*;

#[derive(PartialEq, Properties)]
pub struct MetadataPreviewProps {
    pub value: Metadata,

    #[prop_or_default]
    pub onchange: Option<Callback<Metadata>>,
}

#[function_component(MetadataPreview)]
pub fn metadata_preview(props: &MetadataPreviewProps) -> Html {
    let onchange = {
        let value = props.value.clone();
        let onchange = props.onchange.clone();

        move |key: String| {
            let value = value.clone();
            let onchange = onchange.clone();

            if let Some(onchange) = onchange {
                Some(Callback::from(move |val: serde_json::Value| {
                    let mut value = value.clone();
                    value.insert(key.clone(), val);
                    onchange.emit(value);
                }))
            } else {
                None
            }
        }
    };

    html! {
        <ol class={classes!("thot-ui-metadata-preview")}>
            { props.value.iter().map(|(name, value)| html! {
                <li key={name.clone()}>
                    <InlineMetadatumEditor
                        name={name.clone()}
                        value={value.clone()}
                        onchange={onchange(name.clone())} />
                </li>
            }).collect::<Html>() }
        </ol>
    }
}

#[cfg(test)]
#[path = "./metadata_preview_test.rs"]
mod metadata_preview_test;

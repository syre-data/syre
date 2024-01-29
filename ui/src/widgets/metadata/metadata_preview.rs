//! Metadata preview.
use syre_core::project::Metadata;
use yew::prelude::*;

#[derive(PartialEq, Properties)]
pub struct MetadataPreviewProps {
    pub value: Metadata,
}

#[function_component(MetadataPreview)]
pub fn metadata_preview(props: &MetadataPreviewProps) -> Html {
    html! {
        <ol class={classes!("syre-ui-metadata-preview")}>
            { props.value.iter().map(|(name, value)| html! {
                <li key={name.clone()}>
                    <span class={classes!("metadatum-key")}
                        title={name.clone()}>
                        { &name }
                    </span>

                    <span class={classes!("metadatum-value")}>
                        if value.is_null() {
                            { "(no value)" }
                        } else {
                            { value.to_string() }
                        }
                    </span>
                </li>
            }).collect::<Html>() }
        </ol>
    }
}

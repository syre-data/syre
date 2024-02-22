//! Excel template builder.
use std::path::PathBuf;
use syre_core::project::AssetProperties;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct OutputBuilderProps {
    pub onsubmit: Callback<(PathBuf, AssetProperties)>,

    #[prop_or_default]
    pub path: Option<PathBuf>,

    #[prop_or_default]
    pub properties: Option<AssetProperties>,
}

#[function_component(OutputBuilder)]
pub fn output_builder(props: &OutputBuilderProps) -> Html {
    let output_asset_form_node_ref = use_node_ref();

    let onsubmit = use_callback(props.onsubmit.clone(), {
        let output_asset_form_node_ref = output_asset_form_node_ref.clone();
        move |e: SubmitEvent, onsubmit| {
            e.prevent_default();
            let form = output_asset_form_node_ref
                .cast::<web_sys::HtmlFormElement>()
                .unwrap();

            let form_data = web_sys::FormData::new_with_form(&form).unwrap();
            let path = form_data.get("path").as_string().unwrap();
            let path = PathBuf::from(path.trim());

            let name = form_data.get("name").as_string().unwrap();
            let name = name.as_str().trim();
            let name = if name.is_empty() {
                None
            } else {
                Some(name.to_string())
            };

            let kind = form_data.get("kind").as_string().unwrap();
            let kind = kind.as_str().trim();
            let kind = if kind.is_empty() {
                None
            } else {
                Some(kind.to_string())
            };

            let tags = form_data.get("tags").as_string().unwrap();
            let tags = tags.as_str().trim();
            let tags = if tags.is_empty() {
                Vec::new()
            } else {
                tags.split(",")
                    .filter_map(|tag| {
                        let tag = tag.trim();
                        if tag.is_empty() {
                            None
                        } else {
                            Some(tag.to_string())
                        }
                    })
                    .collect::<Vec<_>>()
            };

            let description = form_data.get("description").as_string().unwrap();
            let description = description.as_str().trim();
            let description = if description.is_empty() {
                None
            } else {
                Some(description.to_string())
            };

            let mut properties = AssetProperties::new();
            properties.name = name;
            properties.kind = kind;
            properties.tags = tags;
            properties.description = description;

            onsubmit.emit((path, properties))
        }
    });

    html! {
        <form ref={output_asset_form_node_ref} {onsubmit}>
            <div>
                <input name={"path"} placeholder={"Path"} />
            </div>
            <div>
                <input name={"name"} placeholder={"Name"} />
            </div>
            <div>
                <input name={"kind"} placeholder={"Type"} />
            </div>
            <div>
                <input name={"tags"} placeholder={"Tags"} />
            </div>
            <div>
                <textarea name={"description"} placeholder={"Description"}></textarea>
            </div>
            <div>
                <button>{ "Next" }</button>
            </div>
        </form>
    }
}

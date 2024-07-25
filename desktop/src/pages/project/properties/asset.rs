use super::INPUT_DEBOUNCE;
use crate::pages::project::state;
use description::Editor as Description;
use kind::Editor as Kind;
use leptos::*;
use metadata::{AddDatum, Editor as Metadata};
use name::Editor as Name;
use serde::Serialize;
use std::path::PathBuf;
use syre_core::types::ResourceId;
use tags::Editor as Tags;

#[derive(derive_more::Deref, Clone)]
struct ActiveAsset(state::Asset);

#[component]
pub fn Editor(asset: state::Asset) -> impl IntoView {
    provide_context(ActiveAsset(asset.clone()));

    view! {
        <div>
            <h3>"Asset"</h3>
            <form on:submit=|e| e.prevent_default()>
                <div>
                    <label>"Name" <Name/></label>
                </div>
                <div>
                    <label>"Type" <Kind/></label>
                </div>
                <div>
                    <label>"Description" <Description/></label>
                </div>
                <div>
                    <label>"Tags" <Tags/></label>
                </div>
                <div>
                    <label>"Metadata" <AddDatum/> <Metadata/></label>
                </div>
            </form>
            <div>{move || asset.path().with(|path| path.to_string_lossy().to_string())}</div>
        </div>
    }
}

mod name {
    use super::{update_properties, ActiveAsset, INPUT_DEBOUNCE};
    use crate::{
        components::{form::debounced::InputText, message::Builder as Message},
        pages::project::state,
        types::Messages,
    };
    use leptos::*;

    #[component]
    pub fn Editor() -> impl IntoView {
        let project = expect_context::<state::Project>();
        let graph = expect_context::<state::Graph>();
        let asset = expect_context::<ActiveAsset>();
        let messages = expect_context::<Messages>();

        let input_value = {
            let value = asset.name().read_only();
            move || value.with(|value| value.clone().unwrap_or(String::new()))
        };

        // TODO: Handle errors with messages.
        // See https://github.com/leptos-rs/leptos/issues/2041
        let oninput = {
            // let messages = messages.write_only();
            move |value: String| {
                let mut properties = asset.as_properties();
                let value = value.trim();
                properties.name = if value.is_empty() {
                    None
                } else {
                    Some(value.to_string())
                };

                let project = project.rid().get_untracked();
                let node = asset
                    .rid()
                    .with_untracked(|rid| graph.find_by_asset_id(rid).unwrap());
                let container_path = graph.path(&node).unwrap();
                let asset_path = asset.path().get_untracked();
                // let messages = messages.clone();
                spawn_local(async move {
                    if let Err(err) =
                        update_properties(project, container_path, asset_path, properties).await
                    {
                        tracing::error!(?err);
                        let mut msg = Message::error("Could not save container");
                        msg.body(format!("{err:?}"));
                        // messages.update(|messages| messages.push(msg.build()));
                    }
                });
            }
        };

        view! { <InputText value=Signal::derive(input_value) oninput debounce=INPUT_DEBOUNCE/> }
    }
}

mod kind {
    use super::{
        super::common::kind::Editor as KindEditor, update_properties, ActiveAsset, INPUT_DEBOUNCE,
    };
    use crate::{components::message::Builder as Message, pages::project::state, types::Messages};
    use leptos::*;

    #[component]
    pub fn Editor() -> impl IntoView {
        let project = expect_context::<state::Project>();
        let graph = expect_context::<state::Graph>();
        let asset = expect_context::<ActiveAsset>();
        let messages = expect_context::<Messages>();

        // TODO: Handle errors with messages.
        // See https://github.com/leptos-rs/leptos/issues/2041
        let oninput = {
            let asset = asset.clone();
            // let messages = messages.write_only();
            move |value: Option<String>| {
                let mut properties = asset.as_properties();
                properties.kind = value;

                let project = project.rid().get_untracked();
                let node = asset
                    .rid()
                    .with_untracked(|rid| graph.find_by_asset_id(rid).unwrap());
                let container_path = graph.path(&node).unwrap();
                let asset_path = asset.path().get_untracked();
                // let messages = messages.clone();
                spawn_local(async move {
                    if let Err(err) =
                        update_properties(project, container_path, asset_path, properties).await
                    {
                        tracing::error!(?err);
                        let mut msg = Message::error("Could not save container");
                        msg.body(format!("{err:?}"));
                        // messages.update(|messages| messages.push(msg.build()));
                    }
                });
            }
        };

        view! { <KindEditor value=asset.kind().read_only() oninput debounce=INPUT_DEBOUNCE/> }
    }
}

mod description {
    use super::{
        super::common::description::Editor as DescriptionEditor, update_properties, ActiveAsset,
        INPUT_DEBOUNCE,
    };
    use crate::{components::message::Builder as Message, pages::project::state, types::Messages};
    use leptos::*;

    #[component]
    pub fn Editor() -> impl IntoView {
        let project = expect_context::<state::Project>();
        let graph = expect_context::<state::Graph>();
        let asset = expect_context::<ActiveAsset>();
        let messages = expect_context::<Messages>();

        // TODO: Handle errors with messages.
        // See https://github.com/leptos-rs/leptos/issues/2041
        let oninput = {
            let asset = asset.clone();
            // let messages = messages.write_only();
            move |value: Option<String>| {
                let mut properties = asset.as_properties();
                properties.description = value;

                let project = project.rid().get_untracked();
                let node = asset
                    .rid()
                    .with_untracked(|rid| graph.find_by_asset_id(rid).unwrap());
                let container_path = graph.path(&node).unwrap();
                let asset_path = asset.path().get_untracked();
                // let messages = messages.clone();
                spawn_local(async move {
                    if let Err(err) =
                        update_properties(project, container_path, asset_path, properties).await
                    {
                        tracing::error!(?err);
                        let mut msg = Message::error("Could not save container");
                        msg.body(format!("{err:?}"));
                        // messages.update(|messages| messages.push(msg.build()));
                    }
                });
            }
        };

        view! { <DescriptionEditor value=asset.description().read_only() oninput debounce=INPUT_DEBOUNCE/> }
    }
}

mod tags {
    use super::{
        super::common::tags::Editor as TagsEditor, update_properties, ActiveAsset, INPUT_DEBOUNCE,
    };
    use crate::{components::message::Builder as Message, pages::project::state, types::Messages};
    use leptos::*;

    #[component]
    pub fn Editor() -> impl IntoView {
        let project = expect_context::<state::Project>();
        let graph = expect_context::<state::Graph>();
        let asset = expect_context::<ActiveAsset>();
        let messages = expect_context::<Messages>();

        // TODO: Handle errors with messages.
        // See https://github.com/leptos-rs/leptos/issues/2041
        let oninput = {
            let asset = asset.clone();
            // let messages = messages.write_only();
            move |value: Vec<String>| {
                let mut properties = asset.as_properties();
                properties.tags = value;

                let project = project.rid().get_untracked();
                let node = asset
                    .rid()
                    .with_untracked(|rid| graph.find_by_asset_id(rid).unwrap());
                let container_path = graph.path(&node).unwrap();
                let asset_path = asset.path().get_untracked();
                // let messages = messages.clone();
                spawn_local(async move {
                    if let Err(err) =
                        update_properties(project, container_path, asset_path, properties).await
                    {
                        tracing::error!(?err);
                        let mut msg = Message::error("Could not save container");
                        msg.body(format!("{err:?}"));
                        // messages.update(|messages| messages.push(msg.build()));
                    }
                });
            }
        };

        view! { <TagsEditor value=asset.tags().read_only() oninput debounce=INPUT_DEBOUNCE/> }
    }
}

mod metadata {
    use super::{
        super::common::metadata::ValueEditor, update_properties, ActiveAsset, INPUT_DEBOUNCE,
    };
    use crate::{components::message::Builder as Message, pages::project::state, types::Messages};
    use leptos::*;
    use syre_core::types::{ResourceId, Value};

    #[component]
    pub fn Editor() -> impl IntoView {
        let project = expect_context::<state::Project>();
        let graph = expect_context::<state::Graph>();
        let asset = expect_context::<ActiveAsset>();

        let remove_datum = {
            let asset = asset.clone();
            move |key| {
                let mut properties = asset.as_properties();
                properties.metadata.retain(|k, _| k != &key);

                let project = project.rid().get_untracked();
                let node = asset
                    .rid()
                    .with_untracked(|rid| graph.find_by_asset_id(rid).unwrap());
                let container_path = graph.path(&node).unwrap();
                let asset_path = asset.path().get_untracked();
                // let messages = messages.clone();
                spawn_local(async move {
                    if let Err(err) =
                        update_properties(project, container_path, asset_path, properties).await
                    {
                        tracing::error!(?err);
                        let mut msg = Message::error("Could not save container");
                        msg.body(format!("{err:?}"));
                        // messages.update(|messages| messages.push(msg.build()));
                    }
                });
            }
        };

        view! {
            <For each=asset.metadata().read_only() key=|(key, _)| key.clone() let:datum>
                <div>
                    <DatumEditor key=datum.0.clone() value=datum.1.read_only()/>
                    <button
                        type="button"
                        on:mousedown={
                            let key = datum.0.clone();
                            let remove_datum = remove_datum.clone();
                            move |_| remove_datum(key.clone())
                        }
                    >

                        "X"
                    </button>
                </div>
            </For>
        }
    }

    #[component]
    pub fn AddDatum() -> impl IntoView {
        let project = expect_context::<state::Project>();
        let graph = expect_context::<state::Graph>();
        let asset = expect_context::<ActiveAsset>();
        let (key, set_key) = create_signal("".to_string());
        let key = leptos_use::signal_debounced(key, INPUT_DEBOUNCE);
        let (value, set_value) = create_signal(Value::String("".to_string()));

        let invalid_key = {
            let metadata = asset.metadata().read_only();
            move || key.with(|key| metadata.with(|metadata| metadata.iter().any(|(k, _)| k == key)))
        };

        let add_metadatum = {
            move |_| {
                if asset
                    .metadata()
                    .with(|metadata| key.with(|key| metadata.iter().any(|(k, _)| k == key)))
                {
                    return;
                }

                let mut properties = asset.as_properties();
                let mut metadata = asset
                    .metadata()
                    .with_untracked(|metadata| metadata.as_properties());
                let key = key.with(|key| key.trim().to_string());
                if key.is_empty() {
                    todo!();
                }
                let value = value.with(|value| match value {
                    Value::String(value) => Value::String(value.trim().to_string()),
                    Value::Quantity { magnitude, unit } => Value::Quantity {
                        magnitude: magnitude.clone(),
                        unit: unit.trim().to_string(),
                    },
                    Value::Null
                    | Value::Bool(_)
                    | Value::Number(_)
                    | Value::Array(_)
                    | Value::Map(_) => value.clone(),
                });

                metadata.insert(key, value);
                properties.metadata = metadata;

                let node = asset
                    .rid()
                    .with_untracked(|rid| graph.find_by_asset_id(rid).unwrap());
                let container_path = graph.path(&node).unwrap();
                let asset_path = asset.path().get_untracked();
                let project = project.rid().get_untracked();
                spawn_local(async move {
                    if let Err(err) =
                        update_properties(project, container_path, asset_path, properties).await
                    {
                        tracing::error!(?err);
                        todo!()
                    }

                    set_key.update(|key| key.clear());
                    set_value(Value::String(String::new()));
                });
            }
        };

        view! {
            <div>
                <input
                    name="key"
                    class=("error", invalid_key)
                    prop:value=key
                    minlength="1"
                    on:input=move |e| set_key(event_target_value(&e))
                />
                <ValueEditor value set_value/>
                <button type="button" on:click=add_metadatum>
                    "Add"
                </button>
            </div>
        }
    }

    #[component]
    pub fn DatumEditor(key: String, value: ReadSignal<Value>) -> impl IntoView {
        assert!(!key.is_empty());
        let project = expect_context::<state::Project>();
        let graph = expect_context::<state::Graph>();
        let asset = expect_context::<ActiveAsset>();
        let messages = expect_context::<Messages>();
        let (input_value, set_input_value) = create_signal(value.get_untracked());
        let input_value = leptos_use::signal_debounced(input_value, INPUT_DEBOUNCE);

        create_effect(move |_| {
            set_input_value(value());
        });

        // TODO: Handle errors with messages.
        // See https://github.com/leptos-rs/leptos/issues/2041
        create_effect({
            let key = key.clone();
            move |asset_id| -> ResourceId {
                // let messages = messages.write_only();
                if asset.rid().with_untracked(|rid| {
                    if let Some(asset_id) = asset_id {
                        *rid != asset_id
                    } else {
                        false
                    }
                }) {
                    return asset.rid().get_untracked();
                }

                let value = input_value.with(|value| match value {
                    Value::String(value) => Value::String(value.trim().to_string()),
                    Value::Quantity { magnitude, unit } => Value::Quantity {
                        magnitude: magnitude.clone(),
                        unit: unit.trim().to_string(),
                    },
                    Value::Null
                    | Value::Bool(_)
                    | Value::Number(_)
                    | Value::Array(_)
                    | Value::Map(_) => value.clone(),
                });
                let mut properties = asset.as_properties();
                properties.metadata.insert(key.clone(), value);

                let project = project.rid().get_untracked();
                let node = asset
                    .rid()
                    .with_untracked(|rid| graph.find_by_asset_id(rid).unwrap());
                let container_path = graph.path(&node).unwrap();
                let asset_path = asset.path().get_untracked();
                // let messages = messages.clone();
                spawn_local(async move {
                    if let Err(err) =
                        update_properties(project, container_path, asset_path, properties).await
                    {
                        tracing::error!(?err);
                        let mut msg = Message::error("Could not save container");
                        msg.body(format!("{err:?}"));
                        // messages.update(|messages| messages.push(msg.build()));
                    }
                });

                // return the current id to track if the container changed
                asset.rid().get_untracked()
            }
        });

        view! {
            <div>
                <span>{key}</span>
                <ValueEditor value=input_value set_value=set_input_value/>
            </div>
        }
    }
}

async fn update_properties(
    project: ResourceId,
    container: impl Into<PathBuf>,
    asset: impl Into<PathBuf>,
    properties: syre_core::project::AssetProperties,
) -> Result<(), ()> {
    #[derive(Serialize)]
    struct Args {
        project: ResourceId,
        container: PathBuf,
        asset: PathBuf,
        // properties: syre_core::project::AssetProperties,
        properties: String, // TODO: Issue with serializing enum with Option. perform manually.
                            // See: https://github.com/tauri-apps/tauri/issues/5993
    }

    tauri_sys::core::invoke_result(
        "asset_properties_update",
        Args {
            project,
            container: container.into(),
            asset: asset.into(),
            properties: serde_json::to_string(&properties).unwrap(),
        },
    )
    .await
}

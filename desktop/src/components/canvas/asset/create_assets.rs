//! Create an [`Asset`](thot_core::project::Asset).
use crate::commands::container::{add_assets_from_info, get_container};
use crate::components::canvas::{GraphStateAction, GraphStateReducer};
use crate::hooks::use_container_path;
use std::path::PathBuf;
use tauri_sys::dialog::FileDialogBuilder;
use thot_core::project::Container;
use thot_core::types::ResourceId;
use thot_desktop_lib::types::AddAssetInfo;
use thot_local::types::AssetFileAction;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

/// Props for [`CreatesAsset`].
#[derive(Properties, PartialEq)]
pub struct CreateAssetsProps {
    pub container: ResourceId,

    #[prop_or_default]
    pub onsuccess: Option<Callback<()>>,
}

// @todo: Alert users for conflicting file paths or already created assets.
#[function_component(CreateAssets)]
pub fn create_assets(props: &CreateAssetsProps) -> HtmlResult {
    let graph_state =
        use_context::<GraphStateReducer>().expect("`GraphStateReducer` context not found");

    let paths: UseStateHandle<Vec<PathBuf>> = use_state(|| Vec::new());
    let file_action = use_state(|| AssetFileAction::Copy);
    let bucket: UseStateHandle<Option<PathBuf>> = use_state(|| None);
    let file_action_ref = use_node_ref();
    let container_path = use_container_path(props.container.clone())?;

    let onsubmit = {
        let graph_state = graph_state.clone();
        let container_id = props.container.clone();
        let paths = paths.clone();
        let file_action = file_action.clone();
        let bucket = bucket.clone();

        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();

            // @todo: Handle buckets.
            let assets = (*paths)
                .clone()
                .into_iter()
                .map(|path| AddAssetInfo {
                    path,
                    action: (*file_action).clone(),
                    bucket: (*bucket).clone(),
                })
                .collect::<Vec<AddAssetInfo>>();

            let graph_state = graph_state.clone();
            let container_id = container_id.clone();
            spawn_local(async move {
                // create assets
                match add_assets_from_info(container_id.clone(), assets).await {
                    Ok(_) => {}
                    Err(err) => {
                        tracing::debug!(err);
                        panic!("{err}");
                    }
                };
            });
        })
    };

    let onclick = {
        let paths = paths.clone();
        let default_path = container_path.clone();

        Callback::from(move |_: MouseEvent| {
            let paths = paths.clone();
            let default_path = default_path.clone();

            spawn_local(async move {
                let mut asset_paths = FileDialogBuilder::new();
                asset_paths.set_default_path(&default_path);

                asset_paths.set_title("Select Asset files");
                let asset_paths = asset_paths.pick_files().await;
                let asset_paths = asset_paths.expect("could not retrieve files");
                let asset_paths = match asset_paths {
                    None => Vec::new(),
                    Some(asset_paths) => asset_paths.into_iter().collect::<Vec<PathBuf>>(),
                };

                paths.set(asset_paths);
            });
        })
    };

    let file_action_options = {
        let file_actions = vec![
            AssetFileAction::Move,
            AssetFileAction::Copy,
            AssetFileAction::Reference,
        ];

        file_actions
            .into_iter()
            .map(|action| {
                let action = Into::<String>::into(action);
                html! {
                    <option value={action.clone()}>{ action.clone() }</option>
                }
            })
            .collect::<Html>()
    };

    let onchange_file_action = {
        let file_action = file_action.clone();
        let file_action_ref = file_action_ref.clone();

        Callback::from(move |_: Event| {
            let action = file_action_ref
                .cast::<web_sys::HtmlSelectElement>()
                .expect("could not cast element to select");

            let action =
                AssetFileAction::from_string(action.value()).expect("invalid action string");

            file_action.set(action);
        })
    };

    Ok(html! {
        <div class={classes!("thot-ui-create-asset")}>
            <form {onsubmit}>
                <div>
                    <label>{ "Files" }</label>
                    <ul>
                        { paths
                            .iter()
                            .map(|path| html! {
                                <li>{ path.to_str().expect("could not convert `PathBuf` to `str`") }</li>
                            }).collect::<Html>()
                        }
                    </ul>

                    <button type={"button"} {onclick}>
                        { if paths.len() > 0 { "Change" } else { "Select" } }
                    </button>
                </div>
                <div>
                    <label>
                        { "Action" }
                        <select ref={file_action_ref} onchange={onchange_file_action}>
                            { file_action_options }
                        </select>
                    </label>
                </div>
                <div>
                    <button>{ "Add Assets" }</button>
                </div>
            </form>
        </div>
    })
}

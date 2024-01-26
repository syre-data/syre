//! Gets the path of a `Container`.
use crate::commands::container::get_container_path;
use std::path::PathBuf;
use thot_core::types::ResourceId;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yew::suspense::{Suspension, SuspensionResult};

/// Gets the path of a `Container`.
#[hook]
pub fn use_container_path(rid: ResourceId) -> SuspensionResult<PathBuf> {
    let path: UseStateHandle<Option<PathBuf>> = use_state(|| None);
    if let Some(path) = path.as_ref() {
        return Ok(path.clone());
    }

    let (s, handle) = Suspension::new();
    {
        let path = path.clone();

        spawn_local(async move {
            match get_container_path(rid).await {
                None => {
                    tracing::debug!("could not get container path");
                    panic!("could not get container path");
                }

                Some(container_path) => {
                    path.set(Some(container_path));
                    handle.resume();
                }
            }
        });
    }

    Err(s)
}

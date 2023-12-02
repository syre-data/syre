//! Gets the path of a `Container`.
use crate::commands::common::ResourceIdArgs;
use crate::common::invoke;
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
            let container_path = invoke::<PathBuf>("get_container_path", ResourceIdArgs { rid })
                .await
                .expect("could not invoke `get_container_path`");

            path.set(Some(container_path));
            handle.resume();
        });
    }

    Err(s)
}

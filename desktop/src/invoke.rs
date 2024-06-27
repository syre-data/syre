//! Common functionality
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_wasm_bindgen as swb;

#[tracing::instrument(skip(args))]
pub async fn invoke<T>(command: &str, args: impl Serialize) -> T
where
    T: DeserializeOwned,
{
    tracing::debug!(command);
    tauri_sys::core::invoke(command, args).await
}

#[tracing::instrument(skip(args))]
pub async fn invoke_result<T, E>(command: &str, args: impl Serialize) -> Result<T, E>
where
    T: DeserializeOwned,
    E: DeserializeOwned,
{
    tracing::debug!(command);
    tauri_sys::core::invoke_result(command, args).await
}

//! Common functionality
use crate::error::{Error, Result};
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_wasm_bindgen as swb;
use thot_desktop_lib::error::Error as LibError;
use tracing::debug;

#[tracing::instrument(level = "debug", skip(args))]
pub async fn invoke<T>(command: &str, args: impl Serialize) -> Result<T>
where
    T: DeserializeOwned,
{
    debug!(command);
    inner::invoke(command, swb::to_value(&args)?)
        .await
        .map(|val| swb::from_value(val).expect("could not convert result"))
        .map_err(|err| {
            // TODO[h]: Unify errors.
            let err: LibError = swb::from_value(err).expect("could not convert error");
            Error::Binding(format!("{err:?}"))
        })
}

mod inner {
    use wasm_bindgen::prelude::*;
    use wasm_bindgen::JsValue;

    #[wasm_bindgen]
    extern "C" {
        #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "tauri"], catch)]
        pub async fn invoke(cmd: &str, args: JsValue) -> Result<JsValue, JsValue>;
    }
}

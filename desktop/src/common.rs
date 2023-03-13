//! Common functionality
use crate::error::{Error, Result};
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_wasm_bindgen as swb;
use thot_desktop_lib::error::Error as LibError;

pub async fn invoke<T>(command: &str, args: impl Serialize) -> Result<T>
where
    T: DeserializeOwned,
{
    inner::invoke(command, swb::to_value(&args)?)
        .await
        .map(|val| swb::from_value(val).expect("could not convert result"))
        .map_err(|err| {
            // @todo[2]: Unify errors.
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

#[cfg(test)]
#[path = "./common_test.rs"]
mod common_test;

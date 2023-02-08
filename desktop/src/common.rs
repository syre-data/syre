//! Common functionality
use crate::Result;
use serde::Serialize;
use serde_wasm_bindgen as swb;
use wasm_bindgen::JsValue;

// @todo: Deserialize result.
pub async fn invoke(command: &str, args: impl Serialize) -> Result<JsValue> {
    let res = inner::invoke(command, swb::to_value(&args)?).await;
    Ok(res)
}

mod inner {
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen]
    extern "C" {
        #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "tauri"])]
        pub async fn invoke(cmd: &str, args: JsValue) -> JsValue;
    }
}

#[cfg(test)]
#[path = "./common_test.rs"]
mod common_test;

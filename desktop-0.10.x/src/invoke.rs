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
    let value = inner::invoke(command, swb::to_value(&args).unwrap()).await;
    match swb::from_value(value) {
        Ok(value) => value,
        Err(err) => {
            panic!("{err:?}");
        }
    }
}

#[tracing::instrument(skip(args))]
pub async fn invoke_result<T, E>(command: &str, args: impl Serialize) -> Result<T, E>
where
    T: DeserializeOwned,
    E: DeserializeOwned,
{
    tracing::debug!(command);
    inner::invoke_result(command, swb::to_value(&args).unwrap())
        .await
        .map(|val| match swb::from_value(val) {
            Ok(value) => value,
            Err(err) => {
                panic!("{err:?}");
            }
        })
        .map_err(|err| match swb::from_value(err) {
            Ok(value) => value,
            Err(err) => {
                panic!("{err:?}");
            }
        })
}

mod inner {
    use wasm_bindgen::prelude::*;
    use wasm_bindgen::JsValue;

    #[wasm_bindgen]
    extern "C" {
        #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "tauri"])]
        pub async fn invoke(cmd: &str, args: JsValue) -> JsValue;
    }

    #[wasm_bindgen]
    extern "C" {
        #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "tauri"], js_name="invoke", catch)]
        pub async fn invoke_result(cmd: &str, args: JsValue) -> Result<JsValue, JsValue>;
    }
}

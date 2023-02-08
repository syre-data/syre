//! Create a [`Script`](CoreScript) from a file.
use std::path::PathBuf;
use tauri_sys::dialog::FileDialogBuilder;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct CreateScriptProps {
    #[prop_or("Add script")]
    pub text: &'static str,

    /// Callback when a user selectr a file to create a path from.
    #[prop_or_default]
    pub oncreate: Option<Callback<PathBuf>>,
}

#[function_component(CreateScript)]
pub fn create_script(props: &CreateScriptProps) -> Html {
    let onclick = {
        let oncreate = props.oncreate.clone();

        Callback::from(move |_: MouseEvent| {
            let oncreate = oncreate.clone();

            spawn_local(async move {
                let mut path = FileDialogBuilder::new();
                path.set_title("Script file").add_filter("Scripts", &["py"]); // @todo: Pull valid extensions from `Script`.
                                                                              // @todo: Set default path.

                let path = path
                    .pick_file()
                    .await
                    .expect("could not get user file selection");

                if let Some(path) = path {
                    if let Some(oncreate) = oncreate.as_ref() {
                        oncreate.emit(path);
                    }
                }
            });
        })
    };

    html! {
        <button {onclick}>{ &props.text }</button>
    }
}

#[cfg(test)]
#[path = "./create_script_test.rs"]
mod create_script_test;

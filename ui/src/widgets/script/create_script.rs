//! Create a [`Script`](CoreScript) from a file.
use std::collections::HashSet;
use std::path::PathBuf;
use tauri_sys::dialog::FileDialogBuilder;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yew_icons::{Icon, IconId};

#[derive(Properties, PartialEq)]
pub struct CreateScriptProps {
    /// Button contents.
    pub children: Html,

    #[prop_or_default]
    pub class: Classes,

    /// Callback when a user selectr a file to create a path from.
    #[prop_or_default]
    pub oncreate: Callback<HashSet<PathBuf>>,
}

#[function_component(CreateScript)]
pub fn create_script(props: &CreateScriptProps) -> Html {
    let onclick = {
        let oncreate = props.oncreate.clone();

        Callback::from(move |_: MouseEvent| {
            let oncreate = oncreate.clone();

            spawn_local(async move {
                let mut path = FileDialogBuilder::new();
                path.set_title("Script files")
                    .add_filter("Scripts", &["py", "R", "r"]); // @todo: Pull valid extensions from `Script`.
                                                               // @todo: Set default path.

                let paths = path
                    .pick_files()
                    .await
                    .expect("could not get user file selection");

                if let Some(paths) = paths {
                    let paths = paths.collect::<HashSet<PathBuf>>();
                    oncreate.emit(paths);
                }
            });
        })
    };

    html! {
        <button title={"Add analysis scripts"}
            class={props.class.clone()}
            {onclick}>

            { props.children.clone() }
        </button>
    }
}

//! Retrieve a user selected file.
use std::path::PathBuf;
use tauri_sys::dialog::FileDialogBuilder;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

#[derive(Clone, PartialEq)]
pub struct FileFilter<'a> {
    pub name: &'a str,
    pub extensions: &'a [&'a str],
}

#[derive(Clone, PartialEq)]
pub enum FileSelectorAction<'a> {
    PickFile,
    PickFileFiltered(FileFilter<'a>),
    PickFolder,
}

#[derive(Properties, PartialEq)]
pub struct FileSelectorProps {
    pub title: AttrValue,
    pub action: FileSelectorAction<'static>,

    #[prop_or_default]
    pub default_path: Option<PathBuf>,

    /// Open immedialely into a file selection.
    #[prop_or(true)]
    pub select_on_open: bool,

    /// Show the cancel button.
    #[prop_or(true)]
    pub show_cancel: bool,

    /// Called if the selection is canceled.
    /// Only relevant if `show_cancel` or `select_on_open` are `true.
    #[prop_or_default]
    pub oncancel: Callback<()>,

    #[prop_or_default]
    pub onsuccess: Callback<PathBuf>,
}

/// Allow the user to select a file
#[function_component(FileSelector)]
pub fn file_selector(props: &FileSelectorProps) -> Html {
    let path: UseStateHandle<Option<PathBuf>> = use_state(|| None);

    let onsubmit = {
        let onsuccess = props.onsuccess.clone();
        let path = path.clone();

        Callback::from(move |_: MouseEvent| {
            let Some(path) = path.as_ref() else {
                // path not set, but should be.
                return;
            };

            onsuccess.emit(path.clone());
        })
    };

    let onchange = {
        let title = props.title.clone();
        let action = props.action.clone();
        let default_path = props.default_path.clone();
        let path = path.clone();

        Callback::from(move |_: MouseEvent| {
            let title = title.clone();
            let action = action.clone();
            let path = path.clone();
            let default_path = (*path).clone().or_else(|| default_path.clone());

            spawn_local(async move {
                let user_path = get_user_path(title.as_str(), action, default_path).await;
                if user_path.is_some() {
                    path.set(user_path);
                }
            });
        })
    };

    let oncancel = {
        let oncancel = props.oncancel.clone();

        Callback::from(move |_: MouseEvent| {
            oncancel.emit(());
        })
    };

    // get initial location
    {
        let title = props.title.clone();
        let action = props.action.clone();
        let default_path = props.default_path.clone();
        let select_on_open = props.select_on_open.clone();
        let oncancel = props.oncancel.clone();
        let path = path.clone();

        use_effect_with((), move |_| {
            if select_on_open {
                spawn_local(async move {
                    let user_path = get_user_path(title.as_str(), action, default_path).await;
                    if user_path.is_some() {
                        path.set(user_path);
                    } else {
                        // canceled
                        oncancel.emit(());
                    }
                });
            }
        });
    }

    html! {
        <div class={classes!("thot-ui-file-selector")}>
            <div class={classes!("path-control")}>
                <span class={classes!("path")}>
                    if let Some(path) = (*path).clone() {
                        { path.to_str().expect("could not convert path to str") }
                    } else {
                        { &props.title }
                    }
                </span>

                <button onclick={onchange}>
                    if path.is_none() {
                        { "Set" }
                    } else {
                        { "Change" }
                    }
                </button>
            </div>
            <div class={classes!("controls")}>
                if props.show_cancel {
                    <button onclick={oncancel}>
                        { "Cancel" }
                    </button>
                }

                <button
                    disabled={path.is_none()}
                    onclick={onsubmit}>

                    { "Ok" }
                </button>
            </div>
        </div>
    }
}

async fn get_user_path<'a>(
    title: &str,
    action: FileSelectorAction<'a>,
    default_path: Option<PathBuf>,
) -> Option<PathBuf> {
    let mut file_selector = FileDialogBuilder::new();
    file_selector.set_title(title);
    if let Some(default_path) = default_path.as_ref() {
        file_selector.set_default_path(&default_path);
    }

    let user_path = match action {
        FileSelectorAction::PickFile => file_selector.pick_file().await,
        FileSelectorAction::PickFileFiltered(filter) => {
            file_selector
                .add_filter(filter.name, filter.extensions)
                .pick_file()
                .await
        }
        FileSelectorAction::PickFolder => file_selector.pick_folder().await,
    };

    // @todo: Return `Result`.
    let user_path = user_path.expect("could not retrieve file");
    user_path
}

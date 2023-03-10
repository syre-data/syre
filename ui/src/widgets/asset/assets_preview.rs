//! Assets preview.
use std::collections::HashSet;
use std::path::PathBuf;
use thot_core::project::Asset as CoreAsset;
use thot_core::types::ResourceId;
use yew::prelude::*;
use yew_icons::{Icon, IconId};

#[derive(Properties, PartialEq)]
pub struct AssetsPreviewProps {
    /// [`Asset`](CoreAsset)s to display.
    pub assets: Vec<CoreAsset>,

    /// Selected.
    #[prop_or_default]
    pub active: HashSet<ResourceId>,

    /// Callback when an [`Asset`](CoreAsset) is clicked.
    #[prop_or_default]
    pub onclick_asset: Option<Callback<(ResourceId, MouseEvent)>>,

    /// Callback when an [`Asset`](CoreAsset) is double clicked.
    #[prop_or_default]
    pub ondblclick_asset: Option<Callback<(ResourceId, MouseEvent)>>,

    /// Callback when an [`Asset`](CoreAsset) is to be deleted.
    #[prop_or_default]
    pub onclick_asset_remove: Option<Callback<ResourceId>>,
}

#[function_component(AssetsPreview)]
pub fn assets_preview(props: &AssetsPreviewProps) -> Html {
    html! {
        <div class={classes!("assets-preview")}>
            if props.assets.len() == 0 {
             { "(no assets)" }
            } else {
                <ol class={classes!("thot-ui-assets-list")}>
                    { props.assets.iter().map(|asset| {
                        let mut class = classes!("thot-ui-asset-preview", "clickable");
                        if props.active.contains(&asset.rid) {
                            class.push("active");
                        }

                       html! {
                            <li key={asset.rid.clone()}
                                {class}
                                onclick={delegate_callback_with_event(
                                    asset.rid.clone(),
                                    props.onclick_asset.clone()
                                )}
                                ondblclick={delegate_callback_with_event(
                                    asset.rid.clone(),
                                    props.ondblclick_asset.clone()
                                )} >

                                <Icon icon_id={ asset_icon_id(&asset) } width={"15px".to_owned()} height={"15px".to_owned()}/>

                                { asset_display_name(&asset) }
                                if props.onclick_asset_remove.is_some() {
                                    <button onclick={delegate_callback(
                                        asset.rid.clone(),
                                        props.onclick_asset_remove.clone()
                                    )}>
                                        { "X" }
                                    </button>
                                }
                            </li>
                        }
                    }).collect::<Html>() }
                </ol>
            }
        </div>
    }
}

// ***************
// *** helpers ***
// ***************

/// Gets the name to display for an [`Asset`](CoreAsset).
///
/// # Returns
/// The `name` if set, otherwise the `path`'s file name.
fn asset_display_name(asset: &CoreAsset) -> String {
    if let Some(name) = asset.properties.name.as_ref() {
        name.clone()
    } else {
        let path = Into::<PathBuf>::into(asset.path.clone());
        let name = path
            .file_name()
            .expect("`Asset.path` could not get file name");

        let name = name.to_str().expect("could not convert path to str");
        name.to_string()
    }
}

/// Gets the icon id to display for an [`Asset`](CoreAsset).
///
/// # Returns
/// The `IconId`.
fn asset_icon_id(asset: &CoreAsset) -> IconId {
    fn get_icon_id(extension: &str) -> IconId {
        match extension {
            "mp3" | "m4a" | "flac" | "wav" => IconId::FontAwesomeRegularFileAudio,
            "py" | "r" | "m" | "js" | "ts" | "cpp" | "c" | "rs" => {
                IconId::FontAwesomeRegularFileCode
            }
            "csv" | "xlsx" | "xlsm" | "xml" | "odf" => IconId::FontAwesomeRegularFileExcel,
            "png" | "svg" | "jpg" | "jpeg" | "tiff" | "bmp" => IconId::FontAwesomeRegularFileImage,
            "txt" => IconId::FontAwesomeRegularFileLines,
            "pdf" => IconId::FontAwesomeRegularFilePdf,
            "pptx" | "pptm" | "ppt" => IconId::FontAwesomeRegularFilePowerpoint,
            "doc" | "docm" | "docx" | "dot" => IconId::FontAwesomeRegularFileWord,
            "mp4" | "mov" | "wmv" | "avi" => IconId::FontAwesomeRegularFileVideo,
            "zip" | "zipx" | "rar" | "7z" | "gz" => IconId::FontAwesomeRegularFileZipper,
            _ => IconId::FontAwesomeRegularFile,
        }
    }

    let file_name = if let Some(name) = asset.properties.name.as_ref() {
        name.clone()
    } else {
        let path = Into::<PathBuf>::into(asset.path.clone());
        let name = path
            .file_name()
            .expect("`Asset.path` could not get file name");

        name.to_str()
            .expect("could not convert path to str")
            .to_owned()
    };
    let parts: Vec<&str> = file_name.split(".").collect();
    let extension = parts.last().unwrap_or(&"");
    let lower_case_extension = extension.to_lowercase();
    get_icon_id(&lower_case_extension)
}

/// Creates a [`Callback`] that passes the [`ResourceId`] through as the only parameter.
fn delegate_callback<In: 'static + Clone>(
    input: In,
    cb: Option<Callback<In>>,
) -> Callback<MouseEvent> {
    Callback::from(move |e: MouseEvent| {
        if let Some(cb) = cb.as_ref() {
            e.stop_propagation();
            cb.emit(input.clone());
        }
    })
}

/// Creates a [`Callback`] that passes the [`ResourceId`] through as the only parameter.
fn delegate_callback_with_event<In: 'static + Clone>(
    input: In,
    cb: Option<Callback<(In, MouseEvent)>>,
) -> Callback<MouseEvent> {
    Callback::from(move |e: MouseEvent| {
        if let Some(cb) = cb.as_ref() {
            e.stop_propagation();
            cb.emit((input.clone(), e));
        }
    })
}

#[cfg(test)]
#[path = "./assets_preview_test.rs"]
mod assets_preview_test;

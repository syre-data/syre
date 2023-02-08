//! Assets preview.
use std::path::PathBuf;
use thot_core::project::Asset as CoreAsset;
use thot_core::types::ResourceId;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct AssetsPreviewProps {
    /// [`Asset`](CoreAsset)s to display.
    pub assets: Vec<CoreAsset>,

    /// Callback when an [`Asset`](CoreAsset) is clicked.
    #[prop_or_default]
    pub onclick_asset: Option<Callback<ResourceId>>,

    /// Callback when an [`Asset`](CoreAsset) is double clicked.
    #[prop_or_default]
    pub ondblclick_asset: Option<Callback<ResourceId>>,
}

#[function_component(AssetsPreview)]
pub fn assets_preview(props: &AssetsPreviewProps) -> Html {
    html! {
        <div class={classes!("assets-preview")}>
            if props.assets.len() == 0 {
             { "(no assets)" }
            } else {
                <ol class={classes!("thot-ui-assets-list")}>
                    { props.assets.iter().map(|asset| html! {
                        <li key={asset.rid.clone()}
                            class={classes!("thot-ui-asset-preview", "clickable")}
                            onclick={delegate_callback(
                                asset.rid.clone(),
                                props.onclick_asset.clone()
                            )}
                            ondblclick={delegate_callback(
                                asset.rid.clone(),
                                props.ondblclick_asset.clone()
                            )} >

                            { asset_display_name(&asset) }
                        </li>
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

/// Creates a [`Callback`] that passes the [`ResourceId`] through as the only parameter.
fn delegate_callback<In: 'static + Clone, De>(input: In, cb: Option<Callback<In>>) -> Callback<De> {
    Callback::from(move |_: De| {
        if let Some(cb) = cb.as_ref() {
            cb.emit(input.clone());
        }
    })
}

#[cfg(test)]
#[path = "./assets_preview_test.rs"]
mod assets_preview_test;

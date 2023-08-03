//! Select element for Container preview.
use crate::types::ContainerPreview;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct ContainerPreviewSelectProps {
    /// Initial value.
    #[prop_or_default]
    pub value: Option<ContainerPreview>,

    /// Callback to run when the preview changes.
    #[prop_or_default]
    pub onchange: Option<Callback<ContainerPreview>>,
}

#[function_component(ContainerPreviewSelect)]
pub fn container_preview_select(props: &ContainerPreviewSelectProps) -> Html {
    let preview_ref = use_node_ref();

    let onchange = {
        let preview_ref = preview_ref.clone();
        let onchange = props.onchange.clone();

        Callback::from(move |_: web_sys::Event| {
            if let Some(onchange) = onchange.clone() {
                let preview = preview_ref
                    .cast::<web_sys::HtmlSelectElement>()
                    .expect("could not cast node ref into select");

                let p_val = preview.value().into();
                onchange.emit(p_val);
            }
        })
    };

    html! {
        <select ref={preview_ref} {onchange}>
            <option value={ContainerPreview::Assets}>{ ContainerPreview::Assets }</option>
            <option value={ContainerPreview::Type}>{ ContainerPreview::Type }</option>
            <option value={ContainerPreview::Description}>{ ContainerPreview::Description }</option>
            <option value={ContainerPreview::Tags}>{ ContainerPreview::Tags }</option>
            <option value={ContainerPreview::Metadata}>{ ContainerPreview::Metadata }</option>
            <option value={ContainerPreview::Scripts}>{ ContainerPreview::Scripts }</option>
            <option value={ContainerPreview::None}>{ ContainerPreview::None }</option>
        </select>
    }
}

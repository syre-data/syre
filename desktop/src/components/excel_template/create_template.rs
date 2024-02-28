//! Create an Excel template.
use super::excel_template_builder::ExcelTemplateBuilder;
use crate::app::PageOverlay;
use syre_core::project::ExcelTemplate;
use tauri_sys::dialog::FileDialogBuilder;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct CreateExcelTemplateProps {
    pub oncreate: Callback<ExcelTemplate>,
}

#[function_component(CreateExcelTemplate)]
pub fn create_excel_template(props: &CreateExcelTemplateProps) -> Html {
    let template_path = use_state(|| None);

    let onclose = use_callback((), {
        let template_path = template_path.setter();
        move |_e, ()| {
            template_path.set(None);
        }
    });

    let onclick = use_callback((), {
        let template_path = template_path.setter();

        move |_e, ()| {
            let template_path = template_path.clone();
            spawn_local(async move {
                let mut path = FileDialogBuilder::new();
                // path.set_default_path(&default_path); TODO Set default path
                path.set_title("Select an Excel template")
                    .add_filter("Excel", &["xlsx"]);

                let path = path.pick_file().await.unwrap();
                template_path.set(path);
            });
        }
    });

    let oncreate = use_callback(props.oncreate.clone(), {
        let template_path = template_path.setter();
        move |template, oncreate| {
            template_path.set(None);
            oncreate.emit(template);
        }
    });

    html! {
        <div>
            <button {onclick}>{ "Create Excel Template" }</button>
            if let Some(path) = (*template_path).clone() {
                <PageOverlay {onclose} >
                    <h1>{ "Create an Excel template" }</h1>
                    <ExcelTemplateBuilder {path} {oncreate} />
                </PageOverlay>
            }
        </div>
    }
}

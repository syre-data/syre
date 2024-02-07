use super::spreadsheet::Spreadsheet;
use syre_desktop_lib::excel_template;
use wasm_bindgen::JsCast;
use web_sys::HtmlDivElement;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct WorkbookProps {
    pub workbook: excel_template::Workbook,

    /// # Fields
    /// 0. Original event.
    /// 1. (sheet name, column index).
    ///
    #[prop_or_default]
    pub onclick_header: Option<Callback<(MouseEvent, (excel_template::WorksheetId, u32))>>,
}

#[function_component(Workbook)]
pub fn workbook(props: &WorkbookProps) -> Html {
    let active_sheet = use_state(|| 0);
    let spreadsheets_ref = use_node_ref();

    let set_active_worksheet = {
        let active_sheet = active_sheet.setter();
        move |index: usize| {
            let active_sheet = active_sheet.clone();
            Callback::from(move |e: MouseEvent| {
                e.stop_propagation();
                active_sheet.set(index);
            })
        }
    };

    let onclick_header = move |sheet: excel_template::WorksheetId| {
        Callback::from({
            let onclick_header = props.onclick_header.clone();
            move |(e, index): (MouseEvent, u32)| {
                e.stop_propagation();
                if let Some(onclick_header) = onclick_header.as_ref() {
                    onclick_header.emit((e, (sheet.clone(), index)));
                }
            }
        })
    };

    use_effect({
        let spreadsheets_ref = spreadsheets_ref.clone();
        move || {
            let spreadsheets_elm = spreadsheets_ref.cast::<web_sys::HtmlDivElement>().unwrap();
            let spreadsheets = spreadsheets_elm.query_selector_all(".spreadsheet").unwrap();
            let mut dimensions = (0.0, 0.0);

            for index in 0..spreadsheets.length() {
                let spreadsheet = spreadsheets.item(index).unwrap();
                let spreadsheet = spreadsheet.dyn_ref::<web_sys::Element>().unwrap();
                let bbox = spreadsheet.get_bounding_client_rect();
                let width = bbox.width();
                let height = bbox.height();

                if width > dimensions.0 {
                    dimensions.0 = width;
                }

                if height > dimensions.1 {
                    dimensions.1 = height;
                }
            }

            spreadsheets_elm
                .set_attribute(
                    "style",
                    &format!("width: {}px; height: {}px", dimensions.0, dimensions.1),
                )
                .unwrap();
        }
    });

    let (spreadsheet_names, spreadsheets): (Vec<_>, Vec<_>) = props
        .workbook
        .iter()
        .map(|worksheet| (worksheet.name(), worksheet.data()))
        .unzip();

    html! {
        <div class={"workbook"}>
            <div ref={spreadsheets_ref}
                class={"spreadsheets"}>

                {spreadsheets.into_iter().enumerate().map(|(index, spreadsheet)| {
                    let mut class = classes!("spreadsheet");
                    if index == *active_sheet {
                        class.push("active");
                    }

                    html! {
                        <div key={index}
                            {class}
                            data-index={index.to_string()}>

                            <Spreadsheet spreadsheet={spreadsheet.clone()}
                                onclick_header={onclick_header(excel_template::WorksheetId::Name(spreadsheet_names[index].clone()))} />
                        </div>
                    }
                }).collect::<Html>()}
            </div>

            <div class={"spreadsheet-tabs flex"}>
                {spreadsheet_names.into_iter().enumerate().map(|(index, name)| {
                    let mut class = classes!("spreadsheet-tab", "clickable");
                    if index == *active_sheet {
                        class.push("active");
                    }

                    html! {
                        <div key={index}
                            {class}
                            data-index={index.to_string()}
                            onclick={set_active_worksheet(index)}>
                            { name }
                        </div>
                    }
                }).collect::<Html>()}
            </div>

        </div>
    }
}

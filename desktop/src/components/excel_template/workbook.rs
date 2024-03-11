use super::spreadsheet::Spreadsheet;
use std::collections::HashMap;
use syre_core::project::excel_template::{
    CoordinateMap, Index, WorkbookCoordinateMap, WorkbookTrackMap, WorksheetId,
};
use syre_desktop_lib::excel_template;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct WorkbookProps {
    pub workbook: excel_template::Workbook,

    #[prop_or_default]
    pub row_classes: WorkbookTrackMap<Classes>,

    #[prop_or_default]
    pub column_classes: WorkbookTrackMap<Classes>,

    #[prop_or_default]
    pub cell_classes: WorkbookCoordinateMap<Classes>,

    /// # Fields
    /// 0. Original event.
    /// 1. (sheet name, column index).
    ///
    #[prop_or_default]
    pub onclick_column_label: Option<Callback<(MouseEvent, (WorksheetId, u32))>>,

    /// # Fields
    /// 0. Original event.
    /// 1. (sheet name, row index).
    ///
    #[prop_or_default]
    pub onclick_row_label: Option<Callback<(MouseEvent, (WorksheetId, u32))>>,
}

#[function_component(Workbook)]
pub fn workbook(props: &WorkbookProps) -> Html {
    let active_sheet = use_state(|| 0);

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

    let onclick_column_label = move |sheet: WorksheetId| {
        Callback::from({
            let onclick_column_label = props.onclick_column_label.clone();
            move |(e, index): (MouseEvent, u32)| {
                e.stop_propagation();
                if let Some(onclick_column_label) = onclick_column_label.as_ref() {
                    onclick_column_label.emit((e, (sheet.clone(), index)));
                }
            }
        })
    };

    let onclick_row_label = move |sheet: WorksheetId| {
        Callback::from({
            let onclick_row_label = props.onclick_column_label.clone();
            move |(e, index): (MouseEvent, u32)| {
                e.stop_propagation();
                if let Some(onclick_row_label) = onclick_row_label.as_ref() {
                    onclick_row_label.emit((e, (sheet.clone(), index)));
                }
            }
        })
    };

    let (spreadsheet_names, spreadsheets): (Vec<_>, Vec<_>) = props
        .workbook
        .iter()
        .map(|worksheet| (worksheet.name(), worksheet.data()))
        .unzip();

    html! {
        <div class={"workbook"}>
            <div class={"spreadsheets"}>
                {spreadsheets.into_iter().enumerate().map(|(index, spreadsheet)| {
                    let mut class = classes!("spreadsheet");
                    if index == *active_sheet {
                        class.push("active");
                    }

                    let mut row_classes = HashMap::new();
                    if let Some(ws_classes) = props.row_classes.get_worksheet(&WorksheetId::Index(index as Index)) {
                        row_classes.extend(ws_classes.into_iter().map(|(k, v)| (k, v.clone())));
                    };

                    if let Some(ws_classes) = props.row_classes.get_worksheet(&WorksheetId::Name(spreadsheet_names[index].to_string())) {
                        row_classes.extend(ws_classes.into_iter().map(|(k, v)| (k, v.clone())));
                    };

                    let mut column_classes = HashMap::new();
                    if let Some(ws_classes) = props.column_classes.get_worksheet(&WorksheetId::Index(index as Index)) {
                        column_classes.extend(ws_classes.into_iter().map(|(k, v)| (k, v.clone())));
                    };

                    if let Some(ws_classes) = props.column_classes.get_worksheet(&WorksheetId::Name(spreadsheet_names[index].to_string())) {
                        column_classes.extend(ws_classes.into_iter().map(|(k, v)| (k, v.clone())));
                    };

                    let mut cell_classes = CoordinateMap::new();
                    if let Some(ws_classes) = props.cell_classes.get_worksheet(&WorksheetId::Index(index as Index)) {
                        cell_classes.extend(ws_classes.iter().map(|(k, v)| (k.clone(), (*v).clone())));
                    };

                    if let Some(ws_classes) = props.cell_classes.get_worksheet(&WorksheetId::Name(spreadsheet_names[index].to_string())) {
                        cell_classes.extend(ws_classes.iter().map(|(k, v)| (k.clone(), (*v).clone())));
                    };

                    html! {
                        <div key={index}
                            {class}
                            data-index={index.to_string()}>

                            <Spreadsheet spreadsheet={spreadsheet.clone()}
                                {row_classes}
                                {column_classes}
                                {cell_classes}
                                onclick_column_label={onclick_column_label(WorksheetId::Name(spreadsheet_names[index].clone()))}
                                onclick_row_label={onclick_row_label(WorksheetId::Name(spreadsheet_names[index].clone()))} />
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

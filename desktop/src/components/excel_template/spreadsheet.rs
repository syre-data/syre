//! Spreadsheet interface.
use super::common;
use std::collections::HashMap;
use syre_core::project::excel_template::{CoordinateMap, Index};
use syre_desktop_lib::excel_template;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct SpreadsheetProps {
    pub spreadsheet: excel_template::Spreadsheet,

    #[prop_or_default]
    pub column_classes: HashMap<Index, Classes>,

    #[prop_or_default]
    pub row_classes: HashMap<Index, Classes>,

    #[prop_or_default]
    pub cell_classes: CoordinateMap<Classes>,

    #[prop_or_default]
    pub onclick_header: Option<Callback<(MouseEvent, u32)>>,
}

#[function_component(Spreadsheet)]
pub fn spreadsheet(props: &SpreadsheetProps) -> HtmlResult {
    let onclick_header = |index| {
        let onclick_header = props.onclick_header.clone();
        Callback::from(move |e: MouseEvent| {
            e.stop_propagation();
            if let Some(onclick_header) = onclick_header.as_ref() {
                onclick_header.emit((e, index));
            }
        })
    };

    let n_cols = if props.spreadsheet.len() > 0 {
        props.spreadsheet[0].len()
    } else {
        10
    };

    let headers = (0..n_cols).map(common::index_to_column).collect::<Vec<_>>();
    Ok(html! {
        <div>
            <table>
                <thead>
                    <th></th>
                    { headers
                        .iter()
                        .enumerate()
                        .map(|(index, header)| {
                            let mut class = classes!("table-label", "column-label");
                            if let Some(cell_class) = props.column_classes.get(&(index as Index)) {
                                class.push(cell_class.clone())
                            }

                            html! {
                                <th key={index}
                                    {class}
                                    data-index={(index).to_string()}
                                    onclick={onclick_header(index as u32)}>

                                    { header }
                                </th>
                            }
                        })
                        .collect::<Html>()
                    }
                </thead>
                <tbody>
                    { props.spreadsheet
                        .iter()
                        .enumerate()
                        .map(|(row_index, row)| {
                            let mut row_class = classes!("table-label", "row-label");
                            if let Some(cell_class) = props.row_classes.get(&(row_index as Index)) {
                                row_class.push(cell_class.clone());
                            }

                            html! {
                                <tr data-index={row_index.to_string()}>
                                    <th class={row_class}>{ row_index + 1 }</th>
                                    { row
                                        .iter()
                                        .enumerate()
                                        .map(|(col_index, cell_value)| {
                                            let mut class = classes!();
                                            if let Some(row_class) = props.row_classes.get(&(row_index as Index)) {
                                                class.push(row_class.clone());
                                            }

                                            if let Some(col_class) = props.column_classes.get(&(col_index as Index)) {
                                                class.push(col_class.clone());
                                            }

                                            if let Some(cell_class) = props.cell_classes.get_coordinate(&(row_index as Index), &(col_index as Index)) {
                                                class.push(cell_class.clone());
                                            }

                                            html! {
                                                <td {class}
                                                    data-row={row_index.to_string()}
                                                    data-column={col_index.to_string()}>

                                                    { cell_value }
                                                </td>
                                            }
                                        })
                                        .collect::<Html>()
                                    }
                                </tr>
                            }
                        })
                        .collect::<Html>()
                    }
                </tbody>
            </table>
        </div>
    })
}

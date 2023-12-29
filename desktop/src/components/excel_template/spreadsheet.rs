//! Spreadsheet interface.
use super::common;
use thot_desktop_lib::excel_template;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct SpreadsheetProps {
    pub spreadsheet: excel_template::Spreadsheet,

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

    let mut th_class = classes!();
    if props.onclick_header.is_some() {
        th_class.push("clickable");
    }

    Ok(html! {
        <div>
            <table>
                <thead>
                    <th></th>
                    { headers
                        .iter()
                        .enumerate()
                        .map(|(index, header)| html! {
                            <th key={index}
                                class={th_class.clone()}
                                data-index={(index).to_string()}
                                onclick={onclick_header(index as u32)}>

                                { header }
                            </th>
                        })
                        .collect::<Html>()
                    }
                </thead>
                <tbody>
                    { props.spreadsheet
                        .iter()
                        .enumerate()
                        .map(|(index, row)| { html! {
                            <tr>
                                <th>{ index + 1 }</th>
                                { row
                                    .iter()
                                    .map(|cell_value| html! { <td>{ cell_value }</td> })
                                    .collect::<Html>()
                                }
                            </tr>
                        }})
                        .collect::<Html>()
                    }
                </tbody>
            </table>
        </div>
    })
}

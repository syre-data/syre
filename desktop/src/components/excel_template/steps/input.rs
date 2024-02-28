//! Excel template input builder.
use syre_core::db::StandardSearchFilter;
use syre_core::project::excel_template::{
    DataSelection, Index, InputParameters, SpreadsheetColumns,
};
use yew::prelude::*;

const ALPHABET_LENGTH: u32 = 26;
static ALPHABET: [char; ALPHABET_LENGTH as usize] = [
    'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's',
    't', 'u', 'v', 'w', 'x', 'y', 'z',
];

#[derive(Properties, PartialEq)]
pub struct InputBuilderProps {
    pub onsubmit: Callback<InputParameters>,

    #[prop_or_default]
    pub input: Option<InputParameters>,
}

#[function_component(InputBuilder)]
pub fn input_builder(props: &InputBuilderProps) -> Html {
    let input_data_form_node_ref = use_node_ref();

    let onsubmit = use_callback(props.onsubmit.clone(), {
        let form_node_ref = input_data_form_node_ref.clone();

        move |e: SubmitEvent, onsubmit| {
            e.prevent_default();
            let form = form_node_ref.cast::<web_sys::HtmlFormElement>().unwrap();

            let form_data = web_sys::FormData::new_with_form(&form).unwrap();
            let filter_kind = form_data.get("filter-kind");
            let filter_kind = filter_kind.as_string().unwrap();
            let filter_kind = filter_kind.as_str().trim();
            let filter_kind = if filter_kind.is_empty() {
                None
            } else {
                Some(filter_kind.to_string())
            };

            let mut asset_filter = StandardSearchFilter::new();
            asset_filter.kind = Some(filter_kind);

            let data_selection = form_data.get("data-selection");
            let data_selection = data_selection.as_string().unwrap();
            let Some(data_selection) = str_to_spreadsheet_columns(&data_selection) else {
                return;
            };
            let data_selection = DataSelection::Spreadsheet(data_selection);

            let skip_rows = form_data.get("skip-rows");
            let skip_rows = skip_rows.as_string().unwrap().parse::<u32>().unwrap();

            onsubmit.emit(InputParameters {
                asset_filter,
                data_selection,
                skip_rows,
            });
        }
    });

    html! {
        <form ref={input_data_form_node_ref} {onsubmit}>
            <div>
                <label for={"filter-kind"}>{ "Which type of assets should be copied in?" }</label>
                <input placeholder={"Type"}
                    name={"filter-kind"}
                    value={props
                        .input
                        .clone()
                        .map(|input| input.asset_filter.kind)
                        .unwrap_or(None)
                        .unwrap_or(None)
                        .unwrap_or("".to_string())} />
                // TODO: Try to load example of this data
                // TODO: Output preview.
            </div>

            <div>
                <label for={"data-selection"}>{ "Which columns should be copied?" }</label>
                <input name={"data-selection"}
                    value={props.input.clone().map_or("".to_string(), |input| data_selection_to_string(&input.data_selection))} />

                <small class="form-hint">
                    { "Column labels separated by commas." }
                </small>
            </div>

            <div>
                <label for={"skip-rows"}>{ "How many rows should be skipped until the header rows or first data?" }</label>
                <input type={"number"}
                    name={"skip-rows"}
                    value={props.input.clone().map_or(0, |input| input.skip_rows).to_string()} />
            </div>

            <div>
                <button>{ "Next"}</button>
            </div>
        </form>
    }
}

fn column_index_from_str(label: impl AsRef<str>) -> Option<Index> {
    let label = label.as_ref();
    if !label.is_ascii() {
        return None;
    }

    let label = label.to_lowercase();
    let chars = label.chars().collect::<Vec<_>>();
    match chars[..] {
        [c] => ALPHABET
            .iter()
            .position(|&l| l == c)
            .map(|idx| idx as Index),
        [c1, c2] => {
            let Some(idx1) = ALPHABET.iter().position(|&l| l == c1) else {
                return None;
            };

            let Some(idx2) = ALPHABET.iter().position(|&l| l == c2) else {
                return None;
            };

            let Ok(idx) = ((idx1 + 1) * 26 + idx2).try_into() else {
                return None;
            };

            Some(idx)
        }
        _ => None,
    }
}

/// Converts an index into a column label.
/// e.g. 0 -> A, 1 -> B, 26 -> AA
///
/// # Returns
/// + `Some` if index is valid. Letters are uppercased.
/// + `None` if index exceeds maximum.
fn column_index_to_str(index: u32) -> Option<String> {
    const COLUMN_LABEL_MAX_INDEX: u32 = ALPHABET_LENGTH * ALPHABET_LENGTH;

    if index < ALPHABET_LENGTH {
        let char = ALPHABET[index as usize];
        let char = char.to_uppercase().to_string();
        return Some(char);
    } else if index < COLUMN_LABEL_MAX_INDEX {
        let i1 = index / ALPHABET_LENGTH;
        let i2 = index % ALPHABET_LENGTH;
        assert!(i1 > 0);

        let c1 = ALPHABET[(i1 - 1) as usize];
        let c2 = ALPHABET[i2 as usize];
        return Some(format!("{}{}", c1.to_uppercase(), c2.to_uppercase()));
    } else {
        return None;
    }
}

fn data_selection_to_string(selection: &DataSelection) -> String {
    match selection {
        DataSelection::Spreadsheet(SpreadsheetColumns::Indices(indices)) => indices
            .iter()
            .map(|index| column_index_to_str(index.clone()).unwrap())
            .collect::<Vec<_>>()
            .join(", "),

        DataSelection::Spreadsheet(SpreadsheetColumns::Header(headers)) => todo!(),
    }
}

/// Parses a string as spreadsheet columns.
///
/// # Notes
/// + Valid input consists of comma-separated column labels.
///     e.g. A, BA
fn str_to_spreadsheet_columns(input: &str) -> Option<SpreadsheetColumns> {
    if input.trim().is_empty() {
        return Some(SpreadsheetColumns::Indices(vec![]));
    }

    let input = input.split(",").collect::<Vec<_>>();
    let mut labels = Vec::with_capacity(input.len());
    for label in input {
        if let Some(label) = column_index_from_str(label.trim()) {
            labels.push(label);
        } else {
            return None;
        }
    }

    Some(SpreadsheetColumns::Indices(labels))
}

#[cfg(test)]
mod test {
    use crate::components::excel_template::steps::input::str_to_spreadsheet_columns;
    use syre_core::project::excel_template::SpreadsheetColumns;

    #[test]
    fn parse_data_selection_should_work() {
        // empty
        let input = "";
        let Some(output) = str_to_spreadsheet_columns(input) else {
            panic!("empty returned None");
        };

        assert_eq!(output, SpreadsheetColumns::Indices(vec![]));

        // single
        let input = "a";
        let Some(output) = str_to_spreadsheet_columns(input) else {
            panic!("single returned None");
        };

        assert_eq!(output, SpreadsheetColumns::Indices(vec![0]));

        // multiple
        let input = "a, b, c";
        let Some(output) = str_to_spreadsheet_columns(input) else {
            panic!("multiple returned None");
        };

        assert_eq!(output, SpreadsheetColumns::Indices(vec![0, 1, 2]));

        // invalid
        assert!(str_to_spreadsheet_columns("0").is_none());
        assert!(str_to_spreadsheet_columns("abc").is_none());
    }
}

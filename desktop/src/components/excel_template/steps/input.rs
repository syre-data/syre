//! Excel template input builder.
use syre_core::db::StandardSearchFilter;
use syre_core::project::excel_template::{
    DataSelection, Index, InputParameters, SpreadsheetColumns,
};
use yew::prelude::*;

static ALPHABET: [char; 26] = [
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
            let data_selection = data_selection.as_str().split(",").collect::<Vec<_>>();

            let data_selection_as_indices = data_selection
                .iter()
                .map(|selector| selector.parse::<u32>())
                .collect::<Vec<_>>();

            let data_selection = if data_selection_as_indices
                .iter()
                .all(|index_result| index_result.is_ok())
            {
                SpreadsheetColumns::Indices(
                    data_selection_as_indices
                        .into_iter()
                        .map(|index_result| index_result.unwrap())
                        .collect(),
                )
            } else {
                let data_selection = data_selection
                    .iter()
                    .filter_map(|label| column_index_from_str(label))
                    .collect::<Vec<_>>();

                SpreadsheetColumns::Indices(data_selection)
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
                    { "Either indices or labels separated by commas." }
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

fn data_selection_to_string(selection: &DataSelection) -> String {
    match selection {
        DataSelection::Spreadsheet(SpreadsheetColumns::Indices(indices)) => indices
            .iter()
            .map(|index| index.to_string())
            .collect::<Vec<_>>()
            .join(", "),

        DataSelection::Spreadsheet(SpreadsheetColumns::Header(headers)) => todo!(),
    }
}

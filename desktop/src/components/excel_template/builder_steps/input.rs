//! Excel template input builder.
use super::super::common;
use syre_core::db::StandardSearchFilter;
use syre_core::project::excel_template::{
    DataSelection, Index, InputParameters, SpreadsheetColumns, WorksheetId,
};
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct InputBuilderProps {
    pub onsubmit: Callback<InputParameters>,

    #[prop_or_default]
    pub input: Option<InputParameters>,

    #[prop_or_default]
    pub step: Step,
}

#[function_component(InputBuilder)]
pub fn input_builder(props: &InputBuilderProps) -> Html {
    let builder = use_reducer_eq(|| {
        if let Some(input) = props.input.as_ref() {
            Builder::from(input.clone())
        } else {
            Builder::new()
        }
    });

    let input_data_form_node_ref = use_node_ref();
    let data_format_type_node_ref = use_node_ref();
    let filter_node_ref = use_node_ref();
    let filter_kind_node_ref = use_node_ref();
    let data_format_input_node_ref = use_node_ref();

    use_effect_with(props.step.clone(), {
        let builder = builder.dispatcher();
        move |step| {
            builder.dispatch(BuilderAction::SetStep(step.clone()));
        }
    });

    use_effect_with(props.input.clone(), {
        let builder = builder.dispatcher();
        move |input| {
            if let Some(input) = input.as_ref() {
                builder.dispatch(BuilderAction::SetInput(input.clone()));
            } else {
                builder.dispatch(BuilderAction::ClearInput);
            }
        }
    });

    use_effect_with(builder.step.clone(), {
        let data_format_type_node_ref = data_format_type_node_ref.clone();
        let filter_node_ref = filter_node_ref.clone();
        let data_format_input_node_ref = data_format_input_node_ref.clone();

        move |step| {
            let node_ref = match step {
                Step::DataFormat => data_format_type_node_ref,
                Step::AssetFilter => filter_node_ref,
                Step::DataFormatInput => data_format_input_node_ref,
            };

            let elm = node_ref
                .cast::<web_sys::HtmlElement>()
                .expect("could not cast node ref as element");

            elm.scroll_into_view();
        }
    });

    let set_step = |step: Step| {
        let builder = builder.dispatcher();
        Callback::from(move |_: MouseEvent| {
            builder.dispatch(BuilderAction::SetStep(step.clone()));
        })
    };

    let onchange_filter_kind = use_callback((), {
        let builder = builder.dispatcher();
        let filter_kind_node_ref = filter_kind_node_ref.clone();
        move |_, _| {
            let filter_kind_elm = filter_kind_node_ref
                .cast::<web_sys::HtmlInputElement>()
                .expect("could not cast node ref as input");

            let value = filter_kind_elm.value();
            tracing::debug!(?value);
            let value = if value.is_empty() { None } else { Some(value) };
            builder.dispatch(BuilderAction::SetAssetFilterKind(value));
        }
    });

    let onsubmit = use_callback((props.onsubmit.clone(), builder.clone()), {
        let form_node_ref = input_data_form_node_ref.clone();

        move |e: SubmitEvent, (onsubmit, builder)| {
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

            let Some(data_format_type) = builder.input.data_selection.as_ref() else {
                return;
            };

            let data_selection = match data_format_type {
                DataSelectionBuilder::Spreadsheet { .. } => {
                    let columns = form_data.get("column-selection");
                    let columns = columns.as_string().unwrap();
                    let Some(columns) = common::str_to_spreadsheet_columns(&columns) else {
                        return;
                    };

                    let skip_rows = form_data.get("skip-rows");
                    let skip_rows = skip_rows.as_string().unwrap();
                    let skip_rows = skip_rows.parse().unwrap();

                    let comment = form_data.get("comment-character");
                    let comment = comment.as_string().unwrap();
                    let comment = comment.trim().chars().collect::<Vec<_>>();
                    let comment = match comment[..] {
                        [] => None,
                        [c] => Some(c.clone()),
                        _ => return,
                    };

                    DataSelection::Spreadsheet {
                        columns,
                        skip_rows,
                        comment,
                    }
                }

                DataSelectionBuilder::Excel { .. } => {
                    let sheet = form_data.get("data-sheet");
                    let sheet = sheet.as_string().unwrap();
                    let sheet = common::worksheet_id_from_str(sheet);

                    let columns = form_data.get("column-selection");
                    let columns = columns.as_string().unwrap();
                    let Some(columns) = common::str_to_spreadsheet_columns(&columns) else {
                        return;
                    };

                    let skip_rows = form_data.get("skip-rows");
                    let skip_rows = skip_rows.as_string().unwrap();
                    let skip_rows = skip_rows.parse().unwrap();

                    DataSelection::ExcelWorkbook {
                        sheet,
                        columns,
                        skip_rows,
                    }
                }
            };

            onsubmit.emit(InputParameters {
                asset_filter,
                data_selection,
            });
        }
    });

    let mut filter_kind = &None;
    if let Some(filter) = &builder.input.asset_filter.kind.as_ref() {
        filter_kind = filter;
    }

    html! {
        <form ref={input_data_form_node_ref}
            class={"builder-steps"}
            {onsubmit}>

            <div ref={data_format_type_node_ref}
                class={"form-step"}>

                <fieldset>
                    <legend>{ "What type of data will be ingested?" }</legend>

                    <div>
                        <input type={"radio"}
                            id={"data-format-type-spreadsheet"}
                            name={"data-format-type"}
                            value={"spreadsheet"}
                            checked={matches!(
                                builder.input.data_selection,
                                Some(DataSelectionBuilder::Spreadsheet {..})
                            )}
                            onchange={
                                let builder = builder.dispatcher();
                                move |_| builder
                                            .dispatch(
                                                BuilderAction::SetDataSelection(
                                                    DataSelectionBuilder::new_spreadsheet()
                                                )
                                            )
                            } />

                        <label for={"data-format-type-spreadsheet"}>
                            { "Spreadsheet" }
                        </label>
                    </div>

                    <div>
                        <input type={"radio"}
                            id={"data-format-type-excel"}
                            name={"data-format-type"}
                            value={"excel"}
                            checked={matches!(
                                builder.input.data_selection,
                                Some(DataSelectionBuilder::Excel {..})
                            )}
                            onchange={
                                let builder = builder.dispatcher();
                                move |_| builder
                                            .dispatch(
                                                BuilderAction::SetDataSelection(
                                                    DataSelectionBuilder::new_excel()
                                                )
                                            )
                            } />


                        <label for={"data-format-type-excel"}>
                            { "Excel" }
                        </label>
                    </div>
                </fieldset>

                <div class={"step-controls"}>
                    <button type={"button"}
                        onclick={set_step(Step::AssetFilter)}>

                        { "Next" }
                    </button>
                </div>
            </div>

            <div ref={filter_node_ref}
                class={"form-step"}>

                <div>
                    <label for={"filter-kind"}>{ "Which type of assets should be copied in?" }</label>
                    <input ref={filter_kind_node_ref}
                        name={"filter-kind"}
                        value={filter_kind.clone().unwrap_or("".to_string())}
                        placeholder={"Type"}
                        onchange={onchange_filter_kind} />
                    // TODO: Try to load example of this data
                    // TODO: Output preview.
                </div>

                <div class={"step-controls"}>
                    <button type={"button"}
                        onclick={set_step(Step::DataFormat)}>

                        { "Back" }
                    </button>
                    <button type={"button"}
                        onclick={set_step(Step::DataFormatInput)}>

                        { "Next" }
                    </button>
                </div>
            </div>

            <div ref={data_format_input_node_ref}
                class={"form-step"}>

                if let Some(data_selection) = builder.input.data_selection.as_ref() {
                    { match data_selection {
                        DataSelectionBuilder::Spreadsheet {
                            columns,
                            skip_rows,
                            comment,
                        } => html! {
                            <SpreadsheetInput
                                builder={builder.dispatcher()}
                                columns={columns.clone()}
                                skip_rows={skip_rows.clone()}
                                comment={comment.clone()} />
                        },

                        DataSelectionBuilder::Excel {
                            sheet,
                            columns,
                            skip_rows,
                        } => html! {
                            <ExcelInput
                                builder={builder.dispatcher()}
                                sheet={sheet.clone()}
                                columns={columns.clone()}
                                skip_rows={skip_rows.clone()} />
                        }
                    }}

                    <div class={"step-controls"}>
                        <button type={"button"}
                            onclick={set_step(Step::AssetFilter)}>

                            { "Back" }
                        </button>
                        <button type={"submit"}>{ "Next" }</button>
                    </div>
                } else {
                    { "Must set data format type" }
                    <div class={"step-controls"}>
                        <button type={"button"}
                            onclick={set_step(Step::DataFormat)}>

                            { "Set data format type" }
                        </button>
                    </div>
                }
            </div>
        </form>
    }
}

#[derive(PartialEq, Properties)]
struct SpreadsheetInputProps {
    builder: UseReducerDispatcher<Builder>,

    #[prop_or_default]
    columns: Option<SpreadsheetColumns>,

    #[prop_or_default]
    skip_rows: u32,

    #[prop_or_default]
    comment: Option<char>,
}

#[function_component(SpreadsheetInput)]
fn spreadsheet_input(props: &SpreadsheetInputProps) -> Html {
    let onchange_columns = use_callback(props.builder.clone(), move |_, builder| {});

    html! {
        <>
        <div>
            <div class={"label-wrapper"}>
                <label for={"column-selection"}>{ "Which columns should be copied?" }</label>
                <small class="form-hint">
                    { "Columns separated by commas." }
                </small>
            </div>
            <input name={"column-selection"}
                value={props
                        .columns
                        .clone()
                        .map(|columns| common::spreadsheet_columns_to_string(&columns))
                        .unwrap_or("".to_string())}
                onchange={onchange_columns} />
        </div>

        <div>
            <label for={"skip-rows"}>{ "How many rows should be skipped until the header rows or first data?" }</label>
            <input type={"number"}
                name={"skip-rows"}
                value={props.skip_rows.to_string()} />
        </div>

        <div>
            <div class={"label-wrapper"}>
                <label>{ "Is there a comment character?" }</label>
                <small>{ "Lines beginning with a comment character are ignored." }</small>
            </div>
            <input name={"comment-character"}
                value={props
                        .comment
                        .clone()
                        .map(|char| char.to_string()).unwrap_or("".to_string())}
                placeholder={"(no comment character)"}
                maxlength={"1"} />
        </div>
        </>
    }
}

#[derive(PartialEq, Properties)]
struct ExcelInputProps {
    builder: UseReducerDispatcher<Builder>,

    #[prop_or_default]
    sheet: Option<WorksheetId>,

    #[prop_or_default]
    columns: Option<SpreadsheetColumns>,

    #[prop_or_default]
    skip_rows: u32,
}

#[function_component(ExcelInput)]
fn excel_input(props: &ExcelInputProps) -> Html {
    html! {
        <>
        <div>
            <input name={"data-sheet"}
                value={props
                        .sheet
                        .clone()
                        .map(|sheet| common::worksheet_id_to_string(&sheet))
                        .unwrap_or("".to_string())}
                placeholder={"Spreadsheet id"} />
        </div>

        <div>
            <div class={"label-wrapper"}>
                <label for={"column-selection"}>{ "Which columns should be copied?" }</label>
                <small class="form-hint">
                    { "Columns separated by commas." }
                </small>
            </div>

            <input name={"column-selection"}
                value={props
                        .columns
                        .clone()
                        .map(|columns| common::spreadsheet_columns_to_string(&columns))
                        .unwrap_or("".to_string())} />

        </div>

        <div>
            <label for={"skip-rows"}>{ "How many rows should be skipped until the header rows or first data?" }</label>
            <input type={"number"}
                name={"skip-rows"}
                value={props.skip_rows.to_string()} />
        </div>
        </>
    }
}

#[derive(PartialEq, Clone, Debug)]
pub enum Step {
    DataFormat,
    AssetFilter,
    DataFormatInput,
}

impl Default for Step {
    fn default() -> Self {
        Self::DataFormat
    }
}

enum BuilderAction {
    SetStep(Step),
    SetInput(InputParameters),
    ClearInput,
    SetAssetFilter(StandardSearchFilter),
    SetAssetFilterKind(Option<String>),
    SetDataSelection(DataSelectionBuilder),
}

#[derive(PartialEq, Clone, Debug)]
struct Builder {
    pub step: Step,
    pub input: InputParametersBuilder,
}

impl Builder {
    pub fn new() -> Self {
        Self {
            step: Step::default(),
            input: InputParametersBuilder::new(),
        }
    }

    pub fn from(input: InputParameters) -> Self {
        Self {
            step: Step::default(),
            input: InputParametersBuilder::from(input),
        }
    }
}

impl Reducible for Builder {
    type Action = BuilderAction;
    fn reduce(self: std::rc::Rc<Self>, action: Self::Action) -> std::rc::Rc<Self> {
        let mut current = (*self).clone();
        match action {
            BuilderAction::SetStep(step) => current.step = step,
            BuilderAction::SetInput(input) => current.input = InputParametersBuilder::from(input),
            BuilderAction::ClearInput => current.input = InputParametersBuilder::new(),
            BuilderAction::SetAssetFilter(filter) => current.input.asset_filter = filter,
            BuilderAction::SetAssetFilterKind(value) => {
                current.input.asset_filter.kind = Some(value)
            }
            BuilderAction::SetDataSelection(data_selection) => {
                current.input.data_selection = Some(data_selection)
            }
        }
        current.into()
    }
}

#[derive(PartialEq, Clone, Debug)]
struct InputParametersBuilder {
    pub asset_filter: StandardSearchFilter,
    pub data_selection: Option<DataSelectionBuilder>,
}

impl InputParametersBuilder {
    pub fn new() -> Self {
        Self {
            asset_filter: StandardSearchFilter::new(),
            data_selection: None,
        }
    }

    pub fn from(
        InputParameters {
            asset_filter,
            data_selection,
        }: InputParameters,
    ) -> Self {
        Self {
            asset_filter,
            data_selection: Some(DataSelectionBuilder::from(data_selection)),
        }
    }
}

#[derive(PartialEq, Clone, Debug)]
enum DataSelectionBuilder {
    Spreadsheet {
        columns: Option<SpreadsheetColumns>,
        skip_rows: u32,
        comment: Option<char>,
    },

    Excel {
        sheet: Option<WorksheetId>,
        columns: Option<SpreadsheetColumns>,
        skip_rows: u32,
    },
}

impl DataSelectionBuilder {
    pub fn new_spreadsheet() -> Self {
        Self::Spreadsheet {
            columns: None,
            skip_rows: 0,
            comment: None,
        }
    }

    pub fn new_excel() -> Self {
        Self::Excel {
            sheet: None,
            columns: None,
            skip_rows: 0,
        }
    }

    pub fn from(data_selection: DataSelection) -> Self {
        match data_selection {
            DataSelection::Spreadsheet {
                columns,
                skip_rows,
                comment,
            } => Self::Spreadsheet {
                columns: Some(columns),
                skip_rows,
                comment,
            },

            DataSelection::ExcelWorkbook {
                sheet,
                columns,
                skip_rows,
            } => Self::Excel {
                sheet: Some(sheet),
                columns: Some(columns),
                skip_rows,
            },
        }
    }
}

//! Excel template builder.
use super::{common, workbook::Workbook};
use crate::hooks::spreadsheet::use_excel;
use std::path::PathBuf;
use std::rc::Rc;
use syre_core::db::StandardSearchFilter;
use syre_core::project::AssetProperties;
use syre_desktop_lib::excel_template;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct ExcelTemplateBuilderProps {
    pub template_path: PathBuf,
    pub oncreate: Callback<excel_template::ExcelTemplate>,
}

#[function_component(ExcelTemplateBuilder)]
pub fn excel_template_builder(props: &ExcelTemplateBuilderProps) -> Html {
    let builder = use_reducer(|| TemplateBuilder::new(props.template_path.clone()));
    let template_replace_column_start: UseStateHandle<Option<u32>> = use_state(|| None);
    let template_form_node_ref = use_node_ref();
    let input_data_form_node_ref = use_node_ref();
    let output_asset_form_node_ref = use_node_ref();

    let onclick_header_template =
        use_callback((builder.clone(), template_replace_column_start.clone()), {
            move |(_e, (sheet, index)), (builder, template_replace_column_start)| {
                match template_replace_column_start.as_ref() {
                    None => {
                        builder.dispatch(TemplateBuilderAction::ClearTemplateReplaceRange);
                        template_replace_column_start.set(Some(index));
                    }
                    Some(&temp_index) => {
                        let start = index.min(temp_index);
                        let end = index.max(temp_index);

                        template_replace_column_start.set(None);
                        builder.dispatch(TemplateBuilderAction::SetTemplateReplaceRange {
                            sheet,
                            range: excel_template::Range { start, end },
                        });

                        builder.dispatch(TemplateBuilderAction::NextStep);
                    }
                }
            }
        });

    let onsubmit_template = use_callback((), {
        let builder = builder.dispatcher();
        let template_form_node_ref = template_form_node_ref.clone();

        move |e: SubmitEvent, _| {
            e.prevent_default();
            let form = template_form_node_ref
                .cast::<web_sys::HtmlFormElement>()
                .unwrap();

            let form_data = web_sys::FormData::new_with_form(&form).unwrap();
            let data_label_action = form_data.get("data-label-action");
            let data_label_action = match data_label_action.as_string().unwrap().as_str() {
                "none" => excel_template::DataLabelAction::None,
                "insert" => excel_template::DataLabelAction::Insert,
                "replace" => excel_template::DataLabelAction::Replace,
                other => panic!("unknown data label action value `{other}`"),
            };

            builder.dispatch(TemplateBuilderAction::SetTemplateDataLabelAction(
                data_label_action,
            ));

            builder.dispatch(TemplateBuilderAction::NextStep);
        }
    });

    let onsubmit_input_data_parameters = use_callback((), {
        let builder = builder.dispatcher();
        let input_data_form_node_ref = input_data_form_node_ref.clone();

        move |e: SubmitEvent, _| {
            e.prevent_default();
            let form = input_data_form_node_ref
                .cast::<web_sys::HtmlFormElement>()
                .unwrap();

            let form_data = web_sys::FormData::new_with_form(&form).unwrap();
            let filter_kind = form_data.get("input-filter-kind");
            let filter_kind = filter_kind.as_string().unwrap();
            let filter_kind = filter_kind.as_str().trim();
            let filter_kind = if filter_kind.is_empty() {
                None
            } else {
                Some(filter_kind.to_string())
            };

            let mut asset_filter = StandardSearchFilter::new();
            asset_filter.kind = Some(filter_kind);

            let data_selection = form_data.get("input-data-selection");
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
                excel_template::SpreadsheetColumns::Indices(
                    data_selection_as_indices
                        .into_iter()
                        .map(|index_result| index_result.unwrap())
                        .collect(),
                )
            } else {
                excel_template::SpreadsheetColumns::Names(
                    data_selection
                        .into_iter()
                        .map(|name| name.to_string())
                        .collect(),
                )
            };

            let data_selection = excel_template::DataSelection::Spreadsheet(data_selection);

            let skip_rows = form_data.get("input-skip-rows");
            let skip_rows = skip_rows.as_f64().unwrap() as u32;

            let input_params = excel_template::InputDataParameters {
                asset_filter,
                data_selection,
                skip_rows,
            };

            builder.dispatch(TemplateBuilderAction::SetInputDataParamters(input_params));
            builder.dispatch(TemplateBuilderAction::NextStep);
        }
    });

    let onsubmit_output_asset = use_callback((), {
        let builder = builder.dispatcher();
        let output_asset_form_node_ref = output_asset_form_node_ref.clone();
        move |e: SubmitEvent, _| {
            e.prevent_default();
            let form = output_asset_form_node_ref
                .cast::<web_sys::HtmlFormElement>()
                .unwrap();

            let form_data = web_sys::FormData::new_with_form(&form).unwrap();
            let name = form_data.get("name").as_string().unwrap();
            let name = name.as_str().trim();
            let name = if name.is_empty() {
                None
            } else {
                Some(name.to_string())
            };

            let kind = form_data.get("kind").as_string().unwrap();
            let kind = kind.as_str().trim();
            let kind = if kind.is_empty() {
                None
            } else {
                Some(kind.to_string())
            };

            let tags = form_data.get("tags").as_string().unwrap();
            let tags = tags.as_str().trim();
            let tags = if tags.is_empty() {
                Vec::new()
            } else {
                tags.split(",")
                    .filter_map(|tag| {
                        let tag = tag.trim();
                        if tag.is_empty() {
                            None
                        } else {
                            Some(tag.to_string())
                        }
                    })
                    .collect::<Vec<_>>()
            };

            let description = form_data.get("description").as_string().unwrap();
            let description = description.as_str().trim();
            let description = if description.is_empty() {
                None
            } else {
                Some(description.to_string())
            };

            let mut output_asset = AssetProperties::new();
            output_asset.name = name;
            output_asset.kind = kind;
            output_asset.tags = tags;
            output_asset.description = description;

            builder.dispatch(TemplateBuilderAction::SetOutputAssetproperties(
                output_asset,
            ));

            builder.dispatch(TemplateBuilderAction::NextStep);
        }
    });

    let create_template = use_callback(
        (builder.clone(), props.oncreate.clone()),
        move |e: MouseEvent, (builder, oncreate)| {
            e.stop_propagation();

            let Ok(template_params) = builder.template_params.clone().try_into() else {
                builder.dispatch(TemplateBuilderAction::SetStep(BuilderStep::Template));
                return;
            };

            let Some(input_data_params) = builder.input_data_params.as_ref() else {
                builder.dispatch(TemplateBuilderAction::SetStep(BuilderStep::InputData));
                return;
            };

            let Some(output_asset) = builder.output_asset.as_ref() else {
                builder.dispatch(TemplateBuilderAction::SetStep(BuilderStep::OutputAsset));
                return;
            };

            let template = excel_template::ExcelTemplate {
                input_data_params: input_data_params.clone(),
                template_params,
                output_asset: output_asset.clone(),
            };

            oncreate.emit(template);
        },
    );

    let mut template_step_class = classes!("builder-step", "excel-template", "flex");
    let mut input_step_class = classes!("builder-step", "input-data");
    let mut output_step_class = classes!("builder-step", "output-asset");
    let mut review_step_class = classes!("builder-step", "review");

    match builder.step {
        BuilderStep::Template => template_step_class.push("active"),
        BuilderStep::InputData => input_step_class.push("active"),
        BuilderStep::OutputAsset => output_step_class.push("active"),
        BuilderStep::Review => review_step_class.push("active"),
    }

    let template_fallback = html! {
        { "Loading template..." }
    };

    html! {
        <div class={"excel-template-builder"}>
            <div class={template_step_class}>
                <Suspense fallback={template_fallback}>
                    <div>
                        <ExcelWorkbook path={props.template_path.clone()}
                            onclick_header={onclick_header_template} />
                    </div>
                    <form ref={template_form_node_ref} onsubmit={onsubmit_template}>
                        <div>
                            <p>
                                { "Select the column range that should be replaced with new data" }
                            </p>
                            <p>
                                <label>
                                    {"Columns to replace"}
                                    <input value={builder.template_params.range_as_value()} disabled={true} />
                                </label>
                            </p>
                        </div>

                        <div>
                            <fieldset>
                                <legend>{ "How should data be inserted?" }</legend>
                                <div>
                                    <input type={"radio"}
                                        name={"data-label-action"}
                                        value={"none"}
                                        checked={builder.template_params.data_label_action == excel_template::DataLabelAction::None} />

                                    <label for={"none"}
                                        title={"Data will be inserted as is from the input source."}>
                                        { "None" }
                                    </label>
                                </div>

                                <div>
                                    <input type={"radio"}
                                        name={"data-label-action"}
                                        value={"insert"}
                                        checked={builder.template_params.data_label_action == excel_template::DataLabelAction::Insert} />

                                    <label for={"insert"}
                                        title={"Input asset's path will be appended as a header."}>
                                        { "Append file name" }
                                    </label>
                                </div>

                                <div>
                                    <input type={"radio"}
                                        name={"data-label-action"}
                                        value={"replace"}
                                        checked={builder.template_params.data_label_action == excel_template::DataLabelAction::Replace} />

                                    <label for={"replace"}
                                        title={"Input asset's path will replace any headers."}>
                                        { "Replace headers" }
                                    </label>
                                </div>
                            </fieldset>
                        </div>

                        <div>
                            <button>{ "Next" }</button>
                        </div>
                    </form>
                </Suspense>
            </div>

            <div class={input_step_class}>
                <form ref={input_data_form_node_ref} onsubmit={onsubmit_input_data_parameters}>
                    <div>
                        <label for={"input-filter-kind"}>{ "Which type of assets should be copied in?" }</label>
                        <input placeholder={"Type"} name={"input-filter-kind"} />
                        // TODO: Try to load example of this data
                        // TODO: Output preview.
                    </div>

                    <div>
                        <label for={"input-data-selection"}>{ "Which columns should be copied?" }</label>
                        <input name={"input-data-selection"} value={builder.input_data_selection_string()} />
                        <small class="form-hint">
                            { "Either indices or labels separated by commas." }
                        </small>
                    </div>

                    <div>
                        <label for={"input-skip-rows"}>{ "How many rows should be skipped until the header rows or first data?" }</label>
                        <input type={"number"} name={"skip-rows"} value={builder.input_data_skip_rows().unwrap_or(&0).to_string()} />
                    </div>

                    <div>
                        <button>{ "Next"}</button>
                    </div>
                </form>
            </div>

            <div class={output_step_class}>
                <form ref={output_asset_form_node_ref} onsubmit={onsubmit_output_asset}>
                    <div>
                        <input name={"name"} placeholder={"name"} />
                    </div>
                    <div>
                        <input name={"type"} placeholder={"type"} />
                    </div>
                    <div>
                        <input name={"tags"} placeholder={"tags"} />
                    </div>
                    <div>
                        <textarea name={"description"} placeholder={"Description"}></textarea>
                    </div>
                    <div>
                        <button>{ "Next" }</button>
                    </div>
                </form>
            </div>

            <div class={review_step_class}>
                {"Review"}
                <button onclick={create_template}>{ "Create template" }</button>
            </div>
        </div>
    }
}

#[derive(Properties, PartialEq)]
struct ExcelWorkbookProps {
    pub path: PathBuf,

    #[prop_or_default]
    pub onclick_header: Option<Callback<(MouseEvent, (excel_template::WorksheetId, u32))>>,
}

#[function_component(ExcelWorkbook)]
fn excel_workbook(props: &ExcelWorkbookProps) -> HtmlResult {
    let workbook = use_excel(props.path.clone())?;
    Ok(html! {
        <Workbook {workbook} onclick_header={props.onclick_header.clone()} />
    })
}

#[derive(PartialEq, Clone, Default, Debug)]
pub enum BuilderStep {
    #[default]
    Template,
    InputData,
    OutputAsset,
    Review,
}

pub enum TemplateBuilderAction {
    NextStep,
    SetStep(BuilderStep),

    SetTemplateReplaceRange {
        sheet: excel_template::WorksheetId,
        range: excel_template::Range,
    },

    ClearTemplateReplaceRange,

    SetTemplateDataLabelAction(excel_template::DataLabelAction),

    SetInputDataParamters(excel_template::InputDataParameters),

    SetOutputAssetproperties(AssetProperties),
}

#[derive(PartialEq, Clone, Debug)]
struct TemplateBuilder {
    pub step: BuilderStep,
    pub template_params: TemplateParamsBuilder,
    pub input_data_params: Option<excel_template::InputDataParameters>,
    pub output_asset: Option<AssetProperties>,
}

impl TemplateBuilder {
    pub fn new(template_path: PathBuf) -> Self {
        Self {
            step: BuilderStep::default(),
            template_params: TemplateParamsBuilder::new(template_path),
            input_data_params: None,
            output_asset: None,
        }
    }

    pub fn input_data_selection_string(&self) -> String {
        let Some(input_data) = self.input_data_params.as_ref() else {
            return "".to_string();
        };

        match &input_data.data_selection {
            excel_template::DataSelection::Spreadsheet(
                excel_template::SpreadsheetColumns::Indices(indices),
            ) => indices
                .iter()
                .map(|index| index.to_string())
                .collect::<Vec<_>>()
                .join(", "),

            excel_template::DataSelection::Spreadsheet(
                excel_template::SpreadsheetColumns::Names(names),
            ) => names.join(", "),
        }
    }

    pub fn input_data_skip_rows(&self) -> Option<&u32> {
        let Some(input_data) = self.input_data_params.as_ref() else {
            return None;
        };

        Some(&input_data.skip_rows)
    }
}

impl Reducible for TemplateBuilder {
    type Action = TemplateBuilderAction;
    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        let mut current = (*self).clone();
        match action {
            TemplateBuilderAction::NextStep => {
                current.step = match current.step {
                    BuilderStep::Template => BuilderStep::InputData,
                    BuilderStep::InputData => BuilderStep::OutputAsset,
                    BuilderStep::OutputAsset => BuilderStep::Review,
                    BuilderStep::Review => panic!("no next step"),
                }
            }

            TemplateBuilderAction::SetStep(step) => current.step = step,

            TemplateBuilderAction::SetTemplateReplaceRange { sheet, range } => {
                current.template_params.sheet = Some(sheet);
                current.template_params.range = Some(range);
            }

            TemplateBuilderAction::ClearTemplateReplaceRange => {
                current.template_params.sheet = None;
                current.template_params.range = None;
            }

            TemplateBuilderAction::SetTemplateDataLabelAction(action) => {
                current.template_params.data_label_action = action;
            }

            TemplateBuilderAction::SetInputDataParamters(params) => {
                current.input_data_params = Some(params);
            }

            TemplateBuilderAction::SetOutputAssetproperties(asset_props) => {
                current.output_asset = Some(asset_props);
            }
        }

        current.into()
    }
}

#[derive(PartialEq, Clone, Debug)]
struct TemplateParamsBuilder {
    pub path: PathBuf,
    pub sheet: Option<excel_template::WorksheetId>,
    pub range: Option<excel_template::Range>,
    pub data_label_action: excel_template::DataLabelAction,
}

impl TemplateParamsBuilder {
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            sheet: None,
            range: None,
            data_label_action: excel_template::DataLabelAction::None,
        }
    }

    pub fn range_as_value(&self) -> String {
        let Some(sheet) = self.sheet.as_ref() else {
            return "".into();
        };

        let Some(range) = self.range.as_ref() else {
            return "".into();
        };

        let col_start = common::index_to_column(range.start as usize);
        let col_end = common::index_to_column(range.end as usize);

        match sheet {
            excel_template::WorksheetId::Name(sheet_name) => {
                format!("{sheet_name}!{col_start}:{col_end}")
            }

            excel_template::WorksheetId::Index(sheet_index) => {
                format!("[{}]!{col_start}:{col_end}", sheet_index + 1)
            }
        }
    }
}

impl TryInto<excel_template::ExcelTemplateParameters> for TemplateParamsBuilder {
    type Error = TemplateParamsBuilderError;
    fn try_into(self) -> Result<excel_template::ExcelTemplateParameters, Self::Error> {
        let Some(worksheet) = self.sheet else {
            return Err(TemplateParamsBuilderError::Incomplete);
        };

        let Some(range) = self.range else {
            return Err(TemplateParamsBuilderError::Incomplete);
        };

        Ok(excel_template::ExcelTemplateParameters {
            path: self.path,
            replace_range: excel_template::WorkbookRange { worksheet, range },
            data_label_action: self.data_label_action,
        })
    }
}

enum TemplateParamsBuilderError {
    Incomplete,
}

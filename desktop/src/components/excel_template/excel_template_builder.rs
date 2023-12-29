//! Excel template builder.
use super::{common, workbook::Workbook};
use crate::hooks::spreadsheet::use_excel;
use std::path::PathBuf;
use std::rc::Rc;
use thot_core::db::StandardSearchFilter;
use thot_core::project::AssetProperties;
use thot_desktop_lib::excel_template;
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::spawn_local;
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

    let onclick_header_template =
        use_callback((builder.clone(), template_replace_column_start.clone()), {
            move |(e, (sheet, index)), (builder, template_replace_column_start)| {
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
                            range: (start, end),
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

    let mut template_step_class = classes!("builder-step", "excel-template", "d-flex");
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

    SetTemplateReplaceRange {
        sheet: excel_template::WorksheetId,
        range: (u32, u32),
    },

    ClearTemplateReplaceRange,

    SetTemplateDataLabelAction(excel_template::DataLabelAction),
}

#[derive(PartialEq, Clone, Debug)]
struct TemplateBuilder {
    pub step: BuilderStep,
    pub template_params: TemplateParamsBuilder,
    pub input_data_params: InputDataParamsBuilder,
}

impl TemplateBuilder {
    pub fn new(template_path: PathBuf) -> Self {
        Self {
            step: BuilderStep::default(),
            template_params: TemplateParamsBuilder::new(template_path),
            input_data_params: InputDataParamsBuilder::new(),
        }
    }
}

impl Reducible for TemplateBuilder {
    type Action = TemplateBuilderAction;
    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        let mut current = (*self).clone();
        match action {
            TemplateBuilderAction::NextStep => {
                current.step = match current.step {
                    BuilderStep::Template => {
                        if !current.input_data_params.is_complete() {
                            BuilderStep::InputData
                        } else {
                            BuilderStep::Review
                        }
                    }

                    BuilderStep::InputData => BuilderStep::Review,

                    BuilderStep::OutputAsset => BuilderStep::Review,

                    BuilderStep::Review => BuilderStep::Review,
                }
            }

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
        }

        current.into()
    }
}

#[derive(PartialEq, Clone, Debug)]
struct TemplateParamsBuilder {
    pub path: PathBuf,
    pub sheet: Option<excel_template::WorksheetId>,
    pub range: Option<(u32, u32)>,
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

    pub fn is_complete(&self) -> bool {
        self.sheet.is_some() && self.range.is_some()
    }

    pub fn range_as_value(&self) -> String {
        let Some(sheet) = self.sheet.as_ref() else {
            return "".into();
        };

        let Some(range) = self.range.as_ref() else {
            return "".into();
        };

        let col_start = common::index_to_column(range.0 as usize);
        let col_end = common::index_to_column(range.1 as usize);

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

#[derive(PartialEq, Clone, Debug, Default)]
struct InputDataParamsBuilder {
    data: Option<InputRange>,
    headers: Option<u32>,
}

impl InputDataParamsBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_complete(&self) -> bool {
        return false;
    }
}

#[derive(PartialEq, Clone, Debug)]
pub enum InputRange {
    Specified(excel_template::Range),
    UntilBreak(excel_template::Range),
}

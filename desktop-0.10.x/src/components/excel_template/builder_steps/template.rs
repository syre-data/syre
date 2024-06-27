//! Excel template builder.
use super::super::{common, workbook::Workbook};
use crate::hooks::spreadsheet::use_excel;
use std::path::PathBuf;
use std::rc::Rc;
use syre_core::project::excel_template::{
    utils as excel_utils, DataLabelAction, Index, Range, TemplateParameters, WorkbookCoordinateMap,
    WorkbookRange, WorkbookTrackMap, WorksheetId,
};
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct TemplateBuilderProps {
    pub path: PathBuf,
    pub onsubmit: Callback<TemplateParameters>,

    #[prop_or_default]
    pub template: Option<TemplateParameters>,

    #[prop_or_default]
    pub step: Option<Step>,
}

#[function_component(TemplateBuilder)]
pub fn template_builder(props: &TemplateBuilderProps) -> Html {
    let builder = use_reducer(|| {
        let mut builder = if let Some(template) = props.template.as_ref() {
            TemplateBuilderState::from_template(template.clone())
        } else {
            TemplateBuilderState::new()
        };

        if let Some(step) = props.step.as_ref() {
            builder.step = step.clone();
        }

        builder
    });

    let template_form_node_ref = use_node_ref();
    let replace_range_node_ref = use_node_ref();
    let data_label_action_node_ref = use_node_ref();

    use_effect_with(props.template.clone(), {
        let builder = builder.dispatcher();
        move |template| {
            if let Some(template) = template {
                builder.dispatch(TemplateBuilderAction::Set(template.clone()));
            } else {
                builder.dispatch(TemplateBuilderAction::Clear);
            }
        }
    });

    use_effect_with(props.step.clone(), {
        let builder = builder.dispatcher();
        move |step| {
            let step = step.clone().unwrap_or(Step::ReplaceRange);
            builder.dispatch(TemplateBuilderAction::SetStep(step));
        }
    });

    use_effect_with(builder.step.clone(), {
        let replace_range_node_ref = replace_range_node_ref.clone();
        let data_label_action_node_ref = data_label_action_node_ref.clone();

        move |step| {
            let node_ref = match step {
                Step::ReplaceRange => replace_range_node_ref,
                Step::DataLabelAction => data_label_action_node_ref,
            };

            let elm = node_ref
                .cast::<web_sys::HtmlElement>()
                .expect("could not cast node ref as element");

            elm.scroll_into_view();
        }
    });

    let set_step = |step: Step| {
        let builder = builder.dispatcher();
        Callback::from(move |e: MouseEvent| {
            e.prevent_default();
            builder.dispatch(TemplateBuilderAction::SetStep(step.clone()));
        })
    };

    let onclick_column_label = use_callback(
        builder.clone(),
        move |(_e, (sheet, index)), builder| match builder.replace_range.as_ref() {
            Some(WorkbookRangeKind::Open(WorkbookOpenRange {
                sheet: set_sheet,
                start,
            })) if set_sheet == &sheet => {
                builder.dispatch(TemplateBuilderAction::SetReplaceRange {
                    sheet,
                    start: start.clone(),
                    end: index,
                });
            }

            _ => {
                builder.dispatch(TemplateBuilderAction::SetOpenReplaceRange {
                    sheet,
                    start: index,
                });
            }
        },
    );

    let onsubmit = use_callback(
        (props.onsubmit.clone(), props.path.clone(), builder.clone()),
        {
            let template_form_node_ref = template_form_node_ref.clone();

            move |e: SubmitEvent, (onsubmit, path, builder)| {
                e.prevent_default();
                let Some(WorkbookRangeKind::Closed(replace_range)) = builder.replace_range.as_ref()
                else {
                    builder.dispatch(TemplateBuilderAction::SetStep(Step::ReplaceRange));
                    return;
                };

                let form = template_form_node_ref
                    .cast::<web_sys::HtmlFormElement>()
                    .unwrap();

                let form_data = web_sys::FormData::new_with_form(&form).unwrap();
                let data_label_action = form_data.get("data-label-action");
                let data_label_action = match data_label_action.as_string().unwrap().as_str() {
                    "none" => DataLabelAction::None,
                    "insert" => DataLabelAction::Insert,
                    "replace" => DataLabelAction::Replace,
                    other => panic!("unknown data label action value `{other}`"),
                };

                onsubmit.emit(TemplateParameters {
                    path: path.clone(),
                    replace_range: replace_range.clone(),
                    data_label_action,
                });
            }
        },
    );

    let mut column_classes = WorkbookTrackMap::new();
    if let Some(WorkbookRangeKind::Closed(WorkbookRange {
        sheet,
        range: Range { start, end },
    })) = builder.replace_range.as_ref()
    {
        for col in *start..=*end {
            column_classes.insert_for(sheet.clone(), col, classes!("replace-range"));
        }
    }

    let mut replace_range_class = classes!("form-step");
    let mut headers_class = classes!("form-step");
    let mut data_label_action_class = classes!("form-step");
    match builder.step {
        Step::ReplaceRange => {
            replace_range_class.push("active");
        }

        Step::DataLabelAction => {
            data_label_action_class.push("active");
        }
    }

    let template_fallback = html! {
        { "Loading template..." }
    };

    html! {
        <Suspense fallback={template_fallback}>
            <div>
                <ExcelWorkbook path={props.path.clone()}
                    {column_classes}
                    {onclick_column_label} />
            </div>
            <form ref={template_form_node_ref}
                class={"builder-steps pl-xl"}
                {onsubmit}>

                <div ref={replace_range_node_ref}
                    class={replace_range_class}>

                    <p>
                        { "Select the column range that should be replaced with new data" }
                    </p>
                    <p>
                        <label>
                            {"Columns to replace"}
                            <input value={builder.range_as_value()} disabled={true} />
                        </label>
                    </p>
                    <div class={"step-controls"}>
                        <button onclick={set_step(Step::DataLabelAction)}>{ "Next" }</button>
                    </div>
                </div>

                <div ref={data_label_action_node_ref}
                    class={data_label_action_class}>

                    <div>
                        <fieldset class={"form-control"}>
                            <legend>{ "How should inserted data be labeled?" }</legend>
                            <div>
                                <input type={"radio"}
                                    name={"data-label-action"}
                                    value={"none"}
                                    checked={builder.data_label_action == DataLabelAction::None} />

                                <label for={"none"}
                                    title={"Data will be inserted as is from the input."}>
                                    { "No label" }
                                </label>
                            </div>

                            <div>
                                <input type={"radio"}
                                    name={"data-label-action"}
                                    value={"insert"}
                                    checked={builder.data_label_action == DataLabelAction::Insert} />

                                <label for={"insert"}
                                    title={"File path will be appended as a header."}>
                                    { "Append file path" }
                                </label>
                            </div>

                            <div>
                                <input type={"radio"}
                                    name={"data-label-action"}
                                    value={"replace"}
                                    checked={builder.data_label_action == DataLabelAction::Replace} />

                                <label for={"replace"}
                                    title={"Input asset's path will replace any headers."}>
                                    { "Replace headers" }
                                </label>
                            </div>
                        </fieldset>
                    </div>

                    <div class={"step-controls"}>
                        <button onclick={set_step(Step::ReplaceRange)}>{ "Previous" }</button>
                        <button>{ "Next" }</button>
                    </div>
                </div>
            </form>
        </Suspense>
    }
}

#[derive(Properties, PartialEq)]
struct ExcelWorkbookProps {
    pub path: PathBuf,

    #[prop_or_default]
    pub row_classes: WorkbookTrackMap<Classes>,

    #[prop_or_default]
    pub column_classes: WorkbookTrackMap<Classes>,

    #[prop_or_default]
    pub cell_classes: WorkbookCoordinateMap<Classes>,

    #[prop_or_default]
    pub onclick_column_label: Option<Callback<(MouseEvent, (WorksheetId, u32))>>,

    #[prop_or_default]
    pub onclick_row_label: Option<Callback<(MouseEvent, (WorksheetId, u32))>>,
}

#[function_component(ExcelWorkbook)]
fn excel_workbook(props: &ExcelWorkbookProps) -> HtmlResult {
    let workbook = use_excel(props.path.clone())?;
    Ok(html! {
        <Workbook {workbook}
            row_classes={props.row_classes.clone()}
            column_classes={props.column_classes.clone()}
            cell_classes={props.cell_classes.clone()}
            onclick_column_label={props.onclick_column_label.clone()}
            onclick_row_label={props.onclick_row_label.clone()} />
    })
}

#[derive(PartialEq, Clone, Debug)]
pub enum Step {
    ReplaceRange,
    DataLabelAction,
}

#[derive(PartialEq, Clone, Debug)]
struct WorkbookOpenRange {
    pub sheet: WorksheetId,
    pub start: Index,
}

#[derive(PartialEq, Clone, Debug)]
enum WorkbookRangeKind {
    Open(WorkbookOpenRange),
    Closed(WorkbookRange),
}

enum TemplateBuilderAction {
    Set(TemplateParameters),
    Clear,
    SetStep(Step),
    SetOpenReplaceRange {
        sheet: WorksheetId,
        start: Index,
    },

    SetReplaceRange {
        sheet: WorksheetId,
        start: Index,
        end: Index,
    },
}

#[derive(PartialEq, Clone, Debug)]
struct TemplateBuilderState {
    pub step: Step,
    pub replace_range: Option<WorkbookRangeKind>,
    pub data_label_action: DataLabelAction,
    pub index_columns: Vec<Index>,
}

impl TemplateBuilderState {
    pub fn new() -> Self {
        Self {
            step: Step::ReplaceRange,
            replace_range: None,
            data_label_action: DataLabelAction::None,
            index_columns: Vec::new(),
        }
    }

    pub fn from_template(template: TemplateParameters) -> Self {
        let TemplateParameters {
            path: _,
            replace_range,
            data_label_action,
        } = template;

        Self {
            step: Step::ReplaceRange,
            replace_range: Some(WorkbookRangeKind::Closed(replace_range)),
            data_label_action,
            index_columns: Vec::new(),
        }
    }

    pub fn range_as_value(&self) -> String {
        let Some(WorkbookRangeKind::Closed(WorkbookRange {
            sheet,
            range: Range { start, end },
        })) = self.replace_range.as_ref()
        else {
            return "".into();
        };

        let col_start = excel_utils::index_to_column(*start as usize);
        let col_end = excel_utils::index_to_column(*end as usize);
        match sheet {
            WorksheetId::Name(sheet_name) => {
                format!("{sheet_name}!{col_start}:{col_end}")
            }

            WorksheetId::Index(sheet_index) => {
                format!("[{}]!{col_start}:{col_end}", sheet_index + 1)
            }
        }
    }
}

impl Reducible for TemplateBuilderState {
    type Action = TemplateBuilderAction;
    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        let mut current = (*self).clone();
        match action {
            TemplateBuilderAction::Set(template) => {
                let mut current = Self::from_template(template);
                current.step = self.step.clone();
                return current.into();
            }

            TemplateBuilderAction::Clear => {
                let mut current = Self::new();
                current.step = self.step.clone();
                return current.into();
            }

            TemplateBuilderAction::SetStep(step) => {
                current.step = step;
            }

            TemplateBuilderAction::SetOpenReplaceRange { sheet, start } => {
                current.replace_range =
                    Some(WorkbookRangeKind::Open(WorkbookOpenRange { sheet, start }));
            }

            TemplateBuilderAction::SetReplaceRange { sheet, start, end } => {
                current.replace_range = Some(WorkbookRangeKind::Closed(WorkbookRange {
                    sheet,
                    range: Range { start, end },
                }));
            }
        }

        current.into()
    }
}

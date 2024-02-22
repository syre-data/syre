//! Excel template builder.
use super::super::{common, workbook::Workbook};
use crate::hooks::spreadsheet::use_excel;
use std::path::PathBuf;
use std::rc::Rc;
use syre_core::project::excel_template::Index;
use syre_core::project::excel_template::{
    DataLabelAction, Range, TemplateParameters, WorkbookCoordinateMap, WorkbookRange,
    WorkbookTrackMap, WorksheetId,
};
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct TemplateBuilderProps {
    pub path: PathBuf,
    pub onsubmit: Callback<TemplateParameters>,

    #[prop_or_default]
    pub template: Option<TemplateParameters>,
}

#[function_component(TemplateBuilder)]
pub fn template_builder(props: &TemplateBuilderProps) -> Html {
    let builder = use_reducer(|| {
        if let Some(template) = props.template.as_ref() {
            TemplateBuilderState::from_template(template.clone())
        } else {
            TemplateBuilderState::new()
        }
    });
    let data_label_action_state = use_state(|| DataLabelAction::None);
    let template_form_node_ref = use_node_ref();
    let data_label_action_none_node_ref = use_node_ref();
    let data_label_action_insert_node_ref = use_node_ref();
    let data_label_action_replace_node_ref = use_node_ref();
    let template_index_columns_node_ref = use_node_ref();

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

    let onclick_header =
        use_callback(
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

    let onchange_data_label_action = use_callback((), {
        let data_label_action_state = data_label_action_state.setter();
        let none_node_ref = data_label_action_none_node_ref.clone();
        let insert_node_ref = data_label_action_insert_node_ref.clone();
        let replace_node_ref = data_label_action_replace_node_ref.clone();

        move |_, _| {
            let none_elm = none_node_ref
                .cast::<web_sys::HtmlInputElement>()
                .expect("could not cast node ref as input");

            let insert_elm = insert_node_ref
                .cast::<web_sys::HtmlInputElement>()
                .expect("could not cast node ref as input");

            let replace_elm = replace_node_ref
                .cast::<web_sys::HtmlInputElement>()
                .expect("could not cast node ref as input");

            if none_elm.checked() {
                data_label_action_state.set(DataLabelAction::None);
            } else if insert_elm.checked() {
                data_label_action_state.set(DataLabelAction::Insert { index: Vec::new() });
            } else if replace_elm.checked() {
                data_label_action_state.set(DataLabelAction::Replace);
            } else {
                data_label_action_state.set(DataLabelAction::None);
            }
        }
    });

    let onsubmit = use_callback(
        (props.onsubmit.clone(), props.path.clone(), builder.clone()),
        {
            let template_form_node_ref = template_form_node_ref.clone();
            let index_cols_node_ref = template_index_columns_node_ref.clone();

            move |e: SubmitEvent, (onsubmit, path, builder)| {
                e.prevent_default();

                let Some(WorkbookRangeKind::Closed(replace_range)) = builder.replace_range.as_ref()
                else {
                    return;
                };

                let form = template_form_node_ref
                    .cast::<web_sys::HtmlFormElement>()
                    .unwrap();

                let form_data = web_sys::FormData::new_with_form(&form).unwrap();
                let data_label_action = form_data.get("data-label-action");
                let data_label_action = match data_label_action.as_string().unwrap().as_str() {
                    "none" => DataLabelAction::None,
                    "insert" => {
                        let index_cols_elm = index_cols_node_ref
                            .cast::<web_sys::HtmlInputElement>()
                            .unwrap();

                        let index = index_cols_elm
                            .value()
                            .split(",")
                            .filter_map(|col| col.trim().parse::<u32>().ok())
                            .collect::<Vec<_>>();

                        DataLabelAction::Insert { index }
                    }
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
        for idx in *start..=*end {
            column_classes.insert_for(sheet.clone(), idx, classes!("replace-range"));
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
                    {onclick_header} />
            </div>
            <form ref={template_form_node_ref} {onsubmit}>
                <div>
                    <p>
                        { "Select the column range that should be replaced with new data" }
                    </p>
                    <p>
                        <label>
                            {"Columns to replace"}
                            <input value={builder.range_as_value()} disabled={true} />
                        </label>
                    </p>
                </div>

                <div>
                    <fieldset>
                        <legend>{ "How should inserted data be labeled?" }</legend>
                        <div>
                            <input ref={data_label_action_none_node_ref}
                                type={"radio"}
                                name={"data-label-action"}
                                value={"none"}
                                checked={builder.data_label_action == DataLabelAction::None}
                                onchange={onchange_data_label_action.clone()} />

                            <label for={"none"}
                                title={"Data will be inserted as is from the input."}>
                                { "No label" }
                            </label>
                        </div>

                        <div>
                            <input ref={data_label_action_insert_node_ref}
                                type={"radio"}
                                name={"data-label-action"}
                                value={"insert"}
                                checked={matches!(&builder.data_label_action, &DataLabelAction::Insert { index: _ })}
                                onchange={onchange_data_label_action.clone()} />

                            <label for={"insert"}
                                title={"File path will be appended as a header."}>
                                { "Append file path" }
                            </label>
                        </div>

                        <div>
                            <input ref={data_label_action_replace_node_ref}
                                type={"radio"}
                                name={"data-label-action"}
                                value={"replace"}
                                checked={&builder.data_label_action == &DataLabelAction::Replace}
                                onchange={onchange_data_label_action} />

                            <label for={"replace"}
                                title={"Input asset's path will replace any headers."}>
                                { "Replace headers" }
                            </label>
                        </div>
                    </fieldset>
                </div>

                if let DataLabelAction::Insert { index } = &builder.data_label_action {
                    <div>
                        <p>
                            { "Select the index columns in the template." }
                        </p>
                        <p>
                            <label>
                                {"Template index columns"}
                                <input ref={template_index_columns_node_ref}
                                    name={"template_index_columns"}
                                    value={index
                                            .iter()
                                            .map(|idx| idx.to_string())
                                            .collect::<Vec<_>>()
                                            .join(", ")} />
                            </label>
                        </p>
                    </div>
                }

                <div>
                    <button>{ "Next" }</button>
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
    pub onclick_header: Option<Callback<(MouseEvent, (WorksheetId, u32))>>,
}

#[function_component(ExcelWorkbook)]
fn excel_workbook(props: &ExcelWorkbookProps) -> HtmlResult {
    let workbook = use_excel(props.path.clone())?;
    Ok(html! {
        <Workbook {workbook}
            row_classes={props.row_classes.clone()}
            column_classes={props.column_classes.clone()}
            cell_classes={props.cell_classes.clone()}
            onclick_header={props.onclick_header.clone()} />
    })
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

    SetOpenReplaceRange {
        sheet: WorksheetId,
        start: Index,
    },

    SetReplaceRange {
        sheet: WorksheetId,
        start: Index,
        end: Index,
    },

    SetDataLabelAction(DataLabelAction),
}

#[derive(PartialEq, Clone, Debug)]
struct TemplateBuilderState {
    pub replace_range: Option<WorkbookRangeKind>,
    pub header_rows: u32,
    pub data_label_action: DataLabelAction,
    pub index_columns: Vec<Index>,
}

impl TemplateBuilderState {
    pub fn new() -> Self {
        Self {
            replace_range: None,
            header_rows: 0,
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
            replace_range: Some(WorkbookRangeKind::Closed(replace_range)),
            header_rows: 0,
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

        let col_start = common::index_to_column(*start as usize);
        let col_end = common::index_to_column(*end as usize);

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
                return Self::from_template(template).into();
            }

            TemplateBuilderAction::Clear => {
                return Self::new().into();
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

            TemplateBuilderAction::SetDataLabelAction(action) => {
                current.data_label_action = action;
            }
        }

        current.into()
    }
}

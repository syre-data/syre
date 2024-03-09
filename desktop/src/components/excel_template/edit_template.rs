//! Create an Excel template.
use super::common;
use crate::app::projects_state::ProjectsStateReducer;
use crate::app::PageOverlay;
use crate::components::canvas::canvas_state::CanvasStateReducer;
use std::path::PathBuf;
use syre_core::db::StandardSearchFilter;
use syre_core::project::excel_template::{
    DataLabelAction, DataSelection, ExcelTemplate, InputParameters, OutputParameters,
    SpreadsheetColumns, TemplateParameters, WorkbookRange, WorksheetId,
};
use syre_core::project::AssetProperties;
use syre_core::types::ResourceId;
use syre_local::types::AnalysisKind;
use syre_ui::widgets::TagsEditor;
use wasm_bindgen::JsCast;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct ExcelTemplateEditorProps {
    pub template: ResourceId,

    /// Called anytime a property of the template changes.
    pub onchange: Callback<ExcelTemplate>,

    pub onclose: Callback<MouseEvent>,
}

#[function_component(ExcelTemplateEditor)]
pub fn excel_template_editor(props: &ExcelTemplateEditorProps) -> Html {
    let projects_state = use_context::<ProjectsStateReducer>().unwrap();
    let canvas_state = use_context::<CanvasStateReducer>().unwrap();

    let Some(analyses) = projects_state.project_analyses.get(&canvas_state.project) else {
        tracing::error!("Project not found in state.");
        return html! {
            { "Project not found" }
        };
    };

    let Some(analysis) = analyses.get(&props.template) else {
        tracing::error!("Template not found in project analyses");
        return html! {
            { "Template not found" }
        };
    };

    let template = match analysis {
        AnalysisKind::ExcelTemplate(template) => template,
        AnalysisKind::Script(_) => {
            tracing::error!("Expected template, but found Script");
            return html! {
                { "Invalid analysis kind" }
            };
        }
    };

    html! {
        <EditorView template={template.clone()}
            onchange={props.onchange.clone()}
            onclose={props.onclose.clone()} />
    }
}

#[derive(Properties, PartialEq)]
pub struct EditorViewProps {
    pub template: ExcelTemplate,

    /// Called anytime a property of the template changes.
    pub onchange: Callback<ExcelTemplate>,

    pub onclose: Callback<MouseEvent>,
}

#[function_component(EditorView)]
pub fn editor_view(props: &EditorViewProps) -> Html {
    let step_state = use_state(|| Step::Template);

    let onsubmit = use_callback((), move |e: SubmitEvent, _| {
        e.prevent_default();
    });

    let onchange_template = use_callback((props.template.clone(), props.onchange.clone()), {
        move |parameters, (template, onchange)| {
            onchange.emit(ExcelTemplate {
                template: parameters,
                ..template.clone()
            });
        }
    });

    let onchange_input = use_callback((props.template.clone(), props.onchange.clone()), {
        move |parameters, (template, onchange)| {
            onchange.emit(ExcelTemplate {
                input: parameters,
                ..template.clone()
            });
        }
    });

    let onchange_output = use_callback((props.template.clone(), props.onchange.clone()), {
        move |parameters, (template, onchange)| {
            onchange.emit(ExcelTemplate {
                output: parameters,
                ..template.clone()
            });
        }
    });

    let mut template_class = classes!();
    let mut input_class = classes!();
    let mut output_class = classes!();
    match *step_state {
        Step::Template => template_class.push("active"),
        Step::Input => input_class.push("active"),
        Step::Output => output_class.push("active"),
    }

    html! {
        <PageOverlay onclose={props.onclose.clone()} >
            <div class={"excel-template-editor flex"}>
                <div class={"sidebar-nav"}>
                    <nav>
                        <ol>
                            <li class={classes!("clickable", template_class.clone())}
                                onclick={
                                    let step_state = step_state.setter();
                                    move |_| step_state.set(Step::Template)
                                }>

                                { "Template" }
                            </li>
                            <li class={classes!("clickable", input_class.clone())}
                                onclick={
                                    let step_state = step_state.setter();
                                    move |_| step_state.set(Step::Input)
                                }>

                                { "Input" }
                            </li>
                            <li class={classes!("clickable", output_class.clone())}
                                onclick={
                                    let step_state = step_state.setter();
                                    move |_| step_state.set(Step::Output)
                                }>

                                { "Output" }
                            </li>
                        </ol>
                    </nav>
                </div>

                <div class={"editor-content grow"}>
                    <form {onsubmit}>
                        <TemplateEditor class={classes!("form-step", template_class)}
                            parameters={props.template.template.clone()}
                            onchange={onchange_template} />

                        <InputEditor class={classes!("form-step", input_class)}
                            parameters={props.template.input.clone()}
                            onchange={onchange_input} />

                        <OutputEditor class={classes!("form-step", output_class)}
                            parameters={props.template.output.clone()}
                            onchange={onchange_output} />
                    </form>
                </div>
            </div>
        </PageOverlay>
    }
}

enum Step {
    Template,
    Input,
    Output,
}

#[derive(Properties, PartialEq)]
struct TemplateEditorProps {
    #[prop_or_default]
    class: Classes,
    parameters: TemplateParameters,
    onchange: Callback<TemplateParameters>,
}

#[function_component(TemplateEditor)]
fn template_editor(props: &TemplateEditorProps) -> Html {
    let onchange_replace_range = use_callback(
        (props.parameters.clone(), props.onchange.clone()),
        move |e: Event, (parameters, onchange)| {
            let elm = e.target().unwrap();
            let elm = elm.dyn_ref::<web_sys::HtmlInputElement>().unwrap();
            assert_eq!(elm.name(), "replace-range");

            let Ok(replace_range) = WorkbookRange::try_from(elm.value()) else {
                return;
            };

            onchange.emit(TemplateParameters {
                replace_range,
                ..parameters.clone()
            })
        },
    );

    html! {
        <div class={props.class.clone()}>
            <div class={"form-control"}>
                <label for={"replace-range"}>{ "Replace range" }</label>
                <input name={"replace-range"}
                    onchange={onchange_replace_range}
                    value={Into::<String>::into(props.parameters.replace_range.clone())} />
            </div>

            <div class={"form-control"}>
                <fieldset>
                    <legend>{ "How should inserted data be labeled?" }</legend>
                    <div>
                        <input type={"radio"}
                            name={"data-label-action"}
                            value={"none"}
                            checked={props.parameters.data_label_action == DataLabelAction::None}
                            onchange={
                                let onchange = props.onchange.clone();
                                let parameters = props.parameters.clone();
                                move |_| {
                                    onchange.emit(TemplateParameters{
                                        data_label_action: DataLabelAction::None,
                                        ..parameters.clone()
                                    });
                                }
                            } />

                        <label for={"none"}
                            title={"Data will be inserted as is from the input."}>
                            { "No label" }
                        </label>
                    </div>

                    <div>
                        <input type={"radio"}
                            name={"data-label-action"}
                            value={"insert"}
                            checked={props.parameters.data_label_action == DataLabelAction::Insert}
                            onchange={
                                let onchange = props.onchange.clone();
                                let parameters = props.parameters.clone();
                                move |_| {
                                    onchange.emit(TemplateParameters{
                                        data_label_action: DataLabelAction::Insert,
                                        ..parameters.clone()
                                    });
                                }
                            }/>

                        <label for={"insert"}
                            title={"File path will be appended as a header."}>
                            { "Append file path" }
                        </label>
                    </div>

                    <div>
                        <input type={"radio"}
                            name={"data-label-action"}
                            value={"replace"}
                            checked={props.parameters.data_label_action == DataLabelAction::Replace}
                            onchange={
                                let onchange = props.onchange.clone();
                                let parameters = props.parameters.clone();
                                move |_| {
                                    onchange.emit(TemplateParameters{
                                        data_label_action: DataLabelAction::Replace,
                                        ..parameters.clone()
                                    });
                                }
                            }/>

                        <label for={"replace"}
                            title={"Input asset's path will replace any headers."}>
                            { "Replace headers" }
                        </label>
                    </div>
                </fieldset>
            </div>
        </div>
    }
}

#[derive(Properties, PartialEq)]
struct InputEditorProps {
    #[prop_or_default]
    class: Classes,
    parameters: InputParameters,
    onchange: Callback<InputParameters>,
}

#[function_component(InputEditor)]
fn input_editor(props: &InputEditorProps) -> Html {
    let onchange_filter_kind = use_callback(
        (props.parameters.clone(), props.onchange.clone()),
        move |e: Event, (parameters, onchange)| {
            let elm = e.target().unwrap();
            let elm = elm.dyn_ref::<web_sys::HtmlInputElement>().unwrap();
            assert_eq!(elm.name(), "filter-kind");

            let value = elm.value();
            let kind = if value.is_empty() {
                None
            } else {
                Some(Some(value))
            };

            let asset_filter = StandardSearchFilter {
                kind,
                ..parameters.asset_filter.clone()
            };

            onchange.emit(InputParameters {
                asset_filter,
                ..parameters.clone()
            });
        },
    );

    let onchange_data_selection = use_callback(
        (props.parameters.clone(), props.onchange.clone()),
        move |data_selection, (parameters, onchange)| {
            onchange.emit(InputParameters {
                data_selection,
                ..parameters.clone()
            });
        },
    );

    let mut filter_kind = &None;
    if let Some(filter) = &props.parameters.asset_filter.kind {
        filter_kind = filter;
    }

    html! {
        <div class={props.class.clone()}>
            <div class={"form-control"}>
                <label for={"filter-kind"}>{ "Which type of assets should be copied in?" }</label>
                <input name={"filter-kind"}
                    value={filter_kind.clone().unwrap_or("".to_string())}
                    placeholder={"Type"}
                    onchange={onchange_filter_kind} />
                // TODO: Try to load example of this data
                // TODO: Output preview.
            </div>

            {match &props.parameters.data_selection {
                DataSelection::Spreadsheet {columns, skip_rows, comment} => html! {
                    <SpreadsheetInput
                        columns={columns.clone()}
                        {skip_rows}
                        {comment}
                        onchange={onchange_data_selection}/>
                },

                DataSelection::ExcelWorkbook {sheet, columns, skip_rows} => html! {
                    <ExcelInput
                        sheet={sheet.clone()}
                        columns={columns.clone()}
                        {skip_rows}
                        onchange={onchange_data_selection} />
                },
            }}
        </div>
    }
}

#[derive(PartialEq, Properties)]
struct SpreadsheetInputProps {
    columns: SpreadsheetColumns,
    skip_rows: u32,
    comment: Option<char>,
    onchange: Callback<DataSelection>,
}

#[function_component(SpreadsheetInput)]
fn spreadsheet_input(props: &SpreadsheetInputProps) -> Html {
    let onchange = use_callback(
        (
            props.columns.clone(),
            props.skip_rows.clone(),
            props.comment.clone(),
            props.onchange.clone(),
        ),
        move |e: Event, (columns, skip_rows, comment, onchange)| {
            let elm = e.target().unwrap();
            let elm = elm.dyn_ref::<web_sys::HtmlInputElement>().unwrap();

            match elm.name().as_str() {
                "column-selection" => {
                    let Some(columns) = common::str_to_spreadsheet_columns(elm.value()) else {
                        return;
                    };

                    onchange.emit(DataSelection::Spreadsheet {
                        columns,
                        skip_rows: skip_rows.clone(),
                        comment: comment.clone(),
                    });
                }

                "skip-rows" => {
                    let Ok(skip_rows) = elm.value().parse() else {
                        return;
                    };

                    onchange.emit(DataSelection::Spreadsheet {
                        columns: columns.clone(),
                        skip_rows,
                        comment: comment.clone(),
                    });
                }

                "comment-character" => {
                    let chars = elm.value().chars().collect::<Vec<_>>();
                    let comment = match chars[..] {
                        [] => None,
                        [c] => Some(c),
                        _ => return,
                    };

                    onchange.emit(DataSelection::Spreadsheet {
                        columns: columns.clone(),
                        skip_rows: skip_rows.clone(),
                        comment,
                    });
                }

                _ => tracing::error!("invalid input name"),
            }
        },
    );

    html! {
        <>
        <div class={"form-control"}>
            <div class={"label-wrapper"}>
                <label for={"column-selection"}>{ "Which columns should be copied?" }</label>
                <small class="form-hint">
                    { "Columns separated by commas." }
                </small>
            </div>
            <input name={"column-selection"}
                value={common::spreadsheet_columns_to_string(&props.columns)}
                onchange={onchange.clone()} />
        </div>

        <div class={"form-control"}>
            <label for={"skip-rows"}>{ "How many rows should be skipped until the header rows or first data?" }</label>
            <input type={"number"}
                name={"skip-rows"}
                value={props.skip_rows.to_string()} />
        </div>

        <div class={"form-control"}>
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
    sheet: WorksheetId,
    columns: SpreadsheetColumns,
    skip_rows: u32,
    onchange: Callback<DataSelection>,
}

#[function_component(ExcelInput)]
fn excel_input(props: &ExcelInputProps) -> Html {
    let onchange = use_callback(
        (
            props.sheet.clone(),
            props.columns.clone(),
            props.skip_rows.clone(),
            props.onchange.clone(),
        ),
        move |e: Event, (sheet, columns, skip_rows, onchange)| {
            let elm = e.target().unwrap();
            let elm = elm.dyn_ref::<web_sys::HtmlInputElement>().unwrap();

            match elm.name().as_str() {
                "data-sheet" => {
                    let sheet = common::worksheet_id_from_str(elm.value());
                    onchange.emit(DataSelection::ExcelWorkbook {
                        sheet,
                        columns: columns.clone(),
                        skip_rows: skip_rows.clone(),
                    });
                }

                "column-selection" => {
                    let Some(columns) = common::str_to_spreadsheet_columns(elm.value()) else {
                        return;
                    };

                    onchange.emit(DataSelection::ExcelWorkbook {
                        sheet: sheet.clone(),
                        columns,
                        skip_rows: skip_rows.clone(),
                    });
                }

                "skip-rows" => {
                    let Ok(skip_rows) = elm.value().parse() else {
                        return;
                    };

                    onchange.emit(DataSelection::ExcelWorkbook {
                        sheet: sheet.clone(),
                        columns: columns.clone(),
                        skip_rows,
                    });
                }

                _ => tracing::error!("invalid input name"),
            }
        },
    );
    html! {
        <>
        <div class={"form-control"}>
            <label for={"data-sheet"}>{ "Spreadsheet id" }</label>
            <input name={"data-sheet"}
                value={common::worksheet_id_to_string(&props.sheet)}
                onchange={onchange.clone()} />
        </div>

        <div class={"form-control"}>
            <div class={"label-wrapper"}>
                <label for={"column-selection"}>{ "Which columns should be copied?" }</label>
                <small class="form-hint">
                    { "Columns separated by commas." }
                </small>
            </div>

            <input name={"column-selection"}
                value={common::spreadsheet_columns_to_string(&props.columns)}
                onchange={onchange.clone()} />
        </div>

        <div class={"form-control"}>
            <label for={"skip-rows"}>{ "How many rows should be skipped until the header rows or first data?" }</label>
            <input type={"number"}
                name={"skip-rows"}
                value={props.skip_rows.to_string()}
                {onchange} />
        </div>
        </>
    }
}

#[derive(Properties, PartialEq)]
struct OutputEditorProps {
    #[prop_or_default]
    class: Classes,
    parameters: OutputParameters,
    onchange: Callback<OutputParameters>,
}

#[function_component(OutputEditor)]
fn output_editor(props: &OutputEditorProps) -> Html {
    let onchange = use_callback(
        (props.parameters.clone(), props.onchange.clone()),
        move |e: Event, (parameters, onchange)| {
            let elm = e.target().unwrap();
            let elm = elm.dyn_ref::<web_sys::HtmlInputElement>().unwrap();
            match elm.name().as_str() {
                "path" => {
                    let path = elm.value();
                    if path.is_empty() {
                        return;
                    };

                    let path = PathBuf::from(path);
                    onchange.emit(OutputParameters {
                        path,
                        properties: parameters.properties.clone(),
                    });
                }

                "name" => {
                    let value = elm.value();
                    let name = if value.is_empty() { None } else { Some(value) };
                    let mut properties = parameters.properties.clone();
                    properties.name = name;
                    onchange.emit(OutputParameters {
                        path: parameters.path.clone(),
                        properties,
                    });
                }

                "kind" => {
                    let value = elm.value();
                    let kind = if value.is_empty() { None } else { Some(value) };
                    let mut properties = parameters.properties.clone();
                    properties.kind = kind;
                    onchange.emit(OutputParameters {
                        path: parameters.path.clone(),
                        properties,
                    });
                }

                name => {
                    tracing::error!("invalid input name {name}");
                    return;
                }
            }
        },
    );

    let onchange_tags = use_callback(
        (props.parameters.clone(), props.onchange.clone()),
        move |tags: Vec<String>, (parameters, onchange)| {
            let mut properties = parameters.properties.clone();
            properties.tags = tags;
            onchange.emit(OutputParameters {
                path: parameters.path.clone(),
                properties,
            });
        },
    );

    let onchange_description = use_callback(
        (props.parameters.clone(), props.onchange.clone()),
        move |e: Event, (parameters, onchange)| {
            let elm = e.target().unwrap();
            let elm = elm.dyn_ref::<web_sys::HtmlTextAreaElement>().unwrap();
            let value = elm.value();
            let description = if value.is_empty() { None } else { Some(value) };

            let mut properties = parameters.properties.clone();
            properties.description = description;
            onchange.emit(OutputParameters {
                path: parameters.path.clone(),
                properties,
            });
        },
    );

    let OutputParameters {
        path,
        properties:
            AssetProperties {
                name,
                kind,
                description,
                tags,
                ..
            },
    } = &props.parameters;

    html! {
        <div class={props.class.clone()}>
            <div class={"flex form-control"}>
                <input name={"path"}
                    placeholder={"File name"}
                    value={path.with_extension("").to_string_lossy().to_string()}
                    onchange={onchange.clone()} />

                <div class={"input-group-append"}>
                    <div class={"input-group-text"}>{ ".xlsx" }</div>
                </div>
            </div>

            <div class="form-control">
                <input name={"name"}
                    placeholder={"Name"}
                    value={name.clone().unwrap_or("".to_string())}
                    onchange={onchange.clone()} />
            </div>

            <div class="form-control">
                <input name={"kind"}
                    placeholder={"Type"}
                    value={kind.clone().unwrap_or("".to_string())}
                    onchange={onchange.clone()} />
            </div>

            <div class="form-control">
                <TagsEditor
                    value={tags.clone()}
                    onchange={onchange_tags} />
            </div>

            <div class="form-control">
                <textarea name={"description"}
                    placeholder={"Description"}
                    onchange={onchange_description} >

                    { description.unwrap_or("".to_string()) }
                </textarea>
            </div>
        </div>
    }
}

//! Project scripts editor.
use crate::actions::container::Action as ContainerAction;
use crate::app::{
    AppStateAction, AppStateDispatcher, AppStateReducer, PageOverlay, ProjectsStateAction,
    ProjectsStateReducer,
};
use crate::commands::analysis::{copy_contents_to_analyses, update_excel_template};
use crate::commands::common::open_file;
use crate::commands::project::get_project_path;
use crate::common::DisplayName;
use crate::components::excel_template::{
    CreateExcelTemplate, ExcelTemplateBuilder, ExcelTemplateEditor,
};
use crate::hooks::use_canvas_project;
use std::collections::HashSet;
use std::path::PathBuf;
use syre_core::project::ExcelTemplate;
use syre_core::types::ResourceId;
use syre_local::types::AnalysisKind;
use syre_ui::types::Message;
use syre_ui::widgets::script::CreateScript;
use wasm_bindgen::prelude::Closure;
use wasm_bindgen::JsCast;
use web_sys::FileReader;
use yew::platform::spawn_local;
use yew::prelude::*;
use yew_icons::{Icon, IconId};

#[derive(Properties, PartialEq)]
pub struct ProjectAnalysesProps {
    /// Called when a user adds `Script`s to the `Project`.
    #[prop_or_default]
    pub onadd: Option<Callback<HashSet<PathBuf>>>,

    /// Called when a user adds Excel Templates to the `Project`.
    #[prop_or_default]
    pub onadd_excel_template: Option<Callback<ExcelTemplate>>,

    /// Called when a user removes a `Script`.
    #[prop_or_default]
    pub onremove: Option<Callback<ResourceId>>,
}

#[function_component(ProjectAnalyses)]
pub fn project_analyses(props: &ProjectAnalysesProps) -> HtmlResult {
    let app_state = use_context::<AppStateReducer>().unwrap();
    let projects_state = use_context::<ProjectsStateReducer>().unwrap();
    let project = use_canvas_project();
    let Some(project_analyses) = projects_state.project_analyses.get(&*project) else {
        panic!("`Project`'s analyses not loaded");
    };

    let excel_template_create_state = use_state(|| Option::<PathBuf>::None);
    let excel_template_edit_state = use_state(|| Option::<syre_core::types::ResourceId>::None);
    let drag_over_state = use_state(|| 0);

    let ondblclick_analysis = {
        let app_state = app_state.dispatcher();
        let projects_state = projects_state.clone();
        let project = project.clone();
        let excel_template_editor_state = excel_template_edit_state.setter();

        move |analysis: ResourceId| {
            let app_state = app_state.clone();
            let project = projects_state.projects.get(&*project).unwrap();
            let analysis_root = project.analysis_root.clone().unwrap();
            match projects_state
                .project_analyses
                .get(&project.rid)
                .unwrap()
                .get(&analysis)
                .unwrap()
            {
                AnalysisKind::Script(script) => {
                    let script_rel_path = analysis_root.join(&script.path);
                    open_script_callback(app_state, project.rid.clone(), script_rel_path)
                }

                AnalysisKind::ExcelTemplate(template) => {
                    let template = template.rid.clone();
                    let excel_template_editor_state = excel_template_editor_state.clone();
                    Callback::from(move |_: MouseEvent| {
                        excel_template_editor_state.set(Some(template.clone()));
                    })
                }
            }
        }
    };

    let onclick_remove = {
        let onremove = props.onremove.clone();
        move |rid: ResourceId| {
            let onremove = onremove.clone();
            Callback::from(move |e: MouseEvent| {
                e.stop_propagation();
                let onremove = onremove.clone();
                if let Some(onremove) = onremove {
                    onremove.emit(rid.clone());
                }
            })
        }
    };

    let ondragstart_script = move |script: ResourceId| {
        Callback::from(move |e: DragEvent| {
            let data_transfer = e.data_transfer().unwrap();
            data_transfer.clear_data().unwrap();
            data_transfer
                .set_data(
                    "application/json",
                    &serde_json::to_string(&ContainerAction::AddScriptAssociation(script.clone()))
                        .unwrap(),
                )
                .unwrap();
        })
    };

    let ondragenter = use_callback(drag_over_state.clone(), {
        move |e: DragEvent, drag_over_state| {
            e.prevent_default();
            drag_over_state.set(**drag_over_state + 1);
        }
    });

    let ondragover = use_callback((), move |e: DragEvent, _| {
        e.prevent_default();
    });

    let ondragleave = use_callback(drag_over_state.clone(), {
        move |e: DragEvent, drag_over_state| {
            e.prevent_default();
            drag_over_state.set(**drag_over_state - 1);
        }
    });

    let ondrop = use_callback((project.clone(), drag_over_state.clone()), {
        let app_state = app_state.dispatcher();
        let excel_template_create_state = excel_template_create_state.setter();

        let project = projects_state.projects.get(&*project).unwrap();
        let analysis_root = project.analysis_root.clone().unwrap();
        move |e: DragEvent, (project, drag_over_state)| {
            e.prevent_default();
            e.stop_propagation();
            drag_over_state.set(0);

            let project = (**project).clone();
            let drop_data = e.data_transfer().unwrap();
            let files = drop_data.files().unwrap();

            if files.length() == 1 {
                let file = files.item(0).unwrap();
                let file_name = PathBuf::from(file.name());
                if let Some(extension) = file_name.extension() {
                    if ExcelTemplate::supported_extensions().contains(&extension.to_str().unwrap())
                    {
                        let app_state = app_state.clone();
                        let excel_template_create_state = excel_template_create_state.clone();
                        let project = project.clone();
                        let analysis_root = analysis_root.clone();
                        spawn_local(async move {
                            let Ok(project_path) = get_project_path(project.clone()).await else {
                                let mut msg = Message::error("Could not create Excel template");
                                msg.set_details("Could not get project path.");
                                app_state.dispatch(AppStateAction::AddMessage(msg));
                                return;
                            };

                            copy_file_contents_to_analyses(file, project, {
                                let app_state = app_state.clone();
                                let file_name = file_name.clone();
                                move |res| match res {
                                    Ok(file_name) => {
                                        let template_path =
                                            project_path.join(analysis_root).join(file_name);

                                        excel_template_create_state.set(Some(template_path));
                                    }

                                    Err(err) => {
                                        tracing::error!(?err);

                                        let mut msg = Message::error(
                                            "Could not copy file contents to analyses folder.",
                                        );
                                        msg.set_details(format!("[{file_name:?}] {err:?}"));
                                        app_state.dispatch(AppStateAction::AddMessage(msg))
                                    }
                                }
                            });
                        });

                        return;
                    }
                }
            }

            let supported_ext = syre_core::project::ScriptLang::supported_extensions();
            let mut scripts = Vec::with_capacity(files.length() as usize);
            let mut invalid = Vec::with_capacity(files.length() as usize);
            for index in 0..files.length() {
                let file = files.item(index).unwrap();
                let file_name = PathBuf::from(file.name());
                match file_name.extension() {
                    Some(ext)
                        if supported_ext.contains(&ext.to_ascii_lowercase().to_str().unwrap()) =>
                    {
                        scripts.push(index);
                    }

                    _ => {
                        invalid.push(index);
                    }
                }
            }

            if invalid.len() > 0 {
                let details = invalid
                    .into_iter()
                    .map(|index| {
                        let file = files.item(index).unwrap();
                        file.name()
                    })
                    .collect::<Vec<_>>()
                    .join(", ");

                let details = format!("{details} are not supported as scripts.");
                let mut message = Message::error("Could not create scripts.");
                message.set_details(details);
                app_state.dispatch(AppStateAction::AddMessage(message));
            }

            for index in scripts {
                let file = files.item(index).unwrap();
                let file_name = file.name();
                copy_file_contents_to_analyses(file, project.clone(), {
                    let app_state = app_state.clone();
                    move |res| match res {
                        Ok(_) => {}
                        Err(err) => {
                            tracing::error!(?err);

                            let mut msg =
                                Message::error("Could not copy file contents to analyses folder.");
                            msg.set_details(format!("[{file_name}] {err:?}"));
                            app_state.dispatch(AppStateAction::AddMessage(msg))
                        }
                    }
                });
            }
        }
    });

    let onchange_excel_template = use_callback(project.clone(), {
        let app_state = app_state.dispatcher();
        let projects_state = projects_state.dispatcher();
        move |template: ExcelTemplate, project| {
            let app_state = app_state.clone();
            let projects_state = projects_state.clone();
            let template = template.clone();
            let project = (**project).clone();
            spawn_local(async move {
                match update_excel_template(template.clone()).await {
                    Ok(_) => {
                        projects_state.dispatch(ProjectsStateAction::UpdateExcelTemplate {
                            project,
                            template,
                        });
                    }

                    Err(err) => {
                        tracing::error!(?err);
                        let mut msg = Message::error("Could not update template.");
                        msg.set_details(err);
                        app_state.dispatch(AppStateAction::AddMessage(msg));
                    }
                }
            });
        }
    });

    let mut class = classes!("project-analyses-widget", "px-xl", "h-100", "box-border");
    if *drag_over_state > 0 {
        class.push("dragover-active");
    }

    Ok(html! {
    <>
        <div {class}
            {ondragenter}
            {ondragover}
            {ondragleave}
            {ondrop} >

            <div class={"create-analyses-controls flex"}>
                <label class={"grow"}>{ "Add" }</label>

                if let Some(onadd) = props.onadd.as_ref() {
                    <CreateScript oncreate={onadd.clone()}>
                        <Icon icon_id={IconId::FontAwesomeSolidCode} />
                    </CreateScript>
                }

                if let Some(onadd_template) = props.onadd_excel_template.as_ref() {
                    <CreateExcelTemplate oncreate={onadd_template.clone()}>
                        <Icon icon_id={IconId::FontAwesomeRegularFileExcel} />
                    </CreateExcelTemplate >
                }
            </div>

            <ul>
                { project_analyses.values().map(|analysis| {
                    let rid = match analysis {
                        AnalysisKind::Script(script) => &script.rid,
                        AnalysisKind::ExcelTemplate(template) => &template.rid,
                    };

                    html! {
                        <li key={rid.clone()}
                            data-rid={format!("{}", rid)}>

                            <span class={"name clickable"}
                                title={analysis.display_name()}
                                ondblclick={ondblclick_analysis(rid.clone())}
                                ondragstart={ondragstart_script(rid.clone())}
                                draggable={"true"} >
                                { analysis.display_name() }
                            </span>

                            if props.onremove.is_some() {
                                <button class={"btn-icon"} type={"button"}
                                    onclick={onclick_remove(rid.clone())}>

                                    <Icon class={"syre-ui-add-remove-icon"}
                                        icon_id={IconId::HeroiconsSolidMinus}/>
                                </button>
                            }
                        </li>
                    }
                }).collect::<Html>() }
            </ul>
        </div>

        if let Some(template) = excel_template_edit_state.as_ref() {
            <ExcelTemplateEditor
                template={template.clone()}
                onchange={onchange_excel_template}
                onclose={move |_| excel_template_edit_state.set(None)} />
        }

        if let Some(path) = excel_template_create_state.as_ref() {
            if let Some(oncreate_template) = props.onadd_excel_template.clone() {
                <PageOverlay
                    onclose={
                        let state = excel_template_create_state.setter();
                        move |_| { state.set(None); }
                    } >
                    <div class={"excel-template-builder-wrapper"}>
                        <h1>{ "Create an Excel template" }</h1>
                        <ExcelTemplateBuilder
                            path={path.clone()}
                            oncreate={oncreate_template.clone()} />
                    </div>
                </PageOverlay>
            }
        }
    </>
    })
}

fn open_script_callback(
    app_state: AppStateDispatcher,
    project: ResourceId,
    rel_script_path: PathBuf,
) -> Callback<MouseEvent> {
    Callback::from(move |e: MouseEvent| {
        e.stop_propagation();
        let app_state = app_state.clone();
        let project = project.clone();
        let rel_script_path = rel_script_path.clone();

        spawn_local(async move {
            let mut path = match get_project_path(project).await {
                Ok(path) => path,
                Err(err) => {
                    let mut msg = Message::error("Could not get project path.");
                    msg.set_details(err);
                    app_state.dispatch(AppStateAction::AddMessage(msg));
                    return;
                }
            };

            path.push(rel_script_path);
            match open_file(path).await {
                Ok(_) => {}
                Err(err) => {
                    let mut msg = Message::error("Could not open file.");
                    msg.set_details(err);
                    app_state.dispatch(AppStateAction::AddMessage(msg));
                    return;
                }
            };
        });
    })
}

fn copy_file_contents_to_analyses<F>(file: web_sys::File, project: ResourceId, on_result: F)
where
    F: 'static + FnOnce(Result<PathBuf, String>),
{
    let file_name = file.name();
    let file_reader = web_sys::FileReader::new().unwrap();
    file_reader.read_as_array_buffer(&file).unwrap();
    let onload = Closure::once(move |e: Event| {
        let file_reader: FileReader = e.target().unwrap().dyn_into().unwrap();
        let file = file_reader.result().unwrap();
        let file = js_sys::Uint8Array::new(&file);

        let mut contents = vec![0; file.length() as usize];
        file.copy_to(&mut contents);

        let file_name = PathBuf::from(file_name.clone());
        let project = project.clone();
        spawn_local(async move {
            let res = copy_contents_to_analyses(project, file_name.clone(), contents).await;
            on_result(res);
        });
    });

    file_reader.set_onload(Some(onload.as_ref().unchecked_ref()));
    onload.forget();
}

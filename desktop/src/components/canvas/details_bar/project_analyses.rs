//! Project scripts editor.
use crate::actions::container::Action as ContainerAction;
use crate::app::{AppStateAction, AppStateDispatcher, AppStateReducer, ProjectsStateReducer};
use crate::commands::analysis::add_script_windows;
use crate::commands::common::open_file;
use crate::commands::project::get_project_path;
use crate::components::excel_template::CreateExcelTemplate;
use crate::hooks::use_canvas_project;
use crate::lib::DisplayName;
use std::collections::HashSet;
use std::path::PathBuf;
use syre_core::project::ExcelTemplate;
use syre_core::types::ResourceId;
use syre_local::types::AnalysisKind;
use syre_ui::types::Message;
use syre_ui::widgets::script::CreateScript;
use wasm_bindgen::prelude::Closure;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use web_sys::FileReader;
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

    let drag_over_state = use_state(|| 0);

    let ondblclick_analysis = {
        let app_state = app_state.dispatcher();
        let projects_state = projects_state.clone();
        let project = project.clone();

        move |script: ResourceId| {
            let app_state = app_state.clone();
            let project = projects_state.projects.get(&*project).unwrap();
            let analysis_root = project.analysis_root.clone().unwrap();
            let analysis_path = match projects_state
                .project_analyses
                .get(&project.rid)
                .unwrap()
                .get(&script)
                .unwrap()
            {
                AnalysisKind::Script(script) => &script.path,
                AnalysisKind::ExcelTemplate(template) => &template.template.path,
            }
            .clone();

            let script_rel_path = analysis_root.join(analysis_path);
            open_script_callback(app_state, project.rid.clone(), script_rel_path)
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

    let ondrop = use_callback(
        (project.clone(), drag_over_state.clone()),
        move |e: DragEvent, (project, drag_over_state)| {
            e.prevent_default();
            e.stop_propagation();
            drag_over_state.set(0);

            let drop_data = e.data_transfer().unwrap();
            let files = drop_data.files().unwrap();

            let supported_ext = syre_core::project::ScriptLang::supported_extensions();
            let mut scripts = Vec::with_capacity(files.length() as usize);
            let mut invalid = Vec::with_capacity(files.length() as usize);
            for index in 0..files.length() {
                let file = files.item(index).expect("could not get `File`");
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

                let file_reader = web_sys::FileReader::new().unwrap();
                file_reader.read_as_array_buffer(&file).unwrap();
                let project = (**project).clone();
                let onload = Closure::<dyn FnMut(Event)>::new(move |e: Event| {
                    let file_reader: FileReader = e.target().unwrap().dyn_into().unwrap();
                    let file = file_reader.result().unwrap();
                    let file = js_sys::Uint8Array::new(&file);

                    let mut contents = vec![0; file.length() as usize];
                    file.copy_to(&mut contents);

                    let file_name = file_name.clone();
                    let project = project.clone();
                    spawn_local(async move {
                        match add_script_windows(project, PathBuf::from(file_name), contents).await
                        {
                            Ok(_) => {}
                            Err(err) => {
                                tracing::debug!(err);
                                panic!("{err}");
                            }
                        }
                    });
                });

                file_reader.set_onload(Some(onload.as_ref().unchecked_ref()));
                onload.forget();
            }
        },
    );

    let mut class = classes!("project-scripts-widget", "px-xl", "h-100", "box-border");
    if *drag_over_state > 0 {
        class.push("dragover-active");
    }

    Ok(html! {
        <div {class}
            {ondragenter}
            {ondragover}
            {ondragleave}
            {ondrop} >

            if let Some(onadd) = props.onadd.as_ref() {
                <CreateScript class={"block mx-auto"} oncreate={onadd.clone()} />
            }
            if let Some(onadd_template) = props.onadd_excel_template.as_ref() {
                <CreateExcelTemplate oncreate={onadd_template.clone()} />
            }

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

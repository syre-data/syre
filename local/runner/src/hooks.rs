//! Local runner hooks.
use std::path::PathBuf;
use syre_core::project::{ExcelTemplate, Project, Script, ScriptLang};
use syre_core::runner::{Runnable, RunnerHooks as CoreRunnerHooks};
use syre_core::types::ResourceId;
use syre_local::system::settings::RunnerSettings;
use syre_local::types::analysis::AnalysisKind;
use syre_local_database::{AnalysisCommand, Client as DbClient, ProjectCommand};

/// Retrieves a local [`Script`](CoreScript) given its [`ResourceId`].
#[tracing::instrument]
pub fn get_script(rid: &ResourceId) -> Result<Box<dyn Runnable>, String> {
    let db = DbClient::new();
    let Ok(script) = db.send(AnalysisCommand::Get(rid.clone()).into()) else {
        return Err("could not retrieve script".to_string());
    };

    let script: Option<AnalysisKind> = serde_json::from_value(script).unwrap();
    let Some(script) = script else {
        return Err("script not loaded".to_string());
    };

    match script {
        AnalysisKind::Script(script) => Ok(handle_script(&db, script)?),
        AnalysisKind::ExcelTemplate(template) => Ok(handle_excel_template(&db, template)?),
    }
}

pub struct RunnerHooks {}
impl RunnerHooks {
    pub fn new() -> CoreRunnerHooks {
        CoreRunnerHooks::new(get_script)
    }
}

fn handle_script(db: &DbClient, mut script: Script) -> Result<Box<Script>, String> {
    if script.path.is_relative() {
        // get absolute path to script
        let mut abs_path = get_base_path(db, script.rid.clone());
        abs_path.push(script.path);
        script.path = abs_path;
    } else if script.path.is_absolute() {
        todo!();
    } else {
        todo!();
    }

    // TODO[h]: Settings should be passed in and not loaded here. This is a temporary fix.
    // Get runner settings and override script's cmd if necessary
    if let Ok(runner_settings) = RunnerSettings::load() {
        match script.env.language {
            ScriptLang::Python => {
                if let Some(python_path) = runner_settings.python_path.clone() {
                    script.env.cmd = python_path;
                }
            }

            ScriptLang::R => {
                if let Some(r_path) = runner_settings.r_path.clone() {
                    script.env.cmd = r_path;
                }
            }
        }
    };

    Ok(Box::new(script))
}

fn handle_excel_template(
    db: &DbClient,
    mut template: ExcelTemplate,
) -> Result<Box<ExcelTemplate>, String> {
    if template.template.path.is_relative() {
        // get absolute path to template
        let mut abs_path = get_base_path(db, template.rid.clone());
        abs_path.push(template.template.path);
        template.template.path = abs_path;
    } else if template.template.path.is_absolute() {
        todo!();
    } else {
        todo!();
    }

    // TODO[h]: Settings should be passed in and not loaded here. This is a temporary fix.
    // Get runner settings and override script's cmd if necessary
    if let Ok(runner_settings) = RunnerSettings::load() {
        if let Some(python_path) = runner_settings.python_path.clone() {
            template.python_exe = python_path;
        }
    }

    Ok(Box::new(template))
}

fn get_base_path(db: &DbClient, rid: ResourceId) -> PathBuf {
    let project = db.send(AnalysisCommand::GetProject(rid).into()).unwrap();

    let project: Option<Project> = serde_json::from_value(project).unwrap();
    let project = project.unwrap();
    let analysis_root = project.analysis_root.unwrap().clone();
    let project_path = db
        .send(ProjectCommand::GetPath(project.rid.clone()).into())
        .unwrap();

    let project_path: Option<PathBuf> = serde_json::from_value(project_path).unwrap();
    let project_path = project_path.unwrap();

    let mut abs_path = project_path;
    abs_path.push(analysis_root);
    abs_path
}

//! Local runner hooks.
use std::path::{Path, PathBuf};
use syre_core::{
    project::{ExcelTemplate, Project, Script, ScriptLang},
    runner::{self, Runnable, RunnerHooks},
    types::ResourceId,
};
use syre_local::{
    self as local,
    system::config::{runner_settings, RunnerSettings},
    types::analysis::AnalysisKind,
};
use syre_local_database::state;

pub struct Runner {
    analyses: Vec<(ResourceId, AnalysisKind)>,
}

impl Runner {
    /// # Arguments
    /// `path`: Path to the projects base directory.
    pub fn from(path: impl AsRef<Path>, project: &state::ProjectData) -> Result<Self, error::From> {
        let analyses = Self::create_analyses(path, project)?;
        Ok(Self { analyses })
    }

    /// # Returns
    /// List of `(id, analysis)`.
    fn create_analyses(
        path: impl AsRef<Path>,
        project_data: &state::ProjectData,
    ) -> Result<Vec<(ResourceId, AnalysisKind)>, error::From> {
        let state::DataResource::Ok(analyses) = project_data.analyses() else {
            return Err(error::From::InvalidAnalysesState);
        };

        let state::DataResource::Ok(properties) = project_data.properties() else {
            return Err(error::From::InvalidPropertiesState);
        };

        let Some(analysis_root) = properties.analysis_root.as_ref() else {
            return Err(error::From::NoAnalysisRoot);
        };
        let analysis_root = path.as_ref().join(analysis_root);

        // TODO: Settings should be passed in and not loaded here. This is a temporary fix.
        // Get runner settings and override script's cmd if necessary
        let runner_settings = RunnerSettings::load();

        let map = analyses
            .clone()
            .into_iter()
            .map(|analysis| match analysis.properties() {
                AnalysisKind::Script(script) => {
                    let script = Self::create_analysis_script(
                        script.clone(),
                        &analysis_root,
                        &runner_settings,
                    );
                    (script.rid().clone(), AnalysisKind::Script(script))
                }
                AnalysisKind::ExcelTemplate(template) => {
                    let template = Self::create_analysis_excel_template(
                        template.clone(),
                        &analysis_root,
                        &runner_settings,
                    );
                    (
                        template.rid().clone(),
                        AnalysisKind::ExcelTemplate(template),
                    )
                }
            })
            .collect();

        Ok(map)
    }

    /// Modifies the given analysis script for the runner.
    fn create_analysis_script(
        mut script: Script,
        analysis_root: &PathBuf,
        runner_settings: &Result<RunnerSettings, local::Error>,
    ) -> Script {
        if script.path.is_relative() {
            script.path = analysis_root.join(script.path);
        } else if script.path.is_absolute() {
            todo!();
        } else {
            todo!();
        }

        if let Ok(runner_settings) = runner_settings {
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

        script
    }

    /// Modifies the given analysis script for the runner.
    fn create_analysis_excel_template(
        mut template: ExcelTemplate,
        analysis_root: &PathBuf,
        runner_settings: &Result<RunnerSettings, local::Error>,
    ) -> ExcelTemplate {
        if template.template.path.is_relative() {
            template.template.path = analysis_root.join(template.template.path);
        } else if template.template.path.is_absolute() {
            todo!();
        } else {
            todo!();
        }

        if let Ok(runner_settings) = runner_settings {
            if let Some(python_path) = runner_settings.python_path.clone() {
                template.python_exe = python_path;
            }
        }

        template
    }
}

impl RunnerHooks for Runner {
    /// Retrieves a local [`Script`](CoreScript) given its [`ResourceId`].
    fn get_analysis(
        &self,
        project: ResourceId,
        analysis: ResourceId,
    ) -> Result<Box<dyn Runnable>, String> {
        self.analyses
            .iter()
            .find_map(|(id, runner_analysis)| {
                if *id == analysis {
                    let analysis = match runner_analysis {
                        AnalysisKind::Script(script) => {
                            Box::new(script.clone()) as Box<dyn Runnable>
                        }
                        AnalysisKind::ExcelTemplate(template) => {
                            Box::new(template.clone()) as Box<dyn Runnable>
                        }
                    };
                    Some(analysis)
                } else {
                    None
                }
            })
            .ok_or(format!("could not find analysis {analysis}"))
    }
}

pub mod error {
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, Debug)]
    pub enum From {
        /// Project analyses are not in a valid state.
        InvalidAnalysesState,

        /// Project properties are not in a valid state.
        InvalidPropertiesState,

        // The Project's analysis root is not set.
        NoAnalysisRoot,
    }
}

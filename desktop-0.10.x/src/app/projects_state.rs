//! Projects state.
use indexmap::IndexSet;
use std::path::PathBuf;
use std::rc::Rc;
use syre_core::project::{ExcelTemplate, Project, Script};
use syre_core::types::{ResourceId, ResourceMap};
use syre_local::types::{AnalysisKind, AnalysisStore, ProjectSettings};
use yew::prelude::*;

pub type ProjectMap = ResourceMap<Project>;
pub type SettingsMap = ResourceMap<ProjectSettings>;

/// Map from a `Project` to its analyses.
pub type ProjectAnalysesMap = ResourceMap<AnalysisStore>;

/// Actions for [`ProjectsState`].
#[derive(Debug)]
pub enum ProjectsStateAction {
    /// Insert a project.
    InsertProject((Project, ProjectSettings)),

    /// Inserts multiple projects.
    InsertProjects(Vec<(Project, ProjectSettings)>),

    RemoveProject(ResourceId),

    /// Add an open project.
    AddOpenProject(ResourceId),

    /// Remove an open project.
    ///
    /// # Fields
    /// + `project`: Project to remove.
    /// + `activate`: Project to set as active, if needed.
    RemoveOpenProject {
        project: ResourceId,
        activate: Option<ResourceId>,
    },

    /// Set the active `Project`.
    SetActiveProject(ResourceId),

    /// Update the [`Project`].
    UpdateProject(Project),

    InsertProjectScript {
        project: ResourceId,
        script: Script,
    },

    /// Inserts a `Project`'s analyses.
    InsertProjectAnalyses {
        project: ResourceId,
        analyses: AnalysisStore,
    },

    InsertProjectExcelTemplate {
        project: ResourceId,
        template: ExcelTemplate,
    },

    UpdateExcelTemplate {
        project: ResourceId,
        template: ExcelTemplate,
    },

    RemoveProjectScript(ResourceId),

    MoveProjectScript {
        script: ResourceId,
        path: PathBuf,
    },
}

#[derive(Debug, Default, PartialEq, Clone)]
/// Maintains the state of application [`Projects`].
pub struct ProjectsState {
    /// All user [`Projects`].
    pub projects: ProjectMap,

    /// Project settings.
    pub settings: SettingsMap,

    /// `Project` analyses.
    pub project_analyses: ProjectAnalysesMap,

    /// Open [`Projects`].
    pub open_projects: IndexSet<ResourceId>,

    /// The active [`Project`].
    pub active_project: Option<ResourceId>,
}

impl Reducible for ProjectsState {
    type Action = ProjectsStateAction;
    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        let mut current = (*self).clone();
        match action {
            ProjectsStateAction::InsertProject((project, settings)) => {
                current.settings.insert(project.rid.clone(), settings);
                current.projects.insert(project.rid.clone(), project);
            }

            ProjectsStateAction::InsertProjects(projects) => {
                for (project, settings) in projects {
                    current.settings.insert(project.rid.clone(), settings);
                    current.projects.insert(project.rid.clone(), project);
                }
            }

            ProjectsStateAction::RemoveProject(project) => {
                current.settings.remove(&project);
                current.project_analyses.remove(&project);
                current.projects.remove(&project);
                current.open_projects.shift_remove(&project);
                if let Some(active_project) = current.active_project.as_ref() {
                    if active_project == &project {
                        current.active_project = None;
                    }
                }
            }

            ProjectsStateAction::AddOpenProject(project) => {
                current.open_projects.insert(project);
            }

            ProjectsStateAction::RemoveOpenProject { project, activate } => {
                if current.active_project.as_ref() == Some(&project) {
                    // closed the active project
                    current.active_project = activate;
                }

                current.open_projects.shift_remove(&project);
            }

            ProjectsStateAction::SetActiveProject(rid) => {
                current.active_project = Some(rid);
            }

            ProjectsStateAction::InsertProjectScript { project, script } => {
                let scripts = current.project_analyses.get_mut(&project).unwrap();
                scripts.insert(script.rid.clone(), script.into());
            }

            ProjectsStateAction::InsertProjectAnalyses { project, analyses } => {
                current.project_analyses.insert(project, analyses);
            }

            ProjectsStateAction::InsertProjectExcelTemplate { project, template } => {
                let scripts = current.project_analyses.get_mut(&project).unwrap();
                scripts.insert(template.rid.clone(), template.into());
            }

            ProjectsStateAction::UpdateExcelTemplate { project, template } => {
                let analyses = current.project_analyses.get_mut(&project).unwrap();
                analyses.insert(template.rid.clone(), AnalysisKind::ExcelTemplate(template));
            }

            ProjectsStateAction::RemoveProjectScript(analysis) => {
                for (_project, analyses) in current.project_analyses.iter_mut() {
                    if analyses.contains_key(&analysis) {
                        analyses.remove(&analysis);
                        break;
                    }
                }
            }

            ProjectsStateAction::MoveProjectScript { script, path } => {
                for (_project, analyses) in current.project_analyses.iter_mut() {
                    if let Some(analysis) = analyses.get_mut(&script) {
                        match analysis {
                            AnalysisKind::Script(script) => script.path = path,
                            AnalysisKind::ExcelTemplate(template) => template.template.path = path,
                        }
                        break;
                    }
                }
            }

            ProjectsStateAction::UpdateProject(project) => {
                current.projects.insert(project.rid.clone(), project);
            }
        }

        current.into()
    }
}

pub type ProjectsStateReducer = UseReducerHandle<ProjectsState>;
pub type ProjectsStateDispatcher = UseReducerDispatcher<ProjectsState>;

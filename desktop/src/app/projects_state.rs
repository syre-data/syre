//! Projects state.
// use crate::commands::settings::UserAppStateArgs;
use indexmap::IndexSet;
use std::path::PathBuf;
use std::rc::Rc;
use syre_core::project::{Project, Script, Scripts};
use syre_core::types::{ResourceId, ResourceMap};
use syre_local::types::ProjectSettings;
use yew::prelude::*;

pub type ProjectMap = ResourceMap<Project>;
pub type SettingsMap = ResourceMap<ProjectSettings>;

/// Map from a `Project` to its `Scripts`.
pub type ProjectScriptsMap = ResourceMap<Scripts>;

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

    /// Inserts `Script`s into a `Project`.
    ///
    /// # Fields
    /// 1. `Project`'s id.
    /// 2. `Project`'s `Script`s.
    InsertProjectScripts(ResourceId, Scripts),

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

    /// `Project` `Script`s.
    pub project_scripts: ProjectScriptsMap,

    /// Open [`Projects`].
    pub open_projects: IndexSet<ResourceId>,

    /// The active [`Project`].
    pub active_project: Option<ResourceId>,
}

impl Reducible for ProjectsState {
    type Action = ProjectsStateAction;

    #[tracing::instrument(level = "debug", skip(self))]
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
                current.project_scripts.remove(&project);
                current.projects.remove(&project);
                current.open_projects.remove(&project);
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

                current.open_projects.remove(&project);
            }

            ProjectsStateAction::SetActiveProject(rid) => {
                current.active_project = Some(rid);
            }

            ProjectsStateAction::InsertProjectScript { project, script } => {
                let scripts = current.project_scripts.get_mut(&project).unwrap();
                scripts.insert(script.rid.clone(), script);
            }

            ProjectsStateAction::InsertProjectScripts(project, scripts) => {
                current.project_scripts.insert(project, scripts);
            }

            ProjectsStateAction::RemoveProjectScript(script) => {
                for (_project, scripts) in current.project_scripts.iter_mut() {
                    if scripts.contains_key(&script) {
                        scripts.remove(&script);
                        break;
                    }
                }
            }

            ProjectsStateAction::MoveProjectScript { script, path } => {
                for (_project, scripts) in current.project_scripts.iter_mut() {
                    if let Some(script) = scripts.get_mut(&script) {
                        script.path = path;
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

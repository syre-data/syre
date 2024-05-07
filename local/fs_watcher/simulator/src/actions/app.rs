use rand::{distributions, prelude::*};

#[derive(Debug)]
pub struct Action {
    resource: Resource,
    action: ResourceAction,
}

impl Action {
    pub fn resource(&self) -> &Resource {
        &self.resource
    }

    pub fn action(&self) -> &ResourceAction {
        &self.action
    }
}

impl Action {
    pub fn user_manifest_create() -> Self {
        Self {
            resource: Resource::App(AppResource::UserManifest),
            action: ResourceAction::Create,
        }
    }

    pub fn user_manifest_remove() -> Self {
        Self {
            resource: Resource::App(AppResource::UserManifest),
            action: ResourceAction::Remove,
        }
    }

    pub fn user_manifest_rename() -> Self {
        Self {
            resource: Resource::App(AppResource::UserManifest),
            action: ResourceAction::Rename,
        }
    }

    pub fn user_manifest_move() -> Self {
        Self {
            resource: Resource::App(AppResource::UserManifest),
            action: ResourceAction::Move,
        }
    }

    pub fn user_manifest_copy() -> Self {
        Self {
            resource: Resource::App(AppResource::UserManifest),
            action: ResourceAction::Copy,
        }
    }

    pub fn user_manifest_corrupt() -> Self {
        Self {
            resource: Resource::App(AppResource::UserManifest),
            action: ResourceAction::Corrupt,
        }
    }

    pub fn user_manifest_repair() -> Self {
        Self {
            resource: Resource::App(AppResource::UserManifest),
            action: ResourceAction::Repair,
        }
    }
}

impl Action {
    pub fn project_manifest_create() -> Self {
        Self {
            resource: Resource::App(AppResource::ProjectManifest),
            action: ResourceAction::Create,
        }
    }

    pub fn project_manifest_remove() -> Self {
        Self {
            resource: Resource::App(AppResource::ProjectManifest),
            action: ResourceAction::Remove,
        }
    }

    pub fn project_manifest_rename() -> Self {
        Self {
            resource: Resource::App(AppResource::ProjectManifest),
            action: ResourceAction::Rename,
        }
    }

    pub fn project_manifest_move() -> Self {
        Self {
            resource: Resource::App(AppResource::ProjectManifest),
            action: ResourceAction::Move,
        }
    }

    pub fn project_manifest_copy() -> Self {
        Self {
            resource: Resource::App(AppResource::ProjectManifest),
            action: ResourceAction::Copy,
        }
    }

    pub fn project_manifest_corrupt() -> Self {
        Self {
            resource: Resource::App(AppResource::ProjectManifest),
            action: ResourceAction::Corrupt,
        }
    }

    pub fn project_manifest_repair() -> Self {
        Self {
            resource: Resource::App(AppResource::ProjectManifest),
            action: ResourceAction::Repair,
        }
    }
}

impl Action {
    pub fn project_config_create() -> Self {
        Self {
            resource: Resource::Project(ProjectResource::Project(Project::ConfigDir)),
            action: ResourceAction::Create,
        }
    }

    pub fn project_config_remove() -> Self {
        Self {
            resource: Resource::Project(ProjectResource::Project(Project::ConfigDir)),
            action: ResourceAction::Remove,
        }
    }

    pub fn project_config_rename() -> Self {
        Self {
            resource: Resource::Project(ProjectResource::Project(Project::ConfigDir)),
            action: ResourceAction::Rename,
        }
    }

    pub fn project_config_move() -> Self {
        Self {
            resource: Resource::Project(ProjectResource::Project(Project::ConfigDir)),
            action: ResourceAction::Move,
        }
    }

    pub fn project_config_copy() -> Self {
        Self {
            resource: Resource::Project(ProjectResource::Project(Project::ConfigDir)),
            action: ResourceAction::Copy,
        }
    }
}

impl Action {
    pub fn project_properties_create() -> Self {
        Self {
            resource: Resource::Project(ProjectResource::Project(Project::Properties)),
            action: ResourceAction::Create,
        }
    }

    pub fn project_properties_remove() -> Self {
        Self {
            resource: Resource::Project(ProjectResource::Project(Project::Properties)),
            action: ResourceAction::Remove,
        }
    }

    pub fn project_properties_rename() -> Self {
        Self {
            resource: Resource::Project(ProjectResource::Project(Project::Properties)),
            action: ResourceAction::Rename,
        }
    }

    pub fn project_properties_move() -> Self {
        Self {
            resource: Resource::Project(ProjectResource::Project(Project::Properties)),
            action: ResourceAction::Move,
        }
    }

    pub fn project_properties_copy() -> Self {
        Self {
            resource: Resource::Project(ProjectResource::Project(Project::Properties)),
            action: ResourceAction::Copy,
        }
    }

    pub fn project_properties_corrupt() -> Self {
        Self {
            resource: Resource::Project(ProjectResource::Project(Project::Properties)),
            action: ResourceAction::Corrupt,
        }
    }

    pub fn project_properties_repair() -> Self {
        Self {
            resource: Resource::Project(ProjectResource::Project(Project::Properties)),
            action: ResourceAction::Repair,
        }
    }
}

impl Action {
    pub fn project_settings_create() -> Self {
        Self {
            resource: Resource::Project(ProjectResource::Project(Project::Settings)),
            action: ResourceAction::Create,
        }
    }

    pub fn project_settings_remove() -> Self {
        Self {
            resource: Resource::Project(ProjectResource::Project(Project::Settings)),
            action: ResourceAction::Remove,
        }
    }

    pub fn project_settings_rename() -> Self {
        Self {
            resource: Resource::Project(ProjectResource::Project(Project::Settings)),
            action: ResourceAction::Rename,
        }
    }

    pub fn project_settings_move() -> Self {
        Self {
            resource: Resource::Project(ProjectResource::Project(Project::Settings)),
            action: ResourceAction::Move,
        }
    }

    pub fn project_settings_copy() -> Self {
        Self {
            resource: Resource::Project(ProjectResource::Project(Project::Settings)),
            action: ResourceAction::Copy,
        }
    }

    pub fn project_settings_corrupt() -> Self {
        Self {
            resource: Resource::Project(ProjectResource::Project(Project::Settings)),
            action: ResourceAction::Corrupt,
        }
    }

    pub fn project_settings_repair() -> Self {
        Self {
            resource: Resource::Project(ProjectResource::Project(Project::Settings)),
            action: ResourceAction::Repair,
        }
    }
}

impl Action {
    pub fn project_analyses_create() -> Self {
        Self {
            resource: Resource::Project(ProjectResource::Project(Project::Analyses)),
            action: ResourceAction::Create,
        }
    }

    pub fn project_analyses_remove() -> Self {
        Self {
            resource: Resource::Project(ProjectResource::Project(Project::Analyses)),
            action: ResourceAction::Remove,
        }
    }

    pub fn project_analyses_rename() -> Self {
        Self {
            resource: Resource::Project(ProjectResource::Project(Project::Analyses)),
            action: ResourceAction::Rename,
        }
    }

    pub fn project_analyses_move() -> Self {
        Self {
            resource: Resource::Project(ProjectResource::Project(Project::Analyses)),
            action: ResourceAction::Move,
        }
    }

    pub fn project_analyses_copy() -> Self {
        Self {
            resource: Resource::Project(ProjectResource::Project(Project::Analyses)),
            action: ResourceAction::Copy,
        }
    }

    pub fn project_analyses_corrupt() -> Self {
        Self {
            resource: Resource::Project(ProjectResource::Project(Project::Analyses)),
            action: ResourceAction::Corrupt,
        }
    }

    pub fn project_analyses_repair() -> Self {
        Self {
            resource: Resource::Project(ProjectResource::Project(Project::Analyses)),
            action: ResourceAction::Repair,
        }
    }
}

impl Action {
    pub fn project_analysis_dir_create() -> Self {
        Self {
            resource: Resource::Project(ProjectResource::Project(Project::AnalysisDir)),
            action: ResourceAction::Create,
        }
    }

    pub fn project_analysis_dir_remove() -> Self {
        Self {
            resource: Resource::Project(ProjectResource::Project(Project::AnalysisDir)),
            action: ResourceAction::Remove,
        }
    }

    pub fn project_analysis_dir_rename() -> Self {
        Self {
            resource: Resource::Project(ProjectResource::Project(Project::AnalysisDir)),
            action: ResourceAction::Rename,
        }
    }

    pub fn project_analysis_dir_move() -> Self {
        Self {
            resource: Resource::Project(ProjectResource::Project(Project::AnalysisDir)),
            action: ResourceAction::Move,
        }
    }

    pub fn project_analysis_dir_copy() -> Self {
        Self {
            resource: Resource::Project(ProjectResource::Project(Project::AnalysisDir)),
            action: ResourceAction::Copy,
        }
    }
}

impl Action {
    pub fn project_data_dir_create() -> Self {
        Self {
            resource: Resource::Project(ProjectResource::Project(Project::DataDir)),
            action: ResourceAction::Create,
        }
    }

    pub fn project_data_dir_remove() -> Self {
        Self {
            resource: Resource::Project(ProjectResource::Project(Project::DataDir)),
            action: ResourceAction::Remove,
        }
    }

    pub fn project_data_dir_rename() -> Self {
        Self {
            resource: Resource::Project(ProjectResource::Project(Project::DataDir)),
            action: ResourceAction::Rename,
        }
    }

    pub fn project_data_dir_move() -> Self {
        Self {
            resource: Resource::Project(ProjectResource::Project(Project::DataDir)),
            action: ResourceAction::Move,
        }
    }

    pub fn project_data_dir_copy() -> Self {
        Self {
            resource: Resource::Project(ProjectResource::Project(Project::DataDir)),
            action: ResourceAction::Copy,
        }
    }
}

impl Action {
    pub fn container_remove() -> Self {
        Self {
            resource: Resource::Project(ProjectResource::Container(Container::Container)),
            action: ResourceAction::Remove,
        }
    }

    pub fn container_rename() -> Self {
        Self {
            resource: Resource::Project(ProjectResource::Container(Container::Container)),
            action: ResourceAction::Rename,
        }
    }

    pub fn container_move() -> Self {
        Self {
            resource: Resource::Project(ProjectResource::Container(Container::Container)),
            action: ResourceAction::Move,
        }
    }

    pub fn container_copy() -> Self {
        Self {
            resource: Resource::Project(ProjectResource::Container(Container::Container)),
            action: ResourceAction::Copy,
        }
    }
}

impl Action {
    pub fn container_config_create() -> Self {
        Self {
            resource: Resource::Project(ProjectResource::Container(Container::ConfigDir)),
            action: ResourceAction::Create,
        }
    }

    pub fn container_config_remove() -> Self {
        Self {
            resource: Resource::Project(ProjectResource::Container(Container::ConfigDir)),
            action: ResourceAction::Remove,
        }
    }

    pub fn container_config_rename() -> Self {
        Self {
            resource: Resource::Project(ProjectResource::Container(Container::ConfigDir)),
            action: ResourceAction::Rename,
        }
    }

    pub fn container_config_move() -> Self {
        Self {
            resource: Resource::Project(ProjectResource::Container(Container::ConfigDir)),
            action: ResourceAction::Move,
        }
    }

    pub fn container_config_copy() -> Self {
        Self {
            resource: Resource::Project(ProjectResource::Container(Container::ConfigDir)),
            action: ResourceAction::Copy,
        }
    }
}

impl Action {
    pub fn container_properties_create() -> Self {
        Self {
            resource: Resource::Project(ProjectResource::Container(Container::Properties)),
            action: ResourceAction::Create,
        }
    }

    pub fn container_properties_remove() -> Self {
        Self {
            resource: Resource::Project(ProjectResource::Container(Container::Properties)),
            action: ResourceAction::Remove,
        }
    }

    pub fn container_properties_rename() -> Self {
        Self {
            resource: Resource::Project(ProjectResource::Container(Container::Properties)),
            action: ResourceAction::Rename,
        }
    }

    pub fn container_properties_move() -> Self {
        Self {
            resource: Resource::Project(ProjectResource::Container(Container::Properties)),
            action: ResourceAction::Move,
        }
    }

    pub fn container_properties_copy() -> Self {
        Self {
            resource: Resource::Project(ProjectResource::Container(Container::Properties)),
            action: ResourceAction::Copy,
        }
    }

    pub fn container_properties_corrupt() -> Self {
        Self {
            resource: Resource::Project(ProjectResource::Container(Container::Properties)),
            action: ResourceAction::Corrupt,
        }
    }

    pub fn container_properties_repair() -> Self {
        Self {
            resource: Resource::Project(ProjectResource::Container(Container::Properties)),
            action: ResourceAction::Repair,
        }
    }
}

impl Action {
    pub fn container_settings_create() -> Self {
        Self {
            resource: Resource::Project(ProjectResource::Container(Container::Settings)),
            action: ResourceAction::Create,
        }
    }

    pub fn container_settings_remove() -> Self {
        Self {
            resource: Resource::Project(ProjectResource::Container(Container::Settings)),
            action: ResourceAction::Remove,
        }
    }

    pub fn container_settings_rename() -> Self {
        Self {
            resource: Resource::Project(ProjectResource::Container(Container::Settings)),
            action: ResourceAction::Rename,
        }
    }

    pub fn container_settings_move() -> Self {
        Self {
            resource: Resource::Project(ProjectResource::Container(Container::Settings)),
            action: ResourceAction::Move,
        }
    }

    pub fn container_settings_copy() -> Self {
        Self {
            resource: Resource::Project(ProjectResource::Container(Container::Settings)),
            action: ResourceAction::Copy,
        }
    }

    pub fn container_settings_corrupt() -> Self {
        Self {
            resource: Resource::Project(ProjectResource::Container(Container::Settings)),
            action: ResourceAction::Corrupt,
        }
    }

    pub fn container_settings_repair() -> Self {
        Self {
            resource: Resource::Project(ProjectResource::Container(Container::Settings)),
            action: ResourceAction::Repair,
        }
    }
}

impl Action {
    pub fn container_assets_create() -> Self {
        Self {
            resource: Resource::Project(ProjectResource::Container(Container::Assets)),
            action: ResourceAction::Create,
        }
    }

    pub fn container_assets_remove() -> Self {
        Self {
            resource: Resource::Project(ProjectResource::Container(Container::Assets)),
            action: ResourceAction::Remove,
        }
    }

    pub fn container_assets_rename() -> Self {
        Self {
            resource: Resource::Project(ProjectResource::Container(Container::Assets)),
            action: ResourceAction::Rename,
        }
    }

    pub fn container_assets_move() -> Self {
        Self {
            resource: Resource::Project(ProjectResource::Container(Container::Assets)),
            action: ResourceAction::Move,
        }
    }

    pub fn container_assets_copy() -> Self {
        Self {
            resource: Resource::Project(ProjectResource::Container(Container::Assets)),
            action: ResourceAction::Copy,
        }
    }

    pub fn container_assets_corrupt() -> Self {
        Self {
            resource: Resource::Project(ProjectResource::Container(Container::Assets)),
            action: ResourceAction::Corrupt,
        }
    }

    pub fn container_assets_repair() -> Self {
        Self {
            resource: Resource::Project(ProjectResource::Container(Container::Assets)),
            action: ResourceAction::Repair,
        }
    }
}

impl Action {
    pub fn asset_file_create() -> Self {
        Self {
            resource: Resource::Project(ProjectResource::AssetFile),
            action: ResourceAction::Create,
        }
    }

    pub fn asset_file_remove() -> Self {
        Self {
            resource: Resource::Project(ProjectResource::AssetFile),
            action: ResourceAction::Remove,
        }
    }

    pub fn asset_file_rename() -> Self {
        Self {
            resource: Resource::Project(ProjectResource::AssetFile),
            action: ResourceAction::Rename,
        }
    }

    pub fn asset_file_move() -> Self {
        Self {
            resource: Resource::Project(ProjectResource::AssetFile),
            action: ResourceAction::Move,
        }
    }

    pub fn asset_file_copy() -> Self {
        Self {
            resource: Resource::Project(ProjectResource::AssetFile),
            action: ResourceAction::Copy,
        }
    }
}

#[derive(Debug)]
pub enum ResourceAction {
    Create,
    Remove,
    Rename,
    Move,
    Copy,
    Corrupt,
    Repair,
    Modify,
}

#[derive(Debug)]
pub enum Resource {
    App(AppResource),
    Project(ProjectResource),
}

#[derive(Debug)]
pub enum AppResource {
    UserManifest,
    ProjectManifest,
}

#[derive(Debug, derive_more::From)]
pub enum ProjectResource {
    Project(Project),
    Container(Container),
    AssetFile,
}

#[derive(Debug)]
pub enum Project {
    /// Project base directory.
    Project,

    /// Project's analysis directory.
    AnalysisDir,

    /// Project's data directory.
    DataDir,

    /// prOject configuration directory (.syre).
    ConfigDir,

    /// Project properties file.
    Properties,

    /// Project settings file.
    Settings,

    /// Analyses manifest file.
    Analyses,
}

#[derive(Debug)]
pub enum Container {
    /// Container base directory.
    Container,
    ConfigDir,
    Properties,
    Settings,
    Assets,
}

impl Distribution<Action> for distributions::Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Action {
        Action {
            resource: self.sample(rng),
            action: self.sample(rng),
        }
    }
}

impl Distribution<ResourceAction> for distributions::Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> ResourceAction {
        match rng.gen_range(0..5) {
            0 => ResourceAction::Create,
            1 => ResourceAction::Remove,
            2 => ResourceAction::Rename,
            3 => ResourceAction::Move,
            4 => ResourceAction::Copy,
            _ => unreachable!(),
        }
    }
}

impl Distribution<Resource> for distributions::Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Resource {
        match rng.gen_range(0..2) {
            0 => Resource::App(self.sample(rng)),
            1 => Resource::Project(self.sample(rng)),
            _ => unreachable!(),
        }
    }
}

impl Distribution<AppResource> for distributions::Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> AppResource {
        match rng.gen_range(0..2) {
            0 => AppResource::ProjectManifest,
            1 => AppResource::UserManifest,
            _ => unreachable!(),
        }
    }
}

impl Distribution<ProjectResource> for distributions::Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> ProjectResource {
        match rng.gen_range(0..2) {
            0 => ProjectResource::Project(self.sample(rng)),
            1 => ProjectResource::Container(self.sample(rng)),
            _ => unreachable!(),
        }
    }
}

impl Distribution<Project> for distributions::Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Project {
        match rng.gen_range(0..7) {
            0 => Project::Project,
            1 => Project::AnalysisDir,
            2 => Project::DataDir,
            3 => Project::ConfigDir,
            4 => Project::Properties,
            5 => Project::Settings,
            6 => Project::Analyses,
            _ => unreachable!(),
        }
    }
}

impl Distribution<Container> for distributions::Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Container {
        match rng.gen_range(0..5) {
            0 => Container::Container,
            1 => Container::ConfigDir,
            2 => Container::Properties,
            3 => Container::Settings,
            4 => Container::Assets,
            _ => unreachable!(),
        }
    }
}

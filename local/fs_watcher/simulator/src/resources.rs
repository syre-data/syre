pub mod fs {
    use rand::{distributions, prelude::*};
    use std::path::PathBuf;

    pub struct Action {
        resource: Resource,
        action: ResourceAction,
        paths: Vec<PathBuf>,
    }

    impl Action {
        pub fn resource(&self) -> &Resource {
            &self.resource
        }

        pub fn action(&self) -> &ResourceAction {
            &self.action
        }

        pub fn add_path(&mut self, path: PathBuf) {
            self.paths.push(path);
        }
    }

    impl Distribution<Action> for distributions::Standard {
        fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Action {
            Action {
                resource: self.sample(rng),
                action: self.sample(rng),
                paths: Vec::with_capacity(2),
            }
        }
    }

    pub enum Resource {
        File,
        Folder,
    }

    pub enum ResourceAction {
        Create,
        Remove,
        Rename,
        Move,
        Copy,
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
                0 => Resource::File,
                1 => Resource::Folder,
                _ => unreachable!(),
            }
        }
    }
}

pub mod app {
    use rand::{distributions, prelude::*};
    use std::path::{Path, PathBuf};
    use syre_local::system::collections::ProjectManifest;

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

    pub enum ResourceAction {
        Create,
        Remove,
        Rename,
        Move,
        Copy,
    }

    pub enum Resource {
        App(AppResource),
        Project(ProjectResource),
    }

    pub enum AppResource {
        UserManifest,
        ProjectManifest,
    }

    #[derive(derive_more::From)]
    pub enum ProjectResource {
        Project(Project),
        Container(Container),
        AssetFile,
    }

    pub enum Project {
        Project,
        AnalysisDir,
        DataDir,
        ConfigDir,
        Properties,
        Settings,
        Analysis,
    }

    pub enum Container {
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
                6 => Project::Analysis,
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

    /// Used to track projects created for simulation.
    #[derive(Default)]
    pub struct ProjectRegistry(Vec<PathBuf>);
    impl ProjectRegistry {
        pub fn new() -> Self {
            Default::default()
        }

        pub fn push(&mut self, path: impl Into<PathBuf>) {
            self.0.push(path.into());
        }

        pub fn remove(&mut self, path: impl AsRef<Path>) {
            let path = path.as_ref();
            if let Some(index) = self.0.iter().position(|p| p == path) {
                self.0.remove(index);
            }
        }
    }

    impl Drop for ProjectRegistry {
        fn drop(&mut self) {
            let Ok(mut manifest) = ProjectManifest::load() else {
                return;
            };

            for project in self.0.iter() {
                manifest.remove(project);
            }

            let _ = manifest.save();
        }
    }
}

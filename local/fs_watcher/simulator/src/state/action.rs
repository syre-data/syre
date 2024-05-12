use rand::{distributions, prelude::*};
use std::path::PathBuf;
use syre_core::types::ResourceId;

#[derive(Debug, derive_more::From)]
pub enum Action {
    #[from]
    App(AppResource),

    /// Create a new project.
    CreateProject { id: ResourceId, path: PathBuf },

    #[from]
    Project {
        project: ResourceId,
        action: ProjectResource,
    },

    /// Begin watching a path.
    Watch(PathBuf),

    /// Stop watching a path.
    Unwatch(PathBuf),
}

#[derive(Debug)]
pub enum AppResource {
    UserManifest(Manifest),
    ProjectManifest(Manifest),
}

#[derive(Debug, derive_more::From)]
pub enum ProjectResource {
    #[from]
    Project(Project),

    /// Create a new Container.
    CreateContainer { parent: ResourceId, name: PathBuf },
    Container {
        container: ResourceId,
        action: Container,
    },

    /// Create a new asset file.
    CreateAssetFile {
        container: ResourceId,
        name: PathBuf,
    },

    AssetFile {
        container: ResourceId,
        asset: ResourceId,
        action: AssetFile,
    },
}

#[derive(Debug)]
pub enum Project {
    /// Project base directory.
    Project(ResourceDir),

    /// Project's analysis directory.
    AnalysisDir(Dir),

    /// Project's data directory.
    DataDir(Dir),

    /// Project configuration directory (.syre).
    ConfigDir(StaticDir),

    /// Project properties file.
    Properties(StaticFile),

    /// Project settings file.
    Settings(StaticFile),

    /// Analyses manifest file.
    Analyses(Manifest),
}

#[derive(Debug)]
pub enum Container {
    /// Container base directory.
    Container(ResourceDir),
    ConfigDir(StaticDir),
    Properties(StaticFile),
    Settings(StaticFile),
    Assets(Manifest),
}

#[derive(Debug)]
pub enum ResourceDir {
    Remove,
    Rename { to: PathBuf },
    Move { to: PathBuf },
    Copy { to: PathBuf },
}

#[derive(Debug)]
pub enum Dir {
    Create { path: PathBuf },
    Remove,
    Rename { to: PathBuf },
    Move { to: PathBuf },
    Copy { to: PathBuf },
}

#[derive(Debug)]
pub enum StaticDir {
    Create,
    Remove,
    Rename,
    Move,
    Copy,
}

#[derive(Debug)]
pub enum File {
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
pub enum StaticFile {
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
pub enum Manifest {
    Create,
    Remove,
    Rename,
    Move,
    Copy,
    Corrupt,
    Repair,
    Modify(ModifyManifest),
}

#[derive(Debug)]
pub enum ModifyManifest {
    /// Add an entry to the manifest.
    Add,

    /// Remove an entry from the manifest.
    Remove,

    /// Alter an entry in the manifest.
    Alter,
}

#[derive(Debug)]
pub enum MoveKind {
    Ancestor,
    Descendant,
    Sibling,

    /// Move out of the resource.
    OutOfResource,
}

impl Distribution<MoveKind> for distributions::Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> MoveKind {
        match rng.gen_range(0..4) {
            0 => MoveKind::Ancestor,
            1 => MoveKind::Descendant,
            2 => MoveKind::Sibling,
            3 => MoveKind::OutOfResource,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug)]
pub enum AssetFile {
    Remove,
    Rename,
    Move,
    Copy,
    Modify,
}

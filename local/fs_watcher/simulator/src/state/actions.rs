use rand::{distributions, prelude::*};
use std::path::PathBuf;
use syre_core::types::ResourceId;

#[derive(Debug, derive_more::From)]
pub enum Action {
    #[from]
    App(AppResource),

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
    Project(Project),
    Container {
        container: ResourceId,
        action: Container,
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
    Project(Dir),

    /// Project's analysis directory.
    AnalysisDir(Dir),

    /// Project's data directory.
    DataDir(Dir),

    /// prOject configuration directory (.syre).
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
    Container(Dir),
    ConfigDir(StaticDir),
    Properties(StaticFile),
    Settings(StaticFile),
    Assets(Manifest),
}

#[derive(Debug)]
pub enum Dir {
    Create(PathBuf),
    Remove,
    Rename(PathBuf),
    Move(PathBuf),
    Copy(PathBuf),
}

#[derive(Debug)]
pub enum StaticDir {
    Create,
    Remove,
    Rename(PathBuf),
    Move(PathBuf),
    Copy(PathBuf),
}

#[derive(Debug)]
pub enum File {
    Create(PathBuf),
    Remove,

    /// Rename to the given file name.
    Rename(PathBuf),
    Move(PathBuf),
    Copy(PathBuf),
    Corrupt,
    Repair,
    Modify,
}

#[derive(Debug)]
pub enum StaticFile {
    Create,
    Remove,
    Rename(PathBuf),
    Move(PathBuf),
    Copy(PathBuf),
    Corrupt,
    Repair,
    Modify,
}

#[derive(Debug)]
pub enum Manifest {
    Create,
    Remove,

    /// Rename to the given file name.
    Rename(PathBuf),
    Move(PathBuf),
    Copy(PathBuf),
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
    Create(PathBuf),
    Remove,

    /// Rename to the given file name.
    Rename(PathBuf),
    Move(PathBuf),
    Copy(PathBuf),
    Modify,
}

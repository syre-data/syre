//! Resources for [`container commands`](thot_desktop_tauri::commands::container).
use super::types::{MetadataAction, ResourcePropertiesUpdate, TagsAction};
use serde::Serialize;
use std::path::PathBuf;
use thot_core::project::container::ScriptMap;
use thot_core::project::{Container, ContainerProperties, ScriptAssociation};
use thot_core::types::ResourceId;
use thot_local_database::command::container::AddAssetInfo;

/// Arguments for
/// [`load_container_tree`](thot_desktop_tauri::commands::container::load_container_tree).
#[derive(Serialize)]
pub struct LoadContainerTreeArgs {
    /// Root of the container tree.
    pub root: PathBuf,
}

/// Arguments for commands requiring a [`Container`](Container) named `container` only.
#[derive(Serialize)]
pub struct ContainerArgs {
    /// [`Container`](Container) to update.
    pub container: Container,
}

/// Arguments to update a resorce's [`StandardProperties`].
#[derive(Clone, Serialize)]
pub struct UpdatePropertiesArgs {
    /// [`ResourceId`] of the resource to update.
    pub rid: ResourceId,

    /// Updated values.
    pub properties: ContainerProperties, // TODO: Issue with serializing `HashMap` of `metadata`. perform manually.
                                         // See: https://github.com/tauri-apps/tauri/issues/6078
}

/// Arguments to update a resorce's [`StandardProperties`].
#[derive(Clone, Serialize)]
pub struct UpdatePropertiesStringArgs {
    /// [`ResourceId`] of the resource to update.
    pub rid: ResourceId,

    /// Updated values.
    pub properties: String, // TODO: Issue with serializing `HashMap` of `metadata`. perform manually.
                            // Unify with `UpdatePropertiesArgs` once resolved.
                            // See: https://github.com/tauri-apps/tauri/issues/6078
}

/// Arguments for [`new_child`](thot_desktop_tauri::commands::container::new_child).
#[derive(Serialize)]
pub struct NewChildArgs {
    /// Name of the child.
    pub name: String,

    /// [`ResourceId`] of the parent [`Container`](thot_core::project::Container).
    pub parent: ResourceId,
}

/// Arguments to update a [`Container`](thot_core::project::Container)'s
/// [`ScriptAssociation`](thot_core::project::ScriptAssociation)s.
#[derive(Clone, Serialize)]
pub struct UpdateScriptAssociationsArgs {
    /// [`ResourceId`] of the [`Container`](thot_core::project::Container).
    pub rid: ResourceId,

    // @todo: Issue with deserializing `HashMap` in Tauri, send as string.
    // See: https://github.com/tauri-apps/tauri/issues/6078
    /// Updated script associations.
    pub associations: ScriptMap,
}

/// TEMPORARY
///
/// Intermediate value for [`UpdateScriptAssociationsArgs`] while dealing with
/// (https://github.com/tauri-apps/tauri/issues/6078)
#[derive(Clone, Serialize)]
pub struct UpdateScriptAssociationsStringArgs {
    /// [`ResourceId`] of the [`Container`](thot_core::project::Container).
    pub rid: ResourceId,

    // @todo: Issue with deserializing `HashMap` in Tauri, send as string.
    // Unify with `UpdateScriptAssociationsArgs` once resolved.
    // See: https://github.com/tauri-apps/tauri/issues/6078
    /// Updated script associations.
    pub associations: String,
}

/// Arguments for [`add_assets`](thot_desktop_tauri::commands::container::add_assets).
#[derive(Serialize, Debug)]
pub struct AddAssetsArgs {
    /// [`ResourceId`] of the [`Container`](Container).
    pub container: ResourceId,

    /// [`Asset`](thot_core::project::Asset)s to add.
    pub assets: Vec<AddAssetInfo>,
}

/// Arguments for [`add_asset_windows`](thot_desktop_tauri::commands::container::add_asset_windows).
#[derive(Serialize)]
pub struct AddAssetWindowsArgs {
    /// [`ResourceId`] of the [`Container`](Container).
    pub container: ResourceId,

    /// Name of the file.
    pub name: String,

    /// File contents.
    pub contents: Vec<u8>,
}

/// Bulk update resources.
#[derive(Clone, Serialize)]
pub struct BulkUpdatePropertiesArgs {
    pub rids: Vec<ResourceId>,
    pub update: ContainerPropertiesUpdate,
}

#[derive(Serialize, Clone, Default, Debug)]
pub struct ContainerPropertiesUpdate {
    pub name: Option<String>,
    pub kind: Option<Option<String>>,
    pub description: Option<Option<String>>,
    pub tags: TagsAction,
    pub metadata: MetadataAction,
}

impl From<ResourcePropertiesUpdate> for ContainerPropertiesUpdate {
    fn from(update: ResourcePropertiesUpdate) -> Self {
        Self {
            name: update.name,
            kind: update.kind,
            description: update.description,
            tags: update.tags,
            metadata: update.metadata,
        }
    }
}

/// Arguments for [`bulk_update_container_script_association`](thot_desktop_tauri::commands::container::bulk_update_container_script_association).
#[derive(Serialize, Clone)]
pub struct BulkUpdateScriptAssociationArgs {
    pub containers: Vec<ResourceId>,
    pub update: ScriptAssociationsBulkUpdate,
}

/// Update action used for [`BulkUpdateScriptAssociationArgs`].
#[derive(Serialize, Default, Clone)]
pub struct ScriptAssociationsBulkUpdate {
    /// Associations to insert.
    pub add: Vec<ScriptAssociation>,

    /// Scripts to remove.
    pub remove: Vec<ResourceId>,

    /// Associations to update.
    pub update: Vec<RunParametersUpdate>,
}

#[derive(Serialize, Clone)]
pub struct RunParametersUpdate {
    pub script: ResourceId,
    pub autorun: Option<bool>,
    pub priority: Option<i32>,
}

impl RunParametersUpdate {
    pub fn new(script: ResourceId) -> Self {
        Self {
            script,
            autorun: None,
            priority: None,
        }
    }
}

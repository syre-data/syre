//! Container.
use super::container_properties::{Builder as PropertiesBuilder, ContainerProperties};
use super::Metadata;
use super::{AnalysisAssociation, Asset, RunParameters};
use crate::db::Resource;
use crate::types::Creator;
use crate::types::{ResourceId, ResourceMap};
use chrono::prelude::*;
use has_id::HasId;
use serde_json::Value as JsValue;
use std::hash::{Hash, Hasher};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

// *************
// *** types ***
// *************

pub type AssetMap = ResourceMap<Asset>;
pub type AnalysisMap = ResourceMap<RunParameters>;

// *****************
// *** Container ***
// *****************

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(PartialEq, Eq, Clone, Debug, HasId)]
pub struct Container {
    #[id]
    pub rid: ResourceId,
    pub properties: ContainerProperties,
    pub assets: AssetMap,
    pub analyses: AnalysisMap,
}

impl Container {
    pub fn new(name: impl Into<String>) -> Container {
        Container {
            rid: ResourceId::new(),
            properties: ContainerProperties::new(name),
            assets: AssetMap::default(),
            analyses: AnalysisMap::default(),
        }
    }

    /// Inserts an [`Asset`] into the [`Container`].
    pub fn insert_asset(&mut self, asset: Asset) -> Option<Asset> {
        self.assets.insert(asset.rid.clone(), asset)
    }

    /// Inserts an [`Asset`] into the [`Container`].
    pub fn remove_asset(&mut self, rid: &ResourceId) -> Option<Asset> {
        self.assets.remove(rid)
    }
}

impl Hash for Container {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.rid.hash(state);
    }
}

impl Resource for Container {}

// ***************
// *** Builder ***
// ***************

#[derive(Default)]
pub struct Builder {
    properties: PropertiesBuilder,
    assets: AssetMap,
    analyses: AnalysisMap,
}

impl Builder {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            properties: PropertiesBuilder::new(name),
            assets: AssetMap::new(),
            analyses: AnalysisMap::new(),
        }
    }

    pub fn set_created(&mut self, value: DateTime<Utc>) -> &mut Self {
        self.properties.set_created(value);
        self
    }

    pub fn clear_created(&mut self) -> &mut Self {
        self.properties.clear_created();
        self
    }

    pub fn set_creator(&mut self, value: Creator) -> &mut Self {
        self.properties.set_creator(value);
        self
    }

    pub fn set_name(&mut self, value: impl Into<String>) -> &mut Self {
        self.properties.set_name(value);
        self
    }

    pub fn set_kind(&mut self, value: impl Into<String>) -> &mut Self {
        self.properties.set_kind(value);
        self
    }

    pub fn clear_kind(&mut self) -> &mut Self {
        self.properties.clear_kind();
        self
    }

    pub fn set_description(&mut self, value: impl Into<String>) -> &mut Self {
        self.properties.set_description(value);
        self
    }

    pub fn clear_description(&mut self) -> &mut Self {
        self.properties.clear_description();
        self
    }

    pub fn set_tags(&mut self, value: Vec<impl Into<String>>) -> &mut Self {
        self.properties.set_tags(value);
        self
    }

    pub fn clear_tags(&mut self) -> &mut Self {
        self.properties.clear_tags();
        self
    }

    pub fn add_tag(&mut self, value: impl Into<String>) -> &mut Self {
        self.properties.add_tag(value);
        self
    }

    pub fn remove_tag(&mut self, value: impl Into<String>) -> &mut Self {
        self.properties.remove_tag(value);
        self
    }

    pub fn set_metadata(&mut self, value: Metadata) -> &mut Self {
        self.properties.set_metadata(value);
        self
    }

    pub fn clear_metadata(&mut self) -> &mut Self {
        self.properties.clear_metadata();
        self
    }

    pub fn set_metadatum(
        &mut self,
        key: impl Into<String>,
        value: impl Into<JsValue>,
    ) -> &mut Self {
        self.properties.set_metadatum(key, value);
        self
    }

    pub fn remove_metadatum(&mut self, key: impl Into<String>) -> &mut Self {
        self.properties.remove_metadatum(key);
        self
    }

    pub fn add_asset(&mut self, asset: Asset) -> &mut Self {
        self.assets.insert(asset.rid.clone(), asset);
        self
    }

    pub fn remove_asset(&mut self, rid: &ResourceId) -> &mut Self {
        self.assets.remove(rid);
        self
    }

    pub fn add_analysis(&mut self, script: AnalysisAssociation) -> &mut Self {
        self.analyses.insert(script.analysis.clone(), script.into());
        self
    }

    pub fn remove_analysis(&mut self, rid: &ResourceId) -> &mut Self {
        self.analyses.remove(rid);
        self
    }

    pub fn build(self) -> Container {
        self.into()
    }
}

impl Into<Container> for Builder {
    fn into(self) -> Container {
        Container {
            rid: ResourceId::new(),
            properties: self.properties.into(),
            assets: self.assets,
            analyses: self.analyses,
        }
    }
}

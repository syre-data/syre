use super::Metadata;
use crate::types::Creator;
use crate::types::{ResourceId, UserPermissions};
use chrono::prelude::*;
use has_id::HasId;
use std::collections::HashMap;

pub trait StandardProperties {
    fn created(&self) -> &DateTime<Utc>;
    fn creator(&self) -> &Creator;

    fn permissions(&self) -> &HashMap<ResourceId, UserPermissions>;
    fn permissions_mut(&mut self) -> &mut HashMap<ResourceId, UserPermissions>;

    fn name(&self) -> Option<&String>;
    fn set_name(&mut self, name: String);
    fn unset_name(&mut self);

    fn kind(&self) -> Option<&String>;
    fn set_kind(&mut self, kind: String);
    fn unset_kind(&mut self);

    fn description(&self) -> Option<&mut String>;
    fn set_description(&mut self, description: String);
    fn unset_description(&mut self);

    fn tags(&self) -> &Vec<&String>;
    fn tags_mut(&mut self) -> &mut Vec<&String>;

    fn metadata(&self) -> &Metadata;
    fn metadata_mut(&mut self) -> &mut Metadata;
}

pub trait StandardResource: HasId + StandardProperties {}

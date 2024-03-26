//! Search functionality.
use super::store::Datastore;
use crate::command::search::{Field, Metadatum, Query, ResourceKind};
use std::{collections::HashMap, hash::Hash};
use syre_core::project::Asset;
use syre_core::types::ResourceId;
use syre_local::project::resources::Container;

pub struct Indices {}

impl Indices {
    pub fn new() -> Self {
        Self {}
    }

    pub fn search(&self, store: &Datastore, query: String) -> Vec<ResourceId> {
        vec![]
    }

    pub fn query(&self, store: &Datastore, query: Query) -> Vec<ResourceId> {
        let resource_kind = &query.resource_kind;
        let resources = match query.project.as_ref() {
            None => {
                let mut resources = vec![];
                for graph in store.graphs().values() {
                    for container in graph.nodes().values() {
                        if resource_kind.is_none()
                            || resource_kind == &Some(ResourceKind::Container)
                        {
                            resources.push(Resource::Container(container));
                        }

                        if resource_kind.is_none() || resource_kind == &Some(ResourceKind::Asset) {
                            for asset in container.assets.values() {
                                resources.push(Resource::Asset(asset));
                            }
                        }
                    }
                }

                resources
            }

            Some(project) => {
                let Some(graph) = store.get_project_graph(project) else {
                    return vec![];
                };

                let mut resources = vec![];
                for container in graph.nodes().values() {
                    if resource_kind.is_none() || resource_kind == &Some(ResourceKind::Container) {
                        resources.push(Resource::Container(container));
                    }

                    if resource_kind.is_none() || resource_kind == &Some(ResourceKind::Asset) {
                        for asset in container.assets.values() {
                            resources.push(Resource::Asset(asset));
                        }
                    }
                }

                resources
            }
        };

        match query.select {
            Field::Name(name) => match name.as_ref() {
                None => resources
                    .into_iter()
                    .filter_map(|resource| match resource {
                        Resource::Container(_) => None,
                        Resource::Asset(asset) => match asset.properties.name {
                            None => Some(asset.rid.clone()),
                            Some(_) => None,
                        },
                    })
                    .collect(),

                Some(name) => resources
                    .into_iter()
                    .filter_map(|resource| match resource {
                        Resource::Container(container) => {
                            if &container.properties.name == name {
                                Some(container.rid.clone())
                            } else {
                                None
                            }
                        }

                        Resource::Asset(asset) => match &asset.properties.name {
                            Some(asset_name) if asset_name == name => Some(asset.rid.clone()),
                            _ => None,
                        },
                    })
                    .collect(),
            },

            _ => todo!(),
        }
    }
}

enum Resource<'a> {
    Container(&'a Container),
    Asset(&'a Asset),
}

//  -----------------
// type Index<K, V> = HashMap<K, Vec<V>>;

// #[derive(Debug)]
// struct OptionalIndex<K, V>
// where
//     K: Hash,
// {
//     pub none: Vec<V>,
//     pub some: Index<K, V>,
// }

// impl<K, V> OptionalIndex<K, V>
// where
//     K: Hash,
// {
//     pub fn new() -> Self {
//         Self::default()
//     }
// }

// impl<K, V> Default for OptionalIndex<K, V>
// where
//     K: Hash,
// {
//     fn default() -> Self {
//         Self {
//             none: Vec::new(),
//             some: HashMap::new(),
//         }
//     }
// }

// #[derive(Default, Debug)]
// struct PropertyIndices {
//     pub names: OptionalIndex<String, ResourceId>,
//     pub kinds: OptionalIndex<String, ResourceId>,
//     pub tags: Index<String, ResourceId>,
//     pub descriptions: OptionalIndex<String, ResourceId>,
//     pub metadata: Index<String, ResourceId>,
// }

// impl PropertyIndices {
//     pub fn new() -> Self {
//         Self::default()
//     }
// }

// #[derive(Default, Debug)]
// struct ContainerIndices {
//     /// Map from analysis id to container id.
//     pub analyses: HashMap<ResourceId, ResourceId>,
// }

// #[derive(Default, Debug)]
// pub struct Indices {
//     properties: PropertyIndices,
//     container: ContainerIndices,
// }

// impl Indices {
//     pub fn new() -> Self {
//         Self::default()
//     }

//     pub fn query(&self, query: Query) -> Vec<&ResourceId> {
//         match query.select {
//             Field::Name(name) => match name.as_ref() {
//                 None => self.properties.names.none.iter().collect(),
//                 Some(name) => match self.properties.names.some.get(name) {
//                     None => vec![],
//                     Some(rids) => rids.iter().collect(),
//                 },
//             },

//             _ => todo!(),
//         }
//     }
// }

#[cfg(test)]
#[path = "./search_test.rs"]
mod search_test;

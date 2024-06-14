//! Handle query commands.
use super::super::Database;
use crate::command::search::{Field, Query};
use crate::command::SearchCommand;
use serde_json::Value as JsValue;
use syre_core::types::ResourceId;

impl Database {
    #[tracing::instrument(skip(self))]
    pub fn handle_command_search(&self, command: SearchCommand) -> JsValue {
        match command {
            SearchCommand::Search(query) => {
                let res = self.handle_search_search(query);
                serde_json::to_value(&res).unwrap()
            }

            SearchCommand::Query(query) => {
                let res = self.handle_search_query(query);
                serde_json::to_value(&res).unwrap()
            }
        }
    }

    fn handle_search_search(&self, query: String) -> Vec<ResourceId> {
        self.data_store.search(query).unwrap()
    }

    fn handle_search_query(&self, query: Query) -> Vec<ResourceId> {
        todo!();
        // let resource_kind = &query.resource_kind;
        // let resources = match query.project.as_ref() {
        //     None => {
        //         let mut resources = vec![];
        //         for graph in self.object_store.graphs().values() {
        //             for container in graph.nodes().values() {
        //                 if resource_kind.is_none()
        //                     || resource_kind == &Some(ResourceKind::Container)
        //                 {
        //                     resources.push(Resource::Container(container));
        //                 }

        //                 if resource_kind.is_none() || resource_kind == &Some(ResourceKind::Asset) {
        //                     for asset in container.assets.values() {
        //                         resources.push(Resource::Asset(asset));
        //                     }
        //                 }
        //             }
        //         }

        //         resources
        //     }

        //     Some(project) => {
        //         let Some(graph) = store.get_project_graph(project) else {
        //             return vec![];
        //         };

        //         let mut resources = vec![];
        //         for container in graph.nodes().values() {
        //             if resource_kind.is_none() || resource_kind == &Some(ResourceKind::Container) {
        //                 resources.push(Resource::Container(container));
        //             }

        //             if resource_kind.is_none() || resource_kind == &Some(ResourceKind::Asset) {
        //                 for asset in container.assets.values() {
        //                     resources.push(Resource::Asset(asset));
        //                 }
        //             }
        //         }

        //         resources
        //     }
        // };

        // match query.select {
        //     Field::Name(name) => match name.as_ref() {
        //         None => resources
        //             .into_iter()
        //             .filter_map(|resource| match resource {
        //                 Resource::Container(_) => None,
        //                 Resource::Asset(asset) => match asset.properties.name {
        //                     None => Some(asset.rid.clone()),
        //                     Some(_) => None,
        //                 },
        //             })
        //             .collect(),

        //         Some(name) => resources
        //             .into_iter()
        //             .filter_map(|resource| match resource {
        //                 Resource::Container(container) => {
        //                     if &container.properties.name == name {
        //                         Some(container.rid.clone())
        //                     } else {
        //                         None
        //                     }
        //                 }

        //                 Resource::Asset(asset) => match &asset.properties.name {
        //                     Some(asset_name) if asset_name == name => Some(asset.rid.clone()),
        //                     _ => None,
        //                 },
        //             })
        //             .collect(),
        //     },

        //     _ => todo!(),
        // }
    }
}

// enum Resource<'a> {
//     Container(&'a Container),
//     Asset(&'a Asset),
// }

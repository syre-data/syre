use crate::{event as update, server, state, Database, Update};
use std::{assert_matches::assert_matches, path::Path};
use syre_fs_watcher::{event, EventKind};
use syre_local::TryReducible;

impl Database {
    pub(super) fn handle_fs_event_asset_file(
        &mut self,
        event: syre_fs_watcher::Event,
    ) -> Vec<Update> {
        let EventKind::AssetFile(kind) = event.kind() else {
            panic!("invalid event kind");
        };

        match kind {
            syre_fs_watcher::event::ResourceEvent::Created => {
                self.handle_fs_event_asset_file_created(event)
            }
            syre_fs_watcher::event::ResourceEvent::Removed => {
                self.handle_fs_event_asset_file_removed(event)
            }
            syre_fs_watcher::event::ResourceEvent::Renamed => todo!(),
            syre_fs_watcher::event::ResourceEvent::Moved => todo!(),
            syre_fs_watcher::event::ResourceEvent::MovedProject => todo!(),
            syre_fs_watcher::event::ResourceEvent::Modified(_) => todo!(),
        }
    }
}

impl Database {
    fn handle_fs_event_asset_file_created(&mut self, event: syre_fs_watcher::Event) -> Vec<Update> {
        assert_matches!(
            event.kind(),
            EventKind::AssetFile(event::ResourceEvent::Created)
        );

        let [path] = &event.paths()[..] else {
            panic!("invalid paths");
        };

        let project = self.state.find_resource_project_by_path(path).unwrap();
        let state::FolderResource::Present(project_state) = project.fs_resource() else {
            panic!("invalid state");
        };

        let state::FolderResource::Present(graph) = project_state.graph() else {
            panic!("invalid state");
        };

        let state::DataResource::Ok(project_properties) = project_state.properties() else {
            panic!("invalid state");
        };

        let container_path = path.parent().unwrap();
        let container_graph_path = container_path
            .strip_prefix(project.path().join(&project_properties.data_root))
            .unwrap();
        let container_graph_path = Path::new("/").join(container_graph_path);
        let container_state = graph.find(&container_graph_path).unwrap();
        let container_state = container_state.lock().unwrap();
        let state::DataResource::Ok(assets) = container_state.assets() else {
            return vec![];
        };

        let Some(asset_state) = assets
            .iter()
            .find(|asset| asset.path == path.file_name().unwrap())
        else {
            let path = path
                .strip_prefix(project.path().join(&project_properties.data_root))
                .unwrap();
            let path = Path::new("/").join(path);

            return vec![Update::project_with_id(
                project_properties.rid().clone(),
                project.path().clone(),
                update::Project::AssetFile(update::AssetFile::Created(path)),
                event.id().clone(),
            )];
        };

        assert!(!asset_state.is_present());
        let project_path = project.path().clone();
        let project_id = project_properties.rid().clone();
        let asset_id = asset_state.rid().clone();
        drop(container_state);
        self.state
            .try_reduce(server::state::Action::Project {
                path: project_path.clone(),
                action: server::state::project::Action::Container {
                    path: container_graph_path.clone(),
                    action: server::state::project::action::Container::Asset {
                        rid: asset_id.clone(),
                        action: server::state::project::action::Asset::SetPresent,
                    },
                },
            })
            .unwrap();

        vec![Update::project_with_id(
            project_id.clone(),
            project_path.clone(),
            update::Project::Asset {
                container: container_graph_path,
                asset: asset_id,
                update: update::Asset::FileCreated,
            },
            event.id().clone(),
        )]
    }

    fn handle_fs_event_asset_file_removed(&mut self, event: syre_fs_watcher::Event) -> Vec<Update> {
        assert_matches!(
            event.kind(),
            EventKind::AssetFile(event::ResourceEvent::Removed)
        );

        let [path] = &event.paths()[..] else {
            panic!("invalid paths");
        };

        let base_path = path.parent().unwrap();
        let project = self.state.find_resource_project_by_path(path).unwrap();
        let state::FolderResource::Present(project_state) = project.fs_resource() else {
            panic!("invalid state");
        };

        let state::FolderResource::Present(graph) = project_state.graph() else {
            panic!("invalid state");
        };

        let state::DataResource::Ok(project_properties) = project_state.properties() else {
            panic!("invalid state");
        };

        let container_graph_path = base_path
            .strip_prefix(project.path().join(&project_properties.data_root))
            .unwrap();
        let container_graph_path = Path::new("/").join(container_graph_path);
        let container_state = graph.find(&container_graph_path).unwrap();
        let container_state = container_state.lock().unwrap();
        let state::DataResource::Ok(assets) = container_state.assets() else {
            return vec![];
        };

        let asset_path = path
            .strip_prefix(
                project
                    .path()
                    .join(&project_properties.data_root)
                    .join(&container_graph_path),
            )
            .unwrap();

        let Some(asset_state) = assets.iter().find(|asset| asset.path == asset_path) else {
            let path = path
                .strip_prefix(project.path().join(&project_properties.data_root))
                .unwrap();
            let path = Path::new("/").join(path);

            return vec![Update::project_with_id(
                project_properties.rid().clone(),
                project.path().clone(),
                update::Project::AssetFile(update::AssetFile::Removed(path)),
                event.id().clone(),
            )];
        };

        assert!(asset_state.is_present());
        let project_path = project.path().clone();
        let project_id = project_properties.rid().clone();
        let asset_id = asset_state.rid().clone();
        drop(container_state);
        self.state
            .try_reduce(server::state::Action::Project {
                path: project_path.clone(),
                action: server::state::project::Action::Container {
                    path: container_graph_path.clone(),
                    action: server::state::project::action::Container::Asset {
                        rid: asset_id.clone(),
                        action: server::state::project::action::Asset::SetAbsent,
                    },
                },
            })
            .unwrap();

        vec![Update::project_with_id(
            project_id.clone(),
            project_path.clone(),
            update::Project::Asset {
                container: container_graph_path,
                asset: asset_id,
                update: update::Asset::FileRemoved,
            },
            event.id().clone(),
        )]
    }
}

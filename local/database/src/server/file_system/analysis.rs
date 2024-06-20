use crate::{event as update, server::state, Database, Update};
use std::assert_matches::assert_matches;
use syre_fs_watcher::{event, EventKind};
use syre_local::TryReducible;

impl Database {
    pub(super) fn handle_fs_event_analysis_file(
        &mut self,
        event: syre_fs_watcher::Event,
    ) -> Vec<Update> {
        let EventKind::AnalysisFile(kind) = event.kind() else {
            panic!("invalid event kind");
        };

        match kind {
            syre_fs_watcher::event::ResourceEvent::Created => {
                self.handle_fs_event_analysis_file_created(event)
            }
            syre_fs_watcher::event::ResourceEvent::Removed => {
                self.handle_fs_event_analysis_file_removed(event)
            }
            syre_fs_watcher::event::ResourceEvent::Renamed => todo!(),
            syre_fs_watcher::event::ResourceEvent::Moved => todo!(),
            syre_fs_watcher::event::ResourceEvent::MovedProject => todo!(),
            syre_fs_watcher::event::ResourceEvent::Modified(_) => todo!(),
        }
    }
}

impl Database {
    fn handle_fs_event_analysis_file_created(
        &mut self,
        event: syre_fs_watcher::Event,
    ) -> Vec<Update> {
        assert_matches!(
            event.kind(),
            EventKind::AnalysisFile(event::ResourceEvent::Created)
        );

        let [path] = &event.paths()[..] else {
            panic!("invalid paths");
        };

        let project = self.state.find_resource_project_by_path(path).unwrap();
        let state::project::FolderResource::Present(project_state) = project.fs_resource() else {
            panic!("invalid state");
        };

        let state::project::DataResource::Ok(analyses) = project_state.analyses() else {
            return vec![];
        };

        let state::project::DataResource::Ok(properties) = project_state.properties() else {
            panic!("invalid state");
        };

        let mut analyses_state = analyses.clone();
        let rel_path = path
            .strip_prefix(
                project
                    .path()
                    .join(properties.analysis_root.as_ref().unwrap()),
            )
            .unwrap();

        if let Some(analysis) =
            state::project::analysis::find_analysis_by_path_mut(rel_path, &mut analyses_state)
        {
            assert!(!analysis.is_present());
            analysis.set_present();
            self.state
                .try_reduce(state::Action::Project {
                    path: project.path().clone(),
                    action: state::project::Action::SetAnalyses(state::project::DataResource::Ok(
                        analyses_state,
                    )),
                })
                .unwrap();

            vec![]
        } else {
            vec![Update::project_with_id(
                properties.rid.clone(),
                project.path().clone(),
                update::AnalysisFile::Created(path.clone()).into(),
                event.id().clone(),
            )]
        }
    }

    fn handle_fs_event_analysis_file_removed(
        &mut self,
        event: syre_fs_watcher::Event,
    ) -> Vec<Update> {
        assert_matches!(
            event.kind(),
            EventKind::AnalysisFile(event::ResourceEvent::Removed)
        );

        let [path] = &event.paths()[..] else {
            panic!("invalid paths");
        };

        let project = self.state.find_resource_project_by_path(path).unwrap();
        let state::project::FolderResource::Present(project_state) = project.fs_resource() else {
            panic!("invalid state");
        };

        let state::project::DataResource::Ok(analyses) = project_state.analyses() else {
            panic!("invalid state");
        };

        let state::project::DataResource::Ok(properties) = project_state.properties() else {
            panic!("invalid state");
        };

        let mut analyses_state = analyses.clone();
        let rel_path = path
            .strip_prefix(
                project
                    .path()
                    .join(properties.analysis_root.as_ref().unwrap()),
            )
            .unwrap();

        if let Some(analysis) =
            state::project::analysis::find_analysis_by_path_mut(rel_path, &mut analyses_state)
        {
            assert!(analysis.is_present());
            analysis.set_absent();

            let project_path = project.path().clone();
            let project_id = properties.rid.clone();
            self.state
                .try_reduce(state::Action::Project {
                    path: project_path.clone(),
                    action: state::project::Action::SetAnalyses(state::project::DataResource::Ok(
                        analyses_state,
                    )),
                })
                .unwrap();

            vec![Update::project_with_id(
                project_id,
                project_path,
                update::AnalysisFile::Removed(path.clone()).into(),
                event.id().clone(),
            )]
        } else {
            vec![]
        }
    }
}

use super::*;
use dev_utils::project::{Build, Fireworks, Options, Project};
use std::assert_matches::assert_matches;
use syre_core::graph::ResourceTree;
use syre_local::{
    common as local_common,
    project::resources::{Container, Project as LocalProject},
};

type ContainerTree = ResourceTree<Container>;

#[test_log::test]
fn watcher_group_notify_events_should_work() {
    let dir = tempfile::TempDir::new().unwrap();
    let dir_sub = tempfile::TempDir::new_in(dir.path()).unwrap();
    let f_from = tempfile::NamedTempFile::new_in(dir_sub.path()).unwrap();
    let f_any_from = tempfile::NamedTempFile::new_in(dir_sub.path()).unwrap();
    let f_alone = tempfile::NamedTempFile::new_in(dir.path()).unwrap();
    let d_from = tempfile::TempDir::new_in(dir_sub.path()).unwrap();
    let d_any_from = tempfile::TempDir::new_in(dir_sub.path()).unwrap();
    let d_alone = tempfile::TempDir::new_in(dir.path()).unwrap();

    let e_f_from = notify::Event::new(NotifyEventKind::Modify(ModifyKind::Name(RenameMode::From)))
        .add_path(f_from.path().to_path_buf());

    let e_f_any_from =
        notify::Event::new(NotifyEventKind::Modify(ModifyKind::Name(RenameMode::Any)))
            .add_path(f_any_from.path().to_path_buf());

    let e_f_alone = notify::Event::new(NotifyEventKind::Modify(ModifyKind::Name(RenameMode::To)))
        .add_path(f_alone.path().to_path_buf());

    let e_d_from = notify::Event::new(NotifyEventKind::Modify(ModifyKind::Name(RenameMode::From)))
        .add_path(d_from.path().to_path_buf());

    let e_d_any_from =
        notify::Event::new(NotifyEventKind::Modify(ModifyKind::Name(RenameMode::Any)))
            .add_path(d_any_from.path().to_path_buf());

    let e_d_alone = notify::Event::new(NotifyEventKind::Modify(ModifyKind::Name(RenameMode::To)))
        .add_path(d_alone.path().to_path_buf());

    let (_, command_rx) = crossbeam::channel::unbounded();
    let (event_tx, _) = crossbeam::channel::unbounded();
    let watcher = FsWatcher::new(command_rx, event_tx);
    watcher.handle_command(Command::Watch(dir.path().to_path_buf()));

    let mut f_to_path = f_from.path().to_path_buf();
    let mut f_any_to_path = f_any_from.path().to_path_buf();
    let mut d_to_path = d_from.path().to_path_buf();
    let mut d_any_to_path = d_any_from.path().to_path_buf();
    f_to_path.set_file_name(format!("{:?}-to", f_to_path.file_name().unwrap()));
    f_any_to_path.set_file_name(format!("{:?}-to", f_any_to_path.file_name().unwrap()));
    d_to_path.set_file_name(format!("{:?}-to", d_to_path.file_name().unwrap()));
    d_any_to_path.set_file_name(format!("{:?}-to", d_any_to_path.file_name().unwrap()));
    fs::rename(f_from.path(), &f_to_path).unwrap();
    fs::rename(f_any_from.path(), &f_any_to_path).unwrap();
    fs::rename(d_from.path(), &d_to_path).unwrap();
    fs::rename(d_any_from.path(), &d_any_to_path).unwrap();

    let e_f_to = notify::Event::new(NotifyEventKind::Modify(ModifyKind::Name(RenameMode::To)))
        .add_path(f_to_path.clone());

    let e_f_any_to = notify::Event::new(NotifyEventKind::Modify(ModifyKind::Name(RenameMode::Any)))
        .add_path(f_any_to_path.clone());

    let e_d_to = notify::Event::new(NotifyEventKind::Modify(ModifyKind::Name(RenameMode::To)))
        .add_path(d_to_path.clone());

    let e_d_any_to = notify::Event::new(NotifyEventKind::Modify(ModifyKind::Name(RenameMode::Any)))
        .add_path(d_any_to_path.clone());

    let events = vec![
        e_f_from,
        e_f_to,
        e_f_alone,
        e_d_from,
        e_d_to,
        e_d_alone,
        e_f_any_from,
        e_f_any_to,
        e_d_any_from,
        e_d_any_to,
    ];

    let events: Vec<DebouncedEvent> = events.into_iter().map(|e| e.into()).collect();
    let (converted, remaining) = watcher.group_events(events.iter().collect());
    assert_eq!(converted.len(), 4);
    assert_eq!(remaining.len(), 2);
}

#[test_log::test]
fn watcher_convert_fs_events_should_work() {
    let options = Options::new()
        .with_fs()
        .with_assets()
        .with_asset_files()
        .with_analysis()
        .with_analysis_files();

    let dir = tempfile::tempdir().unwrap();
    let dir_path = std::fs::canonicalize(dir.path()).unwrap();
    let project: Project<LocalProject, ContainerTree> =
        Fireworks::build_fs(&options, dir_path.clone()).unwrap();

    let (_, command_rx) = crossbeam::channel::unbounded();
    let (event_tx, _) = crossbeam::channel::unbounded();
    let watcher = FsWatcher::new(command_rx, event_tx);
    watcher.handle_command(Command::Watch(dir_path.clone()));

    convert_fs::test_config(&watcher);
    convert_fs::test_project_simple(&watcher, &project);
    convert_fs::test_graph(&watcher, &project);
    convert_fs::test_container(&watcher, &project);
    convert_fs::test_asset_files_simple(&watcher, &project);
    convert_fs::test_analysis_files_simple(&watcher, &project);
    convert_fs::test_files_simple(&watcher, &project);
    convert_fs::test_folders_simple(&watcher, &project);
}

mod convert_fs {
    use super::*;
    use syre_local::{file_resource::SystemResource, system::collections};

    pub fn test_config(watcher: &FsWatcher) {
        // -- created
        let events = watcher
            .process_event_fs_to_apps(&fs_event::Event::new(
                fs_event::File::Created(collections::ProjectManifest::path().unwrap()),
                Instant::now(),
            ))
            .unwrap();

        assert_eq!(events.len(), 1);
        assert_matches!(
            *events[0].kind(),
            EventKind::Config(app::Config::ProjectManifest(
                app::StaticResourceEvent::Created
            ))
        );

        let events = watcher
            .process_event_fs_to_apps(&fs_event::Event::new(
                fs_event::File::Created(collections::UserManifest::path().unwrap()),
                Instant::now(),
            ))
            .unwrap();

        assert_eq!(events.len(), 1);
        assert_matches!(
            *events[0].kind(),
            EventKind::Config(app::Config::UserManifest(app::StaticResourceEvent::Created))
        );
        // -- created end

        // -- removed
        let events = watcher
            .process_event_fs_to_apps(&fs_event::Event::new(
                fs_event::File::Removed(collections::ProjectManifest::path().unwrap()),
                Instant::now(),
            ))
            .unwrap();

        assert_eq!(events.len(), 1);
        assert_matches!(
            *events[0].kind(),
            EventKind::Config(app::Config::ProjectManifest(
                app::StaticResourceEvent::Removed
            ))
        );

        let events = watcher
            .process_event_fs_to_apps(&fs_event::Event::new(
                fs_event::File::Removed(collections::UserManifest::path().unwrap()),
                Instant::now(),
            ))
            .unwrap();

        assert_eq!(events.len(), 1);
        assert_matches!(
            *events[0].kind(),
            EventKind::Config(app::Config::UserManifest(app::StaticResourceEvent::Removed))
        );
        // -- removed end

        // -- modified
        let events = watcher
            .process_event_fs_to_apps(&fs_event::Event::new(
                fs_event::File::DataModified(collections::ProjectManifest::path().unwrap()),
                Instant::now(),
            ))
            .unwrap();

        assert_eq!(events.len(), 1);
        assert_matches!(
            *events[0].kind(),
            EventKind::Config(app::Config::ProjectManifest(
                app::StaticResourceEvent::Modified(app::ModifiedKind::Data)
            ))
        );

        let events = watcher
            .process_event_fs_to_apps(&fs_event::Event::new(
                fs_event::File::DataModified(collections::UserManifest::path().unwrap()),
                Instant::now(),
            ))
            .unwrap();

        assert_eq!(events.len(), 1);
        assert_matches!(
            *events[0].kind(),
            EventKind::Config(app::Config::UserManifest(
                app::StaticResourceEvent::Modified(app::ModifiedKind::Data)
            ))
        );
        // -- modified end
    }

    pub fn test_project_simple(
        watcher: &FsWatcher,
        project: &Project<LocalProject, ContainerTree>,
    ) {
        // -- project folder
        let events = watcher
            .process_event_fs_to_apps(&fs_event::Event::new(
                fs_event::Folder::Removed(project.project.base_path().to_path_buf()),
                Instant::now(),
            ))
            .unwrap();

        assert_eq!(events.len(), 1);
        assert_matches!(*events[0].kind(), EventKind::Project(app::Project::Removed));

        let events = watcher
            .process_event_fs_to_apps(&fs_event::Event::new(
                fs_event::Folder::Moved {
                    from: tempfile::tempdir().unwrap().into_path(),
                    to: project.project.base_path().to_path_buf(),
                },
                Instant::now(),
            ))
            .unwrap();

        assert_eq!(events.len(), 1);
        assert_matches!(*events[0].kind(), EventKind::Project(app::Project::Moved));
        // -- project folder end

        // -- config dir
        let events = watcher
            .process_event_fs_to_apps(&fs_event::Event::new(
                fs_event::Folder::Created(local_common::app_dir_of(project.project.base_path())),
                Instant::now(),
            ))
            .unwrap();

        assert_eq!(events.len(), 1);
        assert_matches!(
            *events[0].kind(),
            EventKind::Project(app::Project::ConfigDir(app::StaticResourceEvent::Created))
        );

        let events = watcher
            .process_event_fs_to_apps(&fs_event::Event::new(
                fs_event::Folder::Removed(local_common::app_dir_of(project.project.base_path())),
                Instant::now(),
            ))
            .unwrap();

        assert_eq!(events.len(), 1);
        assert_matches!(
            *events[0].kind(),
            EventKind::Project(app::Project::ConfigDir(app::StaticResourceEvent::Removed))
        );

        let events = watcher
            .process_event_fs_to_apps(&fs_event::Event::new(
                fs_event::Folder::Moved {
                    from: tempfile::tempdir().unwrap().into_path(),
                    to: local_common::app_dir_of(project.project.base_path()),
                },
                Instant::now(),
            ))
            .unwrap();

        assert_eq!(events.len(), 1);
        assert_matches!(
            *events[0].kind(),
            EventKind::Project(app::Project::ConfigDir(app::StaticResourceEvent::Modified(
                app::ModifiedKind::Other
            )))
        );
        // -- config dir end

        // -- analysis folder
        let analysis_path = project.project.analysis_root_path().unwrap();
        let mut analysis_path_renamed = analysis_path.clone();
        analysis_path_renamed.set_file_name("test");

        let events = watcher
            .process_event_fs_to_apps(&fs_event::Event::new(
                fs_event::Folder::Created(analysis_path.clone()),
                Instant::now(),
            ))
            .unwrap();

        assert_eq!(events.len(), 1);
        assert_matches!(
            *events[0].kind(),
            EventKind::Project(app::Project::AnalysisDir(app::ResourceEvent::Created))
        );

        let events = watcher
            .process_event_fs_to_apps(&fs_event::Event::new(
                fs_event::Folder::Removed(analysis_path.clone()),
                Instant::now(),
            ))
            .unwrap();

        assert_eq!(events.len(), 1);
        assert_matches!(
            *events[0].kind(),
            EventKind::Project(app::Project::AnalysisDir(app::ResourceEvent::Removed))
        );

        let events = watcher
            .process_event_fs_to_apps(&fs_event::Event::new(
                fs_event::Folder::Renamed {
                    from: analysis_path.clone(),
                    to: analysis_path_renamed.clone(),
                },
                Instant::now(),
            ))
            .unwrap();

        assert_eq!(events.len(), 1);
        assert_matches!(
            *events[0].kind(),
            EventKind::Project(app::Project::AnalysisDir(app::ResourceEvent::Renamed))
        );

        let events = watcher
            .process_event_fs_to_apps(&fs_event::Event::new(
                fs_event::Folder::Moved {
                    from: tempfile::tempdir().unwrap().into_path(),
                    to: analysis_path.clone(),
                },
                Instant::now(),
            ))
            .unwrap();

        assert_eq!(events.len(), 1);
        assert_matches!(
            *events[0].kind(),
            EventKind::Project(app::Project::AnalysisDir(app::ResourceEvent::Modified(
                app::ModifiedKind::Other
            )))
        );

        let events = watcher
            .process_event_fs_to_apps(&fs_event::Event::new(
                fs_event::Folder::Moved {
                    from: analysis_path.clone(),
                    to: project.project.base_path().join("test").join("test"),
                },
                Instant::now(),
            ))
            .unwrap();

        assert_eq!(events.len(), 1);
        assert_matches!(
            *events[0].kind(),
            EventKind::Project(app::Project::AnalysisDir(app::ResourceEvent::Moved))
        );

        let events = watcher
            .process_event_fs_to_apps(&fs_event::Event::new(
                fs_event::Folder::Moved {
                    from: analysis_path.clone(),
                    to: tempfile::tempdir().unwrap().into_path(),
                },
                Instant::now(),
            ))
            .unwrap();

        assert_eq!(events.len(), 1);
        assert_matches!(
            *events[0].kind(),
            EventKind::Project(app::Project::AnalysisDir(app::ResourceEvent::Removed))
        );
        // -- analysis folder end

        // --  data folder
        let data_path = project.project.data_root_path();
        let mut data_path_renamed = data_path.clone();
        data_path_renamed.set_file_name("test");

        let events = watcher
            .process_event_fs_to_apps(&fs_event::Event::new(
                fs_event::Folder::Created(data_path.clone()),
                Instant::now(),
            ))
            .unwrap();

        assert_eq!(events.len(), 1);
        assert_matches!(
            *events[0].kind(),
            EventKind::Project(app::Project::DataDir(app::ResourceEvent::Created))
        );

        let events = watcher
            .process_event_fs_to_apps(&fs_event::Event::new(
                fs_event::Folder::Removed(data_path.clone()),
                Instant::now(),
            ))
            .unwrap();

        assert_eq!(events.len(), 1);
        assert_matches!(
            *events[0].kind(),
            EventKind::Project(app::Project::DataDir(app::ResourceEvent::Removed))
        );

        let events = watcher
            .process_event_fs_to_apps(&fs_event::Event::new(
                fs_event::Folder::Renamed {
                    from: data_path.clone(),
                    to: data_path_renamed.clone(),
                },
                Instant::now(),
            ))
            .unwrap();

        assert_eq!(events.len(), 1);
        assert_matches!(
            *events[0].kind(),
            EventKind::Project(app::Project::DataDir(app::ResourceEvent::Renamed))
        );

        let events = watcher
            .process_event_fs_to_apps(&fs_event::Event::new(
                fs_event::Folder::Moved {
                    from: tempfile::tempdir().unwrap().into_path(),
                    to: data_path.clone(),
                },
                Instant::now(),
            ))
            .unwrap();

        assert_eq!(events.len(), 1);
        assert_matches!(
            *events[0].kind(),
            EventKind::Project(app::Project::DataDir(app::ResourceEvent::Modified(
                app::ModifiedKind::Other
            )))
        );

        let events = watcher
            .process_event_fs_to_apps(&fs_event::Event::new(
                fs_event::Folder::Moved {
                    from: data_path.clone(),
                    to: project.project.base_path().join("test").join("test"),
                },
                Instant::now(),
            ))
            .unwrap();

        assert_eq!(events.len(), 1);
        assert_matches!(
            *events[0].kind(),
            EventKind::Project(app::Project::DataDir(app::ResourceEvent::Moved))
        );

        let events = watcher
            .process_event_fs_to_apps(&fs_event::Event::new(
                fs_event::Folder::Moved {
                    from: data_path.clone(),
                    to: tempfile::tempdir().unwrap().into_path(),
                },
                Instant::now(),
            ))
            .unwrap();

        assert_eq!(events.len(), 1);
        assert_matches!(
            *events[0].kind(),
            EventKind::Project(app::Project::DataDir(app::ResourceEvent::Removed))
        );
        // --  data folder end

        // -- properties
        let properties_file = local_common::project_file_of(project.project.base_path());
        let mut properties_file_renamed = properties_file.clone();
        properties_file_renamed.set_file_name("test.txt");

        let events = watcher
            .process_event_fs_to_apps(&fs_event::Event::new(
                fs_event::File::Created(properties_file.clone()),
                Instant::now(),
            ))
            .unwrap();

        assert_eq!(events.len(), 1);
        assert_matches!(
            *events[0].kind(),
            EventKind::Project(app::Project::Properties(app::StaticResourceEvent::Created))
        );

        let events = watcher
            .process_event_fs_to_apps(&fs_event::Event::new(
                fs_event::File::Removed(properties_file.clone()),
                Instant::now(),
            ))
            .unwrap();

        assert_eq!(events.len(), 1);
        assert_matches!(
            *events[0].kind(),
            EventKind::Project(app::Project::Properties(app::StaticResourceEvent::Removed))
        );

        let events = watcher
            .process_event_fs_to_apps(&fs_event::Event::new(
                fs_event::File::DataModified(properties_file.clone()),
                Instant::now(),
            ))
            .unwrap();

        assert_eq!(events.len(), 1);
        assert_matches!(
            *events[0].kind(),
            EventKind::Project(app::Project::Properties(
                app::StaticResourceEvent::Modified(app::ModifiedKind::Data)
            ))
        );

        let events = watcher
            .process_event_fs_to_apps(&fs_event::Event::new(
                fs_event::File::Renamed {
                    from: properties_file.clone(),
                    to: properties_file_renamed.clone(),
                },
                Instant::now(),
            ))
            .unwrap();

        assert_eq!(events.len(), 1);
        assert_matches!(
            *events[0].kind(),
            EventKind::Project(app::Project::Properties(app::StaticResourceEvent::Removed))
        );

        let events = watcher
            .process_event_fs_to_apps(&fs_event::Event::new(
                fs_event::File::Renamed {
                    from: properties_file_renamed.clone(),
                    to: properties_file.clone(),
                },
                Instant::now(),
            ))
            .unwrap();

        assert_eq!(events.len(), 1);
        assert_matches!(
            *events[0].kind(),
            EventKind::Project(app::Project::Properties(app::StaticResourceEvent::Created))
        );

        let events = watcher
            .process_event_fs_to_apps(&fs_event::Event::new(
                fs_event::File::Moved {
                    from: properties_file.clone(),
                    to: project
                        .project
                        .base_path()
                        .join(properties_file.file_name().unwrap()),
                },
                Instant::now(),
            ))
            .unwrap();

        assert_eq!(events.len(), 1);
        assert_matches!(
            *events[0].kind(),
            EventKind::Project(app::Project::Properties(app::StaticResourceEvent::Removed))
        );

        let events = watcher
            .process_event_fs_to_apps(&fs_event::Event::new(
                fs_event::File::Moved {
                    from: properties_file.clone(),
                    to: tempfile::tempdir()
                        .unwrap()
                        .into_path()
                        .join(properties_file.file_name().unwrap()),
                },
                Instant::now(),
            ))
            .unwrap();

        assert_eq!(events.len(), 1);
        assert_matches!(
            *events[0].kind(),
            EventKind::Project(app::Project::Properties(app::StaticResourceEvent::Removed))
        );

        let events = watcher
            .process_event_fs_to_apps(&fs_event::Event::new(
                fs_event::File::Moved {
                    from: project
                        .project
                        .base_path()
                        .join(properties_file.file_name().unwrap()),

                    to: properties_file.clone(),
                },
                Instant::now(),
            ))
            .unwrap();

        assert_eq!(events.len(), 1);
        assert_matches!(
            *events[0].kind(),
            EventKind::Project(app::Project::Properties(
                app::StaticResourceEvent::Modified(app::ModifiedKind::Other)
            ))
        );
        // -- properties end

        // -- settings
        let settings_file = local_common::project_settings_file_of(project.project.base_path());
        let mut settings_file_renamed = settings_file.clone();
        settings_file_renamed.set_file_name("test.txt");

        let events = watcher
            .process_event_fs_to_apps(&fs_event::Event::new(
                fs_event::File::Created(settings_file.clone()),
                Instant::now(),
            ))
            .unwrap();

        assert_eq!(events.len(), 1);
        assert_matches!(
            *events[0].kind(),
            EventKind::Project(app::Project::Settings(app::StaticResourceEvent::Created))
        );

        let events = watcher
            .process_event_fs_to_apps(&fs_event::Event::new(
                fs_event::File::Removed(settings_file.clone()),
                Instant::now(),
            ))
            .unwrap();

        assert_eq!(events.len(), 1);
        assert_matches!(
            *events[0].kind(),
            EventKind::Project(app::Project::Settings(app::StaticResourceEvent::Removed))
        );

        let events = watcher
            .process_event_fs_to_apps(&fs_event::Event::new(
                fs_event::File::DataModified(settings_file.clone()),
                Instant::now(),
            ))
            .unwrap();

        assert_eq!(events.len(), 1);
        assert_matches!(
            *events[0].kind(),
            EventKind::Project(app::Project::Settings(app::StaticResourceEvent::Modified(
                app::ModifiedKind::Data
            )))
        );

        let events = watcher
            .process_event_fs_to_apps(&fs_event::Event::new(
                fs_event::File::Renamed {
                    from: settings_file.clone(),
                    to: settings_file_renamed.clone(),
                },
                Instant::now(),
            ))
            .unwrap();

        assert_eq!(events.len(), 1);
        assert_matches!(
            *events[0].kind(),
            EventKind::Project(app::Project::Settings(app::StaticResourceEvent::Removed))
        );

        let events = watcher
            .process_event_fs_to_apps(&fs_event::Event::new(
                fs_event::File::Renamed {
                    from: settings_file_renamed.clone(),
                    to: settings_file.clone(),
                },
                Instant::now(),
            ))
            .unwrap();

        assert_eq!(events.len(), 1);
        assert_matches!(
            *events[0].kind(),
            EventKind::Project(app::Project::Settings(app::StaticResourceEvent::Created))
        );

        let events = watcher
            .process_event_fs_to_apps(&fs_event::Event::new(
                fs_event::File::Moved {
                    from: settings_file.clone(),
                    to: project
                        .project
                        .base_path()
                        .join(settings_file.file_name().unwrap()),
                },
                Instant::now(),
            ))
            .unwrap();

        assert_eq!(events.len(), 1);
        assert_matches!(
            *events[0].kind(),
            EventKind::Project(app::Project::Settings(app::StaticResourceEvent::Removed))
        );

        let events = watcher
            .process_event_fs_to_apps(&fs_event::Event::new(
                fs_event::File::Moved {
                    from: settings_file.clone(),
                    to: tempfile::tempdir()
                        .unwrap()
                        .into_path()
                        .join(settings_file.file_name().unwrap()),
                },
                Instant::now(),
            ))
            .unwrap();

        assert_eq!(events.len(), 1);
        assert_matches!(
            *events[0].kind(),
            EventKind::Project(app::Project::Settings(app::StaticResourceEvent::Removed))
        );

        let events = watcher
            .process_event_fs_to_apps(&fs_event::Event::new(
                fs_event::File::Moved {
                    from: project
                        .project
                        .base_path()
                        .join(settings_file.file_name().unwrap()),

                    to: settings_file.clone(),
                },
                Instant::now(),
            ))
            .unwrap();

        assert_eq!(events.len(), 1);
        assert_matches!(
            *events[0].kind(),
            EventKind::Project(app::Project::Settings(app::StaticResourceEvent::Modified(
                app::ModifiedKind::Other
            )))
        );
        // -- settings end

        // -- analysis
        let analysis_file = local_common::analyses_file_of(project.project.base_path());
        let mut analysis_file_renamed = analysis_file.clone();
        analysis_file_renamed.set_file_name("test.txt");

        let events = watcher
            .process_event_fs_to_apps(&fs_event::Event::new(
                fs_event::File::Created(analysis_file.clone()),
                Instant::now(),
            ))
            .unwrap();

        assert_eq!(events.len(), 1);
        assert_matches!(
            *events[0].kind(),
            EventKind::Project(app::Project::Analysis(app::StaticResourceEvent::Created))
        );

        let events = watcher
            .process_event_fs_to_apps(&fs_event::Event::new(
                fs_event::File::Removed(analysis_file.clone()),
                Instant::now(),
            ))
            .unwrap();

        assert_eq!(events.len(), 1);
        assert_matches!(
            *events[0].kind(),
            EventKind::Project(app::Project::Analysis(app::StaticResourceEvent::Removed))
        );

        let events = watcher
            .process_event_fs_to_apps(&fs_event::Event::new(
                fs_event::File::DataModified(analysis_file.clone()),
                Instant::now(),
            ))
            .unwrap();

        assert_eq!(events.len(), 1);
        assert_matches!(
            *events[0].kind(),
            EventKind::Project(app::Project::Analysis(app::StaticResourceEvent::Modified(
                app::ModifiedKind::Data
            )))
        );

        let events = watcher
            .process_event_fs_to_apps(&fs_event::Event::new(
                fs_event::File::Renamed {
                    from: analysis_file.clone(),
                    to: analysis_file_renamed.clone(),
                },
                Instant::now(),
            ))
            .unwrap();

        assert_eq!(events.len(), 1);
        assert_matches!(
            *events[0].kind(),
            EventKind::Project(app::Project::Analysis(app::StaticResourceEvent::Removed))
        );

        let events = watcher
            .process_event_fs_to_apps(&fs_event::Event::new(
                fs_event::File::Renamed {
                    from: analysis_file_renamed.clone(),
                    to: analysis_file.clone(),
                },
                Instant::now(),
            ))
            .unwrap();

        assert_eq!(events.len(), 1);
        assert_matches!(
            *events[0].kind(),
            EventKind::Project(app::Project::Analysis(app::StaticResourceEvent::Created))
        );

        let events = watcher
            .process_event_fs_to_apps(&fs_event::Event::new(
                fs_event::File::Moved {
                    from: analysis_file.clone(),
                    to: project
                        .project
                        .base_path()
                        .join(analysis_file.file_name().unwrap()),
                },
                Instant::now(),
            ))
            .unwrap();

        assert_eq!(events.len(), 1);
        assert_matches!(
            *events[0].kind(),
            EventKind::Project(app::Project::Analysis(app::StaticResourceEvent::Removed))
        );

        let events = watcher
            .process_event_fs_to_apps(&fs_event::Event::new(
                fs_event::File::Moved {
                    from: analysis_file.clone(),
                    to: tempfile::tempdir()
                        .unwrap()
                        .into_path()
                        .join(analysis_file.file_name().unwrap()),
                },
                Instant::now(),
            ))
            .unwrap();

        assert_eq!(events.len(), 1);
        assert_matches!(
            *events[0].kind(),
            EventKind::Project(app::Project::Analysis(app::StaticResourceEvent::Removed))
        );

        let events = watcher
            .process_event_fs_to_apps(&fs_event::Event::new(
                fs_event::File::Moved {
                    from: project
                        .project
                        .base_path()
                        .join(analysis_file.file_name().unwrap()),

                    to: analysis_file.clone(),
                },
                Instant::now(),
            ))
            .unwrap();

        assert_eq!(events.len(), 1);
        assert_matches!(
            *events[0].kind(),
            EventKind::Project(app::Project::Analysis(app::StaticResourceEvent::Modified(
                app::ModifiedKind::Other
            )))
        );
        // -- analysis end
    }

    pub fn test_graph(watcher: &FsWatcher, project: &Project<LocalProject, ContainerTree>) {
        let data_path = project.project.data_root_path();
        let root = project.graph.get(project.graph.root()).unwrap();
        let children = project.graph.children(&root.rid).unwrap();
        let recipe_1 = project.graph.get(&children[0]).unwrap();
        let recipe_1_path = recipe_1.base_path().to_path_buf();

        let events = watcher
            .process_event_fs_to_apps(&fs_event::Event::new(
                fs_event::Folder::Created(recipe_1_path.clone()),
                Instant::now(),
            ))
            .unwrap();

        assert_eq!(events.len(), 1);
        assert_matches!(*events[0].kind(), EventKind::Graph(app::Graph::Created));

        let events = watcher
            .process_event_fs_to_apps(&fs_event::Event::new(
                fs_event::Folder::Removed(data_path.join("child")),
                Instant::now(),
            ))
            .unwrap();

        assert_eq!(events.len(), 1);
        assert_matches!(
            *events[0].kind(),
            EventKind::Folder(app::ResourceEvent::Removed)
        );

        let events = watcher
            .process_event_fs_to_apps(&fs_event::Event::new(
                fs_event::Folder::Moved {
                    from: data_path.join("child"),
                    to: recipe_1_path.clone(),
                },
                Instant::now(),
            ))
            .unwrap();

        assert_eq!(events.len(), 1);
        assert_matches!(*events[0].kind(), EventKind::Graph(app::Graph::Moved));

        let events = watcher
            .process_event_fs_to_apps(&fs_event::Event::new(
                fs_event::Folder::Moved {
                    from: tempfile::tempdir().unwrap().into_path(),
                    to: recipe_1_path.clone(),
                },
                Instant::now(),
            ))
            .unwrap();

        assert_eq!(events.len(), 1);
        assert_matches!(*events[0].kind(), EventKind::Graph(app::Graph::Created));
    }

    pub fn test_container(watcher: &FsWatcher, project: &Project<LocalProject, ContainerTree>) {
        let root = project.graph.get(project.graph.root()).unwrap();
        let children = project.graph.children(&root.rid).unwrap();
        let recipe_1 = project.graph.get(&children[0]).unwrap();
        let root_path = root.base_path();
        let container_path = recipe_1.base_path().to_path_buf();

        let events = watcher
            .process_event_fs_to_apps(&fs_event::Event::new(
                fs_event::Folder::Renamed {
                    from: root_path.join("test"),
                    to: container_path.clone(),
                },
                Instant::now(),
            ))
            .unwrap();

        assert_eq!(events.len(), 1);
        assert_matches!(
            *events[0].kind(),
            EventKind::Container(app::Container::Renamed)
        );

        let events = watcher
            .process_event_fs_to_apps(&fs_event::Event::new(
                fs_event::Folder::Removed(local_common::app_dir_of(container_path.clone())),
                Instant::now(),
            ))
            .unwrap();

        assert_eq!(events.len(), 1);
        assert_matches!(
            *events[0].kind(),
            EventKind::Container(app::Container::ConfigDir(app::StaticResourceEvent::Removed))
        );

        let events = watcher
            .process_event_fs_to_apps(&fs_event::Event::new(
                fs_event::Folder::Renamed {
                    from: local_common::app_dir_of(container_path.clone()),
                    to: container_path.join("test"),
                },
                Instant::now(),
            ))
            .unwrap();

        assert_eq!(events.len(), 1);
        assert_matches!(
            *events[0].kind(),
            EventKind::Container(app::Container::ConfigDir(
                app::StaticResourceEvent::KindChanged
            ))
        );

        let events = watcher
            .process_event_fs_to_apps(&fs_event::Event::new(
                fs_event::Folder::Renamed {
                    from: container_path.join("test"),
                    to: local_common::app_dir_of(container_path.clone()),
                },
                Instant::now(),
            ))
            .unwrap();

        assert_eq!(events.len(), 1);
        assert_matches!(
            *events[0].kind(),
            EventKind::Container(app::Container::ConfigDir(app::StaticResourceEvent::Created))
        );

        // -- properties
        let events = watcher
            .process_event_fs_to_apps(&fs_event::Event::new(
                fs_event::File::Created(local_common::container_file_of(container_path.clone())),
                Instant::now(),
            ))
            .unwrap();

        assert_eq!(events.len(), 1);
        assert_matches!(
            *events[0].kind(),
            EventKind::Container(app::Container::Properties(
                app::StaticResourceEvent::Created
            ))
        );

        let events = watcher
            .process_event_fs_to_apps(&fs_event::Event::new(
                fs_event::File::Removed(local_common::container_file_of(container_path.clone())),
                Instant::now(),
            ))
            .unwrap();

        assert_eq!(events.len(), 1);
        assert_matches!(
            *events[0].kind(),
            EventKind::Container(app::Container::Properties(
                app::StaticResourceEvent::Removed
            ))
        );

        let events = watcher
            .process_event_fs_to_apps(&fs_event::Event::new(
                fs_event::File::Renamed {
                    from: local_common::app_dir_of(container_path.clone()).join("test"),
                    to: local_common::container_file_of(container_path.clone()),
                },
                Instant::now(),
            ))
            .unwrap();

        assert_eq!(events.len(), 1);
        assert_matches!(
            *events[0].kind(),
            EventKind::Container(app::Container::Properties(
                app::StaticResourceEvent::Created
            ))
        );

        let events = watcher
            .process_event_fs_to_apps(&fs_event::Event::new(
                fs_event::File::Renamed {
                    from: local_common::container_file_of(container_path.clone()),
                    to: local_common::app_dir_of(container_path.clone()).join("test"),
                },
                Instant::now(),
            ))
            .unwrap();

        assert_eq!(events.len(), 1);
        assert_matches!(
            *events[0].kind(),
            EventKind::Container(app::Container::Properties(
                app::StaticResourceEvent::Removed
            ))
        );

        let events = watcher
            .process_event_fs_to_apps(&fs_event::Event::new(
                fs_event::File::DataModified(local_common::container_file_of(
                    container_path.clone(),
                )),
                Instant::now(),
            ))
            .unwrap();

        assert_eq!(events.len(), 1);
        assert_matches!(
            *events[0].kind(),
            EventKind::Container(app::Container::Properties(
                app::StaticResourceEvent::Modified(app::ModifiedKind::Data)
            ))
        );
        // -- properties end

        // -- settings
        let events = watcher
            .process_event_fs_to_apps(&fs_event::Event::new(
                fs_event::File::Created(local_common::container_settings_file_of(
                    container_path.clone(),
                )),
                Instant::now(),
            ))
            .unwrap();

        assert_eq!(events.len(), 1);
        assert_matches!(
            *events[0].kind(),
            EventKind::Container(app::Container::Settings(app::StaticResourceEvent::Created))
        );

        let events = watcher
            .process_event_fs_to_apps(&fs_event::Event::new(
                fs_event::File::Removed(local_common::container_settings_file_of(
                    container_path.clone(),
                )),
                Instant::now(),
            ))
            .unwrap();

        assert_eq!(events.len(), 1);
        assert_matches!(
            *events[0].kind(),
            EventKind::Container(app::Container::Settings(app::StaticResourceEvent::Removed))
        );

        let events = watcher
            .process_event_fs_to_apps(&fs_event::Event::new(
                fs_event::File::Renamed {
                    from: local_common::app_dir_of(container_path.clone()).join("test"),
                    to: local_common::container_settings_file_of(container_path.clone()),
                },
                Instant::now(),
            ))
            .unwrap();

        assert_eq!(events.len(), 1);
        assert_matches!(
            *events[0].kind(),
            EventKind::Container(app::Container::Settings(app::StaticResourceEvent::Created))
        );

        let events = watcher
            .process_event_fs_to_apps(&fs_event::Event::new(
                fs_event::File::Renamed {
                    from: local_common::container_settings_file_of(container_path.clone()),
                    to: local_common::app_dir_of(container_path.clone()).join("test"),
                },
                Instant::now(),
            ))
            .unwrap();

        assert_eq!(events.len(), 1);
        assert_matches!(
            *events[0].kind(),
            EventKind::Container(app::Container::Settings(app::StaticResourceEvent::Removed))
        );

        let events = watcher
            .process_event_fs_to_apps(&fs_event::Event::new(
                fs_event::File::DataModified(local_common::container_settings_file_of(
                    container_path.clone(),
                )),
                Instant::now(),
            ))
            .unwrap();

        assert_eq!(events.len(), 1);
        assert_matches!(
            *events[0].kind(),
            EventKind::Container(app::Container::Settings(
                app::StaticResourceEvent::Modified(app::ModifiedKind::Data)
            ))
        );
        // -- settings end
    }
    pub fn test_asset_files_simple(
        watcher: &FsWatcher,
        project: &Project<LocalProject, ContainerTree>,
    ) {
        let root = project.graph.get(project.graph.root()).unwrap();
        let children = project.graph.children(&root.rid).unwrap();
        let recipe_1 = project.graph.get(&children[0]).unwrap();
        let batch_1 = project.graph.children(&recipe_1.rid).unwrap();
        let batch_11 = project.graph.get(&batch_1[0]).unwrap();
        let batch_12 = project.graph.get(&batch_1[1]).unwrap();
        let asset_11 = batch_11.assets.values().next().unwrap();
        let asset_11_path = batch_11.base_path().join(asset_11.path.clone());

        let events = watcher
            .process_event_fs_to_apps(&fs_event::Event::new(
                fs_event::File::Created(asset_11_path.clone()),
                Instant::now(),
            ))
            .unwrap();

        assert_eq!(events.len(), 1);
        assert_matches!(
            *events[0].kind(),
            EventKind::AssetFile(app::ResourceEvent::Created)
        );

        let events = watcher
            .process_event_fs_to_apps(&fs_event::Event::new(
                fs_event::File::Removed(batch_11.base_path().join("test.txt")),
                Instant::now(),
            ))
            .unwrap();

        assert_eq!(events.len(), 1);
        assert_matches!(
            *events[0].kind(),
            EventKind::AssetFile(app::ResourceEvent::Removed)
        );

        let events = watcher
            .process_event_fs_to_apps(&fs_event::Event::new(
                fs_event::File::Moved {
                    from: batch_12.base_path().join(asset_11.path.clone()),
                    to: asset_11_path.clone(),
                },
                Instant::now(),
            ))
            .unwrap();

        assert_eq!(events.len(), 1);
        assert_matches!(
            *events[0].kind(),
            EventKind::AssetFile(app::ResourceEvent::Moved)
        );

        let events = watcher
            .process_event_fs_to_apps(&fs_event::Event::new(
                fs_event::File::Renamed {
                    from: batch_11.base_path().join("test.txt"),
                    to: asset_11_path.clone(),
                },
                Instant::now(),
            ))
            .unwrap();

        assert_eq!(events.len(), 1);
        assert_matches!(
            *events[0].kind(),
            EventKind::AssetFile(app::ResourceEvent::Renamed)
        );

        let events = watcher
            .process_event_fs_to_apps(&fs_event::Event::new(
                fs_event::File::DataModified(asset_11_path.clone()),
                Instant::now(),
            ))
            .unwrap();

        assert_eq!(events.len(), 1);
        assert_matches!(
            *events[0].kind(),
            EventKind::AssetFile(app::ResourceEvent::Modified(app::ModifiedKind::Data))
        );
    }

    pub fn test_analysis_files_simple(
        watcher: &FsWatcher,
        project: &Project<LocalProject, ContainerTree>,
    ) {
        let analysis_path = project.project.analysis_root_path().unwrap();

        let events = watcher
            .process_event_fs_to_apps(&fs_event::Event::new(
                fs_event::File::Created(analysis_path.join(Fireworks::SCRIPT_NOISE_STATS_PATH)),
                Instant::now(),
            ))
            .unwrap();

        assert_eq!(events.len(), 1);
        assert_matches!(
            *events[0].kind(),
            EventKind::AnalysisFile(app::ResourceEvent::Created)
        );

        let events = watcher
            .process_event_fs_to_apps(&fs_event::Event::new(
                fs_event::File::Removed(analysis_path.join("test.py")),
                Instant::now(),
            ))
            .unwrap();

        assert_eq!(events.len(), 1);
        assert_matches!(
            *events[0].kind(),
            EventKind::AnalysisFile(app::ResourceEvent::Removed)
        );

        let events = watcher
            .process_event_fs_to_apps(&fs_event::Event::new(
                fs_event::File::Moved {
                    from: analysis_path
                        .join("test")
                        .join(Fireworks::SCRIPT_NOISE_STATS_PATH),
                    to: analysis_path.join(Fireworks::SCRIPT_NOISE_STATS_PATH),
                },
                Instant::now(),
            ))
            .unwrap();

        assert_eq!(events.len(), 1);
        assert_matches!(
            *events[0].kind(),
            EventKind::AnalysisFile(app::ResourceEvent::Moved)
        );

        let events = watcher
            .process_event_fs_to_apps(&fs_event::Event::new(
                fs_event::File::Renamed {
                    from: analysis_path.join("test.py"),
                    to: analysis_path.join(Fireworks::SCRIPT_NOISE_STATS_PATH),
                },
                Instant::now(),
            ))
            .unwrap();

        assert_eq!(events.len(), 1);
        assert_matches!(
            *events[0].kind(),
            EventKind::AnalysisFile(app::ResourceEvent::Renamed)
        );

        let events = watcher
            .process_event_fs_to_apps(&fs_event::Event::new(
                fs_event::File::DataModified(
                    analysis_path.join(Fireworks::SCRIPT_NOISE_STATS_PATH),
                ),
                Instant::now(),
            ))
            .unwrap();

        assert_eq!(events.len(), 1);
        assert_matches!(
            *events[0].kind(),
            EventKind::AnalysisFile(app::ResourceEvent::Modified(app::ModifiedKind::Data))
        );
    }

    pub fn test_files_simple(watcher: &FsWatcher, project: &Project<LocalProject, ContainerTree>) {
        let root = project.graph.get(project.graph.root()).unwrap();
        let children = project.graph.children(&root.rid).unwrap();
        let recipe_1 = project.graph.get(&children[0]).unwrap();
        let analysis_path = project.project.analysis_root_path().unwrap();

        // -- created
        let events = watcher
            .process_event_fs_to_apps(&fs_event::Event::new(
                fs_event::File::Created(project.project.base_path().join("test.txt")),
                Instant::now(),
            ))
            .unwrap();

        assert_eq!(events.len(), 1);
        assert_matches!(
            *events[0].kind(),
            EventKind::File(app::ResourceEvent::Created)
        );

        let events = watcher
            .process_event_fs_to_apps(&fs_event::Event::new(
                fs_event::File::Created(
                    local_common::app_dir_of(project.project.base_path()).join("test.txt"),
                ),
                Instant::now(),
            ))
            .unwrap();

        assert_eq!(events.len(), 1);
        assert_matches!(
            *events[0].kind(),
            EventKind::File(app::ResourceEvent::Created)
        );

        let events = watcher
            .process_event_fs_to_apps(&fs_event::Event::new(
                fs_event::File::Created(analysis_path.join("test.txt")),
                Instant::now(),
            ))
            .unwrap();

        assert_eq!(events.len(), 1);
        assert_matches!(
            *events[0].kind(),
            EventKind::File(app::ResourceEvent::Created)
        );

        let events = watcher
            .process_event_fs_to_apps(&fs_event::Event::new(
                fs_event::File::Created(
                    local_common::app_dir_of(recipe_1.base_path()).join("test.txt"),
                ),
                Instant::now(),
            ))
            .unwrap();

        assert_eq!(events.len(), 1);
        assert_matches!(
            *events[0].kind(),
            EventKind::File(app::ResourceEvent::Created)
        );
        // -- created end

        // -- removed
        let events = watcher
            .process_event_fs_to_apps(&fs_event::Event::new(
                fs_event::File::Removed(project.project.base_path().join("test.txt")),
                Instant::now(),
            ))
            .unwrap();

        assert_eq!(events.len(), 1);
        assert_matches!(
            *events[0].kind(),
            EventKind::File(app::ResourceEvent::Removed)
        );

        let events = watcher
            .process_event_fs_to_apps(&fs_event::Event::new(
                fs_event::File::Removed(
                    local_common::app_dir_of(project.project.base_path()).join("test.txt"),
                ),
                Instant::now(),
            ))
            .unwrap();

        assert_eq!(events.len(), 1);
        assert_matches!(
            *events[0].kind(),
            EventKind::File(app::ResourceEvent::Removed)
        );

        let events = watcher
            .process_event_fs_to_apps(&fs_event::Event::new(
                fs_event::File::Removed(analysis_path.join("test.txt")),
                Instant::now(),
            ))
            .unwrap();

        assert_eq!(events.len(), 1);
        assert_matches!(
            *events[0].kind(),
            EventKind::File(app::ResourceEvent::Removed)
        );

        let events = watcher
            .process_event_fs_to_apps(&fs_event::Event::new(
                fs_event::File::Removed(
                    local_common::app_dir_of(recipe_1.base_path()).join("test.txt"),
                ),
                Instant::now(),
            ))
            .unwrap();

        assert_eq!(events.len(), 1);
        assert_matches!(
            *events[0].kind(),
            EventKind::File(app::ResourceEvent::Removed)
        );
        // -- removed end

        // -- moved
        let events = watcher
            .process_event_fs_to_apps(&fs_event::Event::new(
                fs_event::File::Moved {
                    from: project.project.base_path().join("test").join("test.txt"),
                    to: project.project.base_path().join("test.txt"),
                },
                Instant::now(),
            ))
            .unwrap();

        assert_eq!(events.len(), 1);
        assert_matches!(
            *events[0].kind(),
            EventKind::File(app::ResourceEvent::Moved)
        );

        let events = watcher
            .process_event_fs_to_apps(&fs_event::Event::new(
                fs_event::File::Moved {
                    from: project.project.base_path().join("test").join("test.txt"),
                    to: local_common::app_dir_of(project.project.base_path()).join("test.txt"),
                },
                Instant::now(),
            ))
            .unwrap();

        assert_eq!(events.len(), 1);
        assert_matches!(
            *events[0].kind(),
            EventKind::File(app::ResourceEvent::Moved)
        );

        let events = watcher
            .process_event_fs_to_apps(&fs_event::Event::new(
                fs_event::File::Moved {
                    from: analysis_path.join("test").join("test.txt"),
                    to: analysis_path.join("test.txt"),
                },
                Instant::now(),
            ))
            .unwrap();

        assert_eq!(events.len(), 1);
        assert_matches!(
            *events[0].kind(),
            EventKind::File(app::ResourceEvent::Moved)
        );

        let events = watcher
            .process_event_fs_to_apps(&fs_event::Event::new(
                fs_event::File::Moved {
                    from: local_common::app_dir_of(recipe_1.base_path())
                        .join("test")
                        .join("test.txt"),
                    to: local_common::app_dir_of(recipe_1.base_path()).join("test.txt"),
                },
                Instant::now(),
            ))
            .unwrap();

        assert_eq!(events.len(), 1);
        assert_matches!(
            *events[0].kind(),
            EventKind::File(app::ResourceEvent::Moved)
        );
        // -- moved end

        // -- renamed
        let events = watcher
            .process_event_fs_to_apps(&fs_event::Event::new(
                fs_event::File::Renamed {
                    from: project.project.base_path().join("test-1.txt"),
                    to: project.project.base_path().join("test-2.txt"),
                },
                Instant::now(),
            ))
            .unwrap();

        assert_eq!(events.len(), 1);
        assert_matches!(
            *events[0].kind(),
            EventKind::File(app::ResourceEvent::Renamed)
        );

        let events = watcher
            .process_event_fs_to_apps(&fs_event::Event::new(
                fs_event::File::Renamed {
                    from: project.project.base_path().join("test-1.txt"),
                    to: local_common::app_dir_of(project.project.base_path()).join("test-2.txt"),
                },
                Instant::now(),
            ))
            .unwrap();

        assert_eq!(events.len(), 1);
        assert_matches!(
            *events[0].kind(),
            EventKind::File(app::ResourceEvent::Renamed)
        );

        let events = watcher
            .process_event_fs_to_apps(&fs_event::Event::new(
                fs_event::File::Renamed {
                    from: analysis_path.join("test-1.txt"),
                    to: analysis_path.join("test-2.txt"),
                },
                Instant::now(),
            ))
            .unwrap();

        assert_eq!(events.len(), 1);
        assert_matches!(
            *events[0].kind(),
            EventKind::File(app::ResourceEvent::Renamed)
        );

        let events = watcher
            .process_event_fs_to_apps(&fs_event::Event::new(
                fs_event::File::Renamed {
                    from: local_common::app_dir_of(recipe_1.base_path()).join("test-1.txt"),
                    to: local_common::app_dir_of(recipe_1.base_path()).join("test-2.txt"),
                },
                Instant::now(),
            ))
            .unwrap();

        assert_eq!(events.len(), 1);
        assert_matches!(
            *events[0].kind(),
            EventKind::File(app::ResourceEvent::Renamed)
        );
        // -- renamed end
    }

    pub fn test_folders_simple(
        watcher: &FsWatcher,
        project: &Project<LocalProject, ContainerTree>,
    ) {
        let project_root = project.project.base_path();
        let events = watcher
            .process_event_fs_to_apps(&fs_event::Event::new(
                fs_event::Folder::Created(project_root.join("test")),
                Instant::now(),
            ))
            .unwrap();

        assert_eq!(events.len(), 1);
        assert_matches!(
            *events[0].kind(),
            app::EventKind::Folder(app::ResourceEvent::Created)
        );

        let events = watcher
            .process_event_fs_to_apps(&fs_event::Event::new(
                fs_event::Folder::Removed(project_root.join("test")),
                Instant::now(),
            ))
            .unwrap();

        assert_eq!(events.len(), 1);
        assert_matches!(
            *events[0].kind(),
            app::EventKind::Folder(app::ResourceEvent::Removed)
        );

        let events = watcher
            .process_event_fs_to_apps(&fs_event::Event::new(
                fs_event::Folder::Renamed {
                    from: project_root.join("test-from"),
                    to: project_root.join("test-to"),
                },
                Instant::now(),
            ))
            .unwrap();

        assert_eq!(events.len(), 0);

        let events = watcher
            .process_event_fs_to_apps(&fs_event::Event::new(
                fs_event::Folder::Moved {
                    from: project_root.join("test-from").join("test"),
                    to: project_root.join("test-to").join("test"),
                },
                Instant::now(),
            ))
            .unwrap();

        assert_eq!(events.len(), 0);
    }
}

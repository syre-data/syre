use super::*;

#[tokio::test]
async fn file_system_event_processor_group_rename_events_should_work() {
    let dir = tempfile::TempDir::new().unwrap();
    let dir_sub = tempfile::TempDir::new_in(dir.path()).unwrap();
    let f_from = tempfile::NamedTempFile::new_in(dir_sub.path()).unwrap();
    let f_to = tempfile::NamedTempFile::new_in(dir_sub.path()).unwrap();
    let f_alone = tempfile::NamedTempFile::new_in(dir.path()).unwrap();
    let d_from = tempfile::TempDir::new_in(dir_sub.path()).unwrap();
    let d_to = tempfile::TempDir::new_in(dir_sub.path()).unwrap();
    let d_alone = tempfile::TempDir::new_in(dir.path()).unwrap();

    let e_f_from = notify::Event::new(EventKind::Modify(ModifyKind::Name(RenameMode::From)))
        .add_path(f_from.path().to_path_buf());

    let e_f_to = notify::Event::new(EventKind::Modify(ModifyKind::Name(RenameMode::To)))
        .add_path(f_to.path().to_path_buf());

    let e_f_alone = notify::Event::new(EventKind::Modify(ModifyKind::Name(RenameMode::To)))
        .add_path(f_alone.path().to_path_buf());

    let e_d_from = notify::Event::new(EventKind::Modify(ModifyKind::Name(RenameMode::From)))
        .add_path(d_from.path().to_path_buf());

    let e_d_to = notify::Event::new(EventKind::Modify(ModifyKind::Name(RenameMode::To)))
        .add_path(d_to.path().to_path_buf());

    let e_d_alone = notify::Event::new(EventKind::Modify(ModifyKind::Name(RenameMode::To)))
        .add_path(d_alone.path().to_path_buf());

    let events = vec![e_f_from, e_f_to, e_f_alone, e_d_from, e_d_to, e_d_alone];
    let (_, command_rx) = tokio_mpsc::unbounded_channel();
    let (event_tx, _) = mpsc::channel();
    let mut watcher = FsWatcher::new(command_rx, event_tx);
    let (converted, remaining) = watcher
        .group_events(events.into_iter().map(|e| e.into()).collect())
        .await;

    dbg!(&converted);
    dbg!(&remaining);
    assert_eq!(converted.len(), 2);
    assert_eq!(remaining.len(), 2);
}

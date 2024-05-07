use syre_fs_watcher_simulator as simulator;

#[test]
fn main() {
    let base_dir = tempfile::tempdir().unwrap();
    let user_manifest =
        tempfile::NamedTempFile::with_prefix_in("user_manifest", base_dir.path()).unwrap();

    let project_manifest =
        tempfile::NamedTempFile::with_prefix_in("project_manifest", base_dir.path()).unwrap();

    let mut options = simulator::options::Builder::new(base_dir.path().to_path_buf());
    options.set_user_manifest(user_manifest.path().to_path_buf());
    options.set_project_manifest(project_manifest.path().to_path_buf());
    options.set_max_ticks(10);

    let options = options.build();
    let mut sim = simulator::Simulator::new(options);
    sim.run();
}

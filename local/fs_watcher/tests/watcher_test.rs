use syre_fs_watcher_simulator as simulator;

#[test]
fn main() {
    // TODO: Nondeterministic dir paths.
    let config_dir = tempfile::TempDir::new().unwrap();
    let base_dir = tempfile::TempDir::new().unwrap();
    let user_manifest = tempfile::NamedTempFile::new_in(config_dir.path()).unwrap();
    let project_manifest = tempfile::NamedTempFile::new_in(config_dir.path()).unwrap();

    let mut options = simulator::Options::new(base_dir.path().to_path_buf());
    options.set_user_manifest(user_manifest.path());
    options.set_project_manifest(project_manifest.path());
    let mut simulator = options.build();
    simulator.run();
}

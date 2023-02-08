use settings_manager_change_path::settings_path;
fn main() {
    let a = BasicSettings::new();
    let p = {
        a.path();
        Ok(PathBuf::from("/tmp/my_test_file.json"))
    };
}

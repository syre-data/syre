use settings_manager_change_path::settings_path;

#[settings_path("a", "/tmp/my_test_file.json")]
fn main() {
    let a = BasicSettings::new();
    let p = a.path();
}

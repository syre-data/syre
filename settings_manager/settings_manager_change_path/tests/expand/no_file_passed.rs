use settings_manager_change_path::settings_path;

#[settings_path("a")]
fn main() {
    let a = BasicSettings::new();
    let p = a.path();
}

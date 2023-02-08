use super::*;

#[test]
fn user_preferences_new_should_work() {
    let _prefs = UserPreferences::new();
}

#[test]
fn project_user_preferences_new_should_work() {
    let prefs = ProjectUserPreferences::new();

    assert_eq!(
        AssetFileAction::Move,
        prefs.asset_file_action,
        "asset_file_action should default to move"
    );

    assert_eq!(
        true, prefs.rename_folder_on_name_change,
        "rename_folder_on_name_change should default to true"
    );

    assert_eq!(
        false, prefs.delete_on_exclude,
        "delete_on_exclude should default to false"
    );

    assert_eq!(
        true, prefs.format_metdata_objects,
        "format_metdata_objects should default to true"
    );

    assert_eq!(
        false, prefs.show_inherited_metadata,
        "show_inherited_metadata should defualt to false"
    );
}

#[test]
fn analysis_user_preferences_new_should_work() {
    let _prefs = AnalysisUserPreferences::new();
}

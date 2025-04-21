pub const APPLICATION_JSON: &'static str = "application/json";
pub const PATH_SEP_WINDOW: &'static str = "\\";
pub const PATH_SEP_NIX: &'static str = "/";

/// File system resource size in bytes at which to notify user
/// because file system transfer action may take significant time.
pub const FS_RESOURCE_ACTION_NOTIFY_THRESHOLD: u64 = 5_000_000;

/// Drag-drop event debounce in ms.
pub const THROTTLE_DRAG_EVENT: f64 = 50.0;

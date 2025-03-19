pub(super) mod db;
pub(super) mod graph;
pub(super) mod no_graph;
pub(self) mod editors;

/// Drag-drop event debounce in ms.
pub(self) const THROTTLE_DRAG_EVENT: f64 = 50.0;
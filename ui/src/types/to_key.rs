use yew::virtual_dom::Key;

/// Functionality for an object to provide a key for iteration.
pub trait ToKey {
    fn key(&self) -> Key;
}

//! A `TreeView` item.
use yew::virtual_dom::Key;
use yew::Html;

pub trait TreeViewItem: PartialEq {
    fn id(&self) -> Key;
    fn html(&self) -> Html;
    // fn iter_children(&self) -> Box<dyn Iterator<Item = Box<dyn TreeViewItem>>>;
}

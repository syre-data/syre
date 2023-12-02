//! Tree view for a `Container` tree.
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct TreeViewProps<I: PartialEq> {
    #[prop_or_default]
    class: Classes,

    root: I,

    #[prop_or(false)]
    expanded: bool,
}

#[function_component(TreeView)]
pub fn tree_view<I>(props: &TreeViewProps<I>) -> Html
where
    I: PartialEq,
{
    let expanded = use_state(|| props.expanded);
    let class = classes!("thot-ui-tree-view", props.class);

    let toggle_expanded = {
        let expanded = expanded.clone();

        Callback::from(move |_: MouseEvent| {
            expanded.set(!*expanded);
        })
    };

    html! {
        <div {class}>
            <button class="expand-controller"
                onclick={toggle_expanded}>

                if *expanded {
                    { '\u{2304}' }
                } else {
                    { '\u{2303}' }
                }
            </button>
            <span class="item">
                { props.root.html() }
            </span>
            if *expanded {
                <ol>
                    { props.root.iter_children().map(|child| html! {
                        <li key={child.id()}>
                            <TreeView<I> root={child} />
                        </li>
                    }).collect::<Html>() }
                </ol>
            }
        </div>
    }
}

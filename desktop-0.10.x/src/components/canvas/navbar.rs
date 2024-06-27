//! Navbar.
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct TreeViewContainerItemProps {
    container: CoreContainer,
}

#[function_component(TreeViewContainerItem)]
pub fn tree_view_container_item(props: &TreeViewContainerItemProps) -> Html {
    html! {
        <span class="item-name">{
            if let Some(name) = props.container.name.as_ref() {
                name
            } else {
                "(no name)"
            }
        }</span>
    }
}

#[function_component(ContainerTreeView)]
pub fn container_tree_view(props: &ContainerTreeViewProps) -> Html {
    let expanded = use_state(|| props.expanded);
    let class = classes!("syre-ui-tree-view", props.class);

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
            <TreeViewContainerItem container={root} />
            if *expanded {
                <ol>
                    { props.root.children.iter().map(|child| html! {
                        <li key={child.rid}>
                            <ContainerTreeView root={child} />
                        </li>
                    }).collect::<Html>() }
                </ol>
            }
        </div>
    }
}

#[function_component(NavBar)]
pub fn navbar() -> Html {
    let canvas_state =
        use_context::<CanvasStateReducer>().expect("`CanvasStateReducer` context not found");

    let project = use_project(&canvas_state.project);

    html! {
        <ContainerTreeView root={project.root} />
    }
}

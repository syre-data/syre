//! Tags.
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct TagsProps {
    /// Tags
    #[prop_or(Vec::new())]
    pub value: Vec<String>,
}

#[function_component(Tags)]
pub fn tags_editor(props: &TagsProps) -> Html {
    html! {
        <div class={classes!("thot-ui-tags")}>
            if props.value.is_empty() {
                { "(no tags)" }
            } else {
                { props.value.iter().map(|tag| html!{ <span class={classes!("tag")}>{ tag }</span> }).collect::<Html>() }
            }
        </div>
    }
}

#[cfg(test)]
#[path = "./tags_test.rs"]
mod tags_test;

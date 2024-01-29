use crate::components::card::{Card, CardProps, CardUi};
use syre_core::project::Project;
use yew::prelude::*;

impl CardUi for Project {
    fn title(&self) -> &str {
        &self.name
    }

    fn body(&self) -> Html {
        match &self.description {
            None => html! {},
            Some(description) => html! { { description } },
        }
    }
}

pub type ProjectCardProps = CardProps<Project>;

#[function_component(ProjectCard)]
pub fn project_card(props: &ProjectCardProps) -> Html {
    html! {
        <Card<Project> item={props.item.clone()} onclick={&props.onclick} />
    }
}

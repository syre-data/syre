use crate::components::card::{Card, CardProps, CardUi};
use thot_core::project::Project;
use yew::prelude::*;

impl CardUi for Project {
    fn title(&self) -> &str {
        &self.name
    }

    fn body(&self) -> Html {
        html! { {&self.name} }
    }
}

pub type ProjectCardProps = CardProps<Project>;

#[function_component(ProjectCard)]
pub fn project_card(props: &ProjectCardProps) -> Html {
    html! {
        <Card<Project> item={props.item.clone()} onclick={&props.onclick} />
    }
}

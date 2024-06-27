//! Excel template builder review.
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct ReviewProps {
    pub onaccept: Callback<()>,
}

#[function_component(TemplateReview)]
pub fn template_builder(props: &ReviewProps) -> Html {
    let onaccept = use_callback(props.onaccept.clone(), move |_, onaccept| {
        onaccept.emit(());
    });

    html! {
        <button class={"btn-primary"}
            onclick={onaccept}>

            { "Create template" }
        </button>
    }
}

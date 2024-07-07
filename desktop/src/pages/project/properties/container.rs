use crate::pages::project::state::Container;
use leptos::*;

#[component]
pub fn Editor(container: Container) -> impl IntoView {
    view! {
        <div>
            <h1>
                {container
                    .properties()
                    .read_only()
                    .with(|properties| properties.as_ref().unwrap().name().get())}
            </h1>
            <form></form>

        </div>
    }
}

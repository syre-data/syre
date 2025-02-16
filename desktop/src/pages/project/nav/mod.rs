use leptos::prelude::*;

mod layers;
mod search;

#[component]
pub fn NavBar() -> impl IntoView {
    view! {
        <div>
            <search::Search />
            <layers::LayersNav />
        </div>
    }
}

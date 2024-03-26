use super::super::CanvasStateReducer;
use super::layers_bar::LayersBar;
use super::search_bar::SearchBar;
use yew::prelude::*;

#[function_component(ResourcesBar)]
pub fn resources_bar() -> Html {
    let canvas_state = use_context::<CanvasStateReducer>().unwrap();

    html! {
        { match canvas_state.resources_bar_widget {
            ResourcesBarWidget::Layers=> html! { <LayersBar /> },
            ResourcesBarWidget::Search=> html! { <SearchBar /> },
        }}
    }
}

#[derive(PartialEq, Clone, Debug)]
pub enum ResourcesBarWidget {
    Layers,
    Search,
}

impl Default for ResourcesBarWidget {
    fn default() -> Self {
        Self::Layers
    }
}

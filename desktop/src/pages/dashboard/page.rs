//! Home dashboard.
use super::dashboard::Dashboard as DashboardComponent;
use thot_ui::widgets::suspense::Loading;
use yew::prelude::*;

#[function_component(Dashboard)]
pub fn dashboard() -> Html {
    let fallback = html! { <Loading text={"Loading resources"} />  };

    html! {
        <Suspense {fallback}>
            <DashboardComponent />
        </Suspense>
    }
}

#[cfg(test)]
#[path = "./page_test.rs"]
mod page_test;

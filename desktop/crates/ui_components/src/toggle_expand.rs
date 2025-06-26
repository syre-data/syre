use leptos::{ev::MouseEvent, prelude::*};
use leptos_icons::Icon;
use syre_desktop_ui_lib as ui_lib;

#[component]
pub fn ToggleExpand(expanded: RwSignal<bool>) -> impl IntoView {
    let toggle = move |e: MouseEvent| {
        if e.button() != ui_lib::types::MouseButton::Primary {
            return;
        }

        e.stop_propagation();
        expanded.set(!expanded());
    };

    // TODO: Center icon vertically.
    view! {
        <button on:mousedown=toggle type="button">
            <span class=("rotate-90", expanded) class="inline-block transition">
                <Icon icon=ui_lib::icon::ChevronRight />
            </span>
        </button>
    }
}


#[component]
pub fn ToggleExpandSymbol(
     #[prop(into)]
     symbol_id: String, 
     expanded: RwSignal<bool>
) -> impl IntoView {
    let toggle = move |e: MouseEvent| {
        if e.button() != ui_lib::types::MouseButton::Primary {
            return;
        }

        e.stop_propagation();
        expanded.set(!expanded());
    };

    // TODO: Center icon vertically.
    view! {
        <button on:mousedown=toggle type="button">
                <span class=("rotate-90", expanded) class="inline-block transition">
                    <svg width="1em" height="1em">
                        <use href=format!("#{symbol_id}") />
                    </svg>
                </span>
        </button>
    }
}
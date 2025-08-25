use leptos::{ev::MouseEvent, prelude::*};
use leptos_icons::Icon;
use syre_desktop_ui_lib as ui_lib;

mod layers;
mod search;

#[derive(PartialEq, Copy, Clone)]
enum Widget {
    Layers,
    Search,
}

#[component]
pub fn NavBar() -> impl IntoView {
    let (widget, set_widget) = signal(Widget::Layers);

    let mousedown = move |e: MouseEvent, view: Widget| {
        if e.button() != ui_lib::types::MouseButton::Primary {
            return;
        }

        if widget.with_untracked(|widget| view != *widget) {
            set_widget(view);
        }
    };

    view! {
        <div class="flex flex-col h-full">
            <div class="flex gap-1 px-1 pt-px pb-2">
                <button
                    on:mousedown=move |e| mousedown(e, Widget::Layers)
                    class=(
                        ["bg-secondary-100", "dark:bg-secondary-900"],
                        move || widget.with(|widget| matches!(widget, Widget::Layers)),
                    )
                    class="p-px hover:bg-secondary-100 dark:hover:bg-secondary-800 rounded-xs \
                    cursor-pointer"
                    title="Layers view"
                >
                    <Icon icon=icondata::TbListTree />
                </button>
                <button
                    on:mousedown=move |e| mousedown(e, Widget::Search)
                    class=(
                        ["bg-secondary-100", "dark:bg-secondary-900"],
                        move || widget.with(|widget| matches!(widget, Widget::Search)),
                    )
                    class="p-px hover:bg-secondary-100 dark:hover:bg-secondary-800 rounded-xs \
                    cursor-pointer"
                    title="Search"
                >
                    <Icon icon=ui_lib::icon::Search />
                </button>
            </div>
            <div class="grow overflow-auto scrollbar-thin dark:scrollbar-track-secondary-800">
                <layers::LayersNav
                    {..}
                    class=(
                        "hidden",
                        move || widget.with(|widget| !matches!(widget, Widget::Layers)),
                    )
                />
                <search::Search
                    {..}
                    class=(
                        "hidden",
                        move || widget.with(|widget| !matches!(widget, Widget::Search)),
                    )
                />
            </div>
        </div>
    }
}

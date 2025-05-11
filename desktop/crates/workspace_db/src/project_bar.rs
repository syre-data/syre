use crate::state;
use leptos::{ev::MouseEvent, prelude::*};
use leptos_icons::Icon;
use syre_desktop_ui_lib as ui_lib;
use wasm_bindgen::{JsCast, closure::Closure};

#[component]
pub fn ProjectBar() -> impl IntoView {
    let display_state = expect_context::<state::display::State>();

    let toggle_filter_bar = {
        let filter_bar = display_state.filter_bar();
        move |e: MouseEvent| {
            if e.button() != ui_lib::types::MouseButton::Primary {
                return;
            }

            filter_bar.set(!filter_bar.get());
        }
    };

    view! {
        <div class="flex px-2 py-1">
            <div class="w-1/3 flex gap-2">
                <ColumnSelector />
                <button
                    class="btn-secondary p-1 rounded-xs cursor-pointer"
                    class=("text-primary-700", display_state.filter_bar())
                    on:click=toggle_filter_bar
                >
                    <Icon icon=ui_lib::icon::Filter />
                </button>
            </div>
            <div class="w-1/3 text-center">
                <ProjectInfo />
            </div>
            <div class="w-1/3 text-right">
                <Controls />
            </div>
        </div>
    }
}

#[component]
fn ColumnSelector() -> impl IntoView {
    const MENU_ID: &str = "db-workspace-column-selection-menu";

    let state = expect_context::<state::data::State>();
    let display_state = expect_context::<state::display::State>();
    let (active, set_active) = signal_local::<Option<Closure<dyn FnMut(MouseEvent)>>>(None);

    let columns = display_state.columns();
    let activate = move |e: MouseEvent| {
        if e.button() != ui_lib::types::MouseButton::Primary {
            return;
        }
        if active.read().is_some() {
            return;
        }
        e.stop_propagation();

        let cb: Closure<dyn FnMut(MouseEvent)> = Closure::new(move |e: MouseEvent| {
            if e.button() != ui_lib::types::MouseButton::Primary {
                return;
            }

            let target = e.target().unwrap();
            if let Some(target) = target.dyn_ref::<web_sys::HtmlElement>() {
                if target.closest(&format!("#{MENU_ID}")).unwrap().is_some() {
                    return;
                }
            } else if let Some(target) = target.dyn_ref::<web_sys::SvgElement>() {
                if target.closest(&format!("#{MENU_ID}")).unwrap().is_some() {
                    return;
                }
            };

            let window = web_sys::window().unwrap();
            active.with_untracked(|active| {
                let cb = active.as_ref().unwrap();
                window
                    .remove_event_listener_with_callback("mousedown", cb.as_ref().unchecked_ref())
                    .unwrap();
            });

            set_active.write().take();
        });

        let window = web_sys::window().unwrap();
        window
            .add_event_listener_with_callback("mousedown", cb.as_ref().unchecked_ref())
            .unwrap();

        let _ = set_active.write().insert(cb);
    };

    let set_visibility_for_all = |visibile: bool| {
        let columns = columns.clone();
        move |e: MouseEvent| {
            if e.button() != ui_lib::types::MouseButton::Primary {
                return;
            }

            columns.set_visibility_for_all(visibile);
        }
    };

    const CLASS_FORM_DIV: &str = "px-2 w-full";
    const CLASS_CHECKBOX: &str = "w-4 h-4 rounded-sm";
    const CLASS_LABEL: &str = "pl-2";
    view! {
        <div class="relative z-10">
            <div
                on:mousedown=activate
                class=("rounded-b-none", move || active.read().is_some())
                class="cursor-pointer inline-flex w-40 px-2 rounded-sm border \
                border-secondary-600 dark:border-secondary-200"
            >
                <span class="grow">"Columns"</span>
                <span class="pl-2 inline-flex items-center">
                    <Icon icon=icondata::FaChevronDownSolid />
                </span>
            </div>
            <div
                id=MENU_ID
                class:hidden=move || active.read().is_none()
                class="absolute w-40 rounded-b bg-white dark:bg-secondary-900 border \
                border-t-0 border-secondary-600 dark:border-secondary-200"
            >
                <form on:submit=move |e| e.prevent_default()>
                    <div class="scrollbar-thin overflow-y-auto max-h-24">
                        <div class=CLASS_FORM_DIV>
                            <label class="cursor-pointer">
                                <input
                                    type="checkbox"
                                    name="path"
                                    on:input={
                                        let visible = columns.path().visible();
                                        move |_| visible.set(!visible())
                                    }
                                    prop:checked=columns.path().visible().read_only()
                                    class=CLASS_CHECKBOX
                                />
                                <span class=CLASS_LABEL>"Path"</span>
                            </label>
                        </div>

                        <div class=CLASS_FORM_DIV>
                            <label class="cursor-pointer">
                                <input
                                    type="checkbox"
                                    name="file"
                                    on:input={
                                        let visible = columns.file().visible();
                                        move |_| visible.set(!visible())
                                    }
                                    prop:checked=columns.file().visible().read_only()
                                    class=CLASS_CHECKBOX
                                />
                                <span class=CLASS_LABEL>"File"</span>
                            </label>
                        </div>

                        <div class=CLASS_FORM_DIV>
                            <label class="cursor-pointer">
                                <input
                                    type="checkbox"
                                    name="name"
                                    on:input={
                                        let visible = columns.name().visible();
                                        move |_| visible.set(!visible())
                                    }
                                    prop:checked=columns.name().visible().read_only()
                                    class=CLASS_CHECKBOX
                                />
                                <span class=CLASS_LABEL>"Name"</span>
                            </label>
                        </div>

                        <div class=CLASS_FORM_DIV>
                            <label class="cursor-pointer">
                                <input
                                    type="checkbox"
                                    name="kind"
                                    on:input={
                                        let visible = columns.kind().visible();
                                        move |_| visible.set(!visible())
                                    }
                                    prop:checked=columns.kind().visible().read_only()

                                    class=CLASS_CHECKBOX
                                />
                                <span class=CLASS_LABEL>"Kind"</span>
                            </label>
                        </div>

                        <div class=CLASS_FORM_DIV>
                            <label class="cursor-pointer">
                                <input
                                    type="checkbox"
                                    name="description"
                                    on:input={
                                        let visible = columns.description().visible();
                                        move |_| visible.set(!visible())
                                    }
                                    prop:checked=columns.description().visible().read_only()
                                    class=CLASS_CHECKBOX
                                />
                                <span class=CLASS_LABEL>"Description"</span>
                            </label>
                        </div>

                        <div class=CLASS_FORM_DIV>
                            <label class="cursor-pointer">
                                <input
                                    type="checkbox"
                                    name="tags"
                                    on:input={
                                        let visible = columns.tags().visible();
                                        move |_| visible.set(!visible())
                                    }
                                    prop:checked=columns.tags().visible().read_only()
                                    class=CLASS_CHECKBOX
                                />
                                <span class=CLASS_LABEL>"Tags"</span>
                            </label>
                        </div>
                        <hr class="border-secondary-900 dark:border-secondary-200" />
                        <div>
                            <For
                                each=state.metadata_keys()
                                key=|key| key.clone()
                                let:key
                                clone:display_state
                            >
                                <div class=CLASS_FORM_DIV>
                                    <label class="cursor-pointer">
                                        {
                                            let column = display_state
                                                .columns()
                                                .metadata(&key)
                                                .unwrap();
                                            view! {
                                                <input
                                                    type="checkbox"
                                                    name="tags"
                                                    on:input={
                                                        let visible = column.visible();
                                                        move |_| visible.set(!visible())
                                                    }
                                                    prop:checked=column.visible().read_only()
                                                    class=CLASS_CHECKBOX
                                                />
                                            }
                                        } <span class=CLASS_LABEL>{key}</span>
                                    </label>
                                </div>
                            </For>
                        </div>
                    </div>
                    <hr class="border-secondary-900 dark:border-secondary-200" />
                    <div>
                        <div class="px-2 text-center dark:border-secondary-200">
                            <button
                                on:mousedown=set_visibility_for_all(true)
                                class="w-full h-full cursor-pointer"
                            >
                                "All"
                            </button>
                        </div>
                        <div class="px-2 text-center dark:border-secondary-200">
                            <button
                                on:mousedown=set_visibility_for_all(false)
                                class="w-full h-full cursor-pointer"
                            >
                                "None"
                            </button>
                        </div>
                    </div>
                </form>
            </div>
        </div>
    }
}

#[component]
fn ProjectInfo() -> impl IntoView {
    let project = expect_context::<ui_lib::state::Project>();
    view! { <div class="grow text-center font-primary">{project.properties().name()}</div> }
}

#[component]
fn Controls() -> impl IntoView {
    const COMMAND_BUTTON_CLASS: &str = "btn-secondary p-1 rounded-xs cursor-pointer";

    let data_view = expect_context::<RwSignal<ui_lib::types::DataView>>();

    let toggle_data_view = move |e: MouseEvent| {
        if e.button() != ui_lib::types::MouseButton::Primary {
            return;
        }

        data_view.set(ui_lib::types::DataView::Graph)
    };

    let refresh = move |e: MouseEvent| {
        if e.button() != ui_lib::types::MouseButton::Primary {
            return;
        }

        let window = web_sys::window().unwrap();
        window.location().reload().unwrap();
    };

    view! {
        <ol class="flex gap-1 justify-end">
            <li>
                <button
                    on:mousedown=toggle_data_view
                    type="button"
                    class=COMMAND_BUTTON_CLASS
                    title="Toggle data view"
                >
                    <Icon icon=icondata::ImTree />
                </button>

            </li>
            <li>
                <button
                    on:mousedown=refresh
                    type="button"
                    class="btn-secondary p-1 rounded-xs cursor-pointer"
                    title="Refresh"
                >
                    <Icon icon=ui_lib::icon::Refresh />
                </button>
            </li>
        </ol>
    }
}

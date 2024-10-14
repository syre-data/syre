use crate::{app::PrefersDarkTheme, components::icon, types};
use leptos::{
    ev::{Event, MouseEvent},
    *,
};
use leptos_icons::*;
use std::path::PathBuf;

#[component]
pub fn Settings(
    /// Called when the user requests to close the page.
    #[prop(into)]
    onclose: Callback<()>,
) -> impl IntoView {
    let trigger_close = move |e: MouseEvent| {
        if e.button() == types::MouseButton::Primary {
            onclose(());
        }
    };

    view! {
        <div class="relative bg-white dark:bg-secondary-800 dark:text-white h-full w-full">
            <div>
                <button
                    on:mousedown=trigger_close
                    type="button"
                    class="absolute top-2 right-2 rounded hover:bg-secondary-100 dark:hover:bg-secondary-700"
                >
                    <Icon icon=icon::Close />
                </button>
            </div>
            <h1 class="text-lg font-primary pt-2 pb-4 px-2">"Settings"</h1>
            <div class="px-2 pb-4">
                <h2 class="text-md font-primary pb-2">"Desktop"</h2>
                <DesktopSettings />
            </div>
            <div class="px-2">
                <h2 class="text-md font-primary pb-2">"Runner"</h2>
                <RunnerSettings />
            </div>
        </div>
    }
}

#[component]
fn DesktopSettings() -> impl IntoView {
    let prefers_dark_theme = expect_context::<PrefersDarkTheme>();
    let (input_debounce, set_input_debounce) = create_signal(400);

    let toggle_theme = move |e: MouseEvent| {
        if e.button() != types::MouseButton::Primary {
            return;
        }

        prefers_dark_theme.set(!prefers_dark_theme());
    };

    let update_input_debounce = move |e: Event| {
        let value = event_target_value(&e);
        if let Ok(value) = value.parse::<usize>() {
            set_input_debounce(value);
        }
    };

    view! {
        <form on:submit=move |e| e.prevent_default()>
            <div>
                <label>
                    "Theme"
                    {move || {
                        if prefers_dark_theme() {
                            view! {
                                <button
                                    type="button"
                                    on:mousedown=toggle_theme
                                    class="rounded"
                                    title="Light theme"
                                >
                                    <Icon icon=icondata::BsSun />
                                </button>
                            }
                        } else {
                            view! {
                                <button
                                    type="button"
                                    on:mousedown=toggle_theme
                                    class="rounded"
                                    title="Dark theme"
                                >
                                    <Icon icon=icondata::BsMoon />
                                </button>
                            }
                        }
                    }}
                </label>
            </div>
            <div>
                <label>
                    "Input debounce"
                    <input
                        type="number"
                        min="250"
                        max="1000"
                        prop:value=input_debounce
                        on:input=update_input_debounce
                        class="input-simple"
                    />
                </label>
            </div>
        </form>
    }
}

#[component]
fn RunnerSettings() -> impl IntoView {
    let (python_path, set_python_path) = create_signal(PathBuf::new());
    let (r_path, set_r_path) = create_signal(PathBuf::new());

    let update_python_path = move |e: Event| {
        let value = PathBuf::from(event_target_value(&e));
        set_python_path(value);
    };

    let update_r_path = move |e: Event| {
        let value = PathBuf::from(event_target_value(&e));
        set_r_path(value);
    };

    view! {
        <form on:submit=move |e| e.prevent_default()>
            <div>
                <label>
                    <span>
                        <Icon icon=icondata::FaPythonBrands />
                    </span>
                    "Python path"
                    <input
                        type="file"
                        prop:value=move || {
                            python_path.with(|path| path.to_string_lossy().to_string())
                        }
                        on:input=update_python_path
                        class="input-simple"
                    />
                </label>
            </div>
            <div>
                <label>
                    <span>
                        <Icon icon=icondata::FaRProjectBrands />
                    </span>
                    "R path"
                    <input
                        type="file"
                        prop:value=move || r_path.with(|path| path.to_string_lossy().to_string())
                        on:input=update_r_path
                        class="input-simple"
                    />
                </label>
            </div>
        </form>
    }
}

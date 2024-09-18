use crate::types;
use leptos::{ev::MouseEvent, *};
use leptos_icons::Icon;

#[component]
pub fn DetailPopout(
    #[prop(into)] title: TextProp,
    visibility: RwSignal<bool>,
    children: Children,
    #[prop(optional, into)] onclose: Option<Callback<MouseEvent>>,
) -> impl IntoView {
    let cancel = move |e: MouseEvent| {
        if e.button() == types::MouseButton::Primary {
            visibility.set(false);
            if let Some(onclose) = onclose {
                onclose(e);
            }
        }
    };

    view! {
        <div
            class:hidden=move || !visibility()
            class="absolute -left-[105%] right-[105%] top-0 rounded border border-secondary-300 dark:border-secondary-600 bg-white dark:bg-secondary-700"
        >
            <div class="flex p-1 border-b dark:border-secondary-500">
                <span class="grow">{title}</span>
                <span>
                    <button
                        type="button"
                        on:mousedown=cancel
                        class="hover:bg-secondary-200 dark:hover:bg-secondary-600"
                    >
                        <Icon icon=icondata::AiCloseOutlined/>
                    </button>
                </span>
            </div>
            <div class="pt-1">{children()}</div>
        </div>
    }
}

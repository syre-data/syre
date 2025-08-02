use crate::pages::{
    Index,
    auth::{Login, Logout, Register},
    project::Workspace,
};
use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::{
    components::{FlatRoutes, Redirect, Route, Router},
    path,
};
use message::Messages;
use syre_desktop_ui_lib as ui_lib;

/// For Tailwind to include classes
/// they must appear as string literals in at least one place.
/// This array is used to include them when needed.
static _TAILWIND_CLASSES: &'static [&'static str] = &[
    "hidden",
    "invisible",
    "collapse",
    "dark:text-primary-100",
    "dark:text-primary-200",
    "dark:text-primary-400",
    "dark:text-primary-500",
    "dark:text-secondary-50",
    "dark:text-syre-green-200",
    "dark:text-syre-green-400",
    "dark:text-syre-red-400",
    "dark:text-syre-red-500",
    "dark:text-syre-yellow-400",
    "dark:text-syre-yellow-500",
    "dark:text-syre-yellow-600",
    "text-primary-700",
    "text-primary-800",
    "text-primary-900",
    "text-secondary-900",
    "text-syre-green-700",
    "text-syre-green-900",
    "text-syre-red-700",
    "text-syre-red-800",
    "text-syre-yellow-700",
    "text-syre-yellow-800",
    "text-syre-yellow-900",
];

#[cfg_attr(feature = "tracing", tracing::instrument(level = "trace", skip_all))]
#[component]
pub fn App() -> impl IntoView { 
    provide_meta_context();
    provide_context(ui_lib::message::Messages::new()); // TODO: Only provide after user is logged in?
    let (stored_prefers_dark, set_stored_prefers_dark, _) = leptos_use::storage::use_local_storage::<
        bool,
        codee::string::FromToStringCodec,
    >("dark_mode");
    let prefers_dark_theme =
        ui_lib::types::PrefersDarkTheme::new(stored_prefers_dark.get_untracked());
    provide_context(prefers_dark_theme);
    Effect::new(move |_| {
        set_stored_prefers_dark(prefers_dark_theme());
    });

    let class_html = move || {
        if prefers_dark_theme() { "dark" } else { "" }
    };

    view! {
        <Title formatter=|text| text text="Syre" />
        <Html attr:class=class_html />
        <Body attr:class="h-screen font-secondary overflow-hidden dark:bg-secondary-800 dark:text-white" />

        <Router>
            <FlatRoutes fallback=NotFound>
                <Route path=path!("") view=Index />
                <Route path=path!("register") view=move || view! { <Register /> } />
                <Route path=path!("login") view=Login />
                <Route path=path!("logout") view=Logout />
                <Route path=path!(":id") view=Workspace />
            </FlatRoutes>
        </Router>
        <Messages />
    }
}

#[component]
fn NotFound() -> impl IntoView {
    view! {
        <div class="text-center">
            <div class="text-lg">"Page not found"</div>
            <div>"Redirecting you home"</div>
        </div>
        <Redirect path="/" />
    }
}

mod message {
    use leptos::{either::either, ev::MouseEvent, prelude::*};
    use leptos_icons::Icon;
    use std::sync::Arc;
    use syre_desktop_ui_components::ToggleExpand;
    use syre_desktop_ui_lib as ui_lib;

    #[component]
    pub fn Messages() -> impl IntoView {
        let messages = expect_context::<ui_lib::message::Messages>();
        view! {
            <div class="absolute bottom-0 right-2 w-1/3 max-h-[75%] \
            flex flex-col gap-2 scrollbar-thin z-50">
                {move || {
                    messages
                        .get()
                        .into_iter()
                        .rev()
                        .map(|message| {
                            view! { <Message message /> }
                        })
                        .collect::<Vec<_>>()
                }}
            </div>
        }
    }

    #[component]
    fn Message(message: ui_lib::message::MessageContainer) -> impl IntoView + 'static {
        use ui_lib::message::MessageContainer;

        either!( message,
            MessageContainer::NoBody(message) => view! { <MessageNoBody message /> },
            MessageContainer::String(message) => view! { <MessageStringBody message /> },
            MessageContainer::AnyView(message) => view! { <MessageAnyBody message /> },
        )
    }

    #[component]
    fn MessageNoBody(
        message: ui_lib::message::Message<ui_lib::message::NoBody>,
    ) -> impl IntoView + 'static {
        let messages = expect_context::<ui_lib::message::Messages>();

        let close = {
            let message_id = message.id();
            move |e: MouseEvent| {
                if e.button() != ui_lib::types::MouseButton::Primary {
                    return;
                }

                messages.update(|messages| messages.retain(|msg| msg.id() != message_id));
            }
        };

        let (class_main, class_btn) = match message.kind() {
            ui_lib::message::MessageKind::Info => (
                "flex bg-primary-500 border border-primary-600 rounded-sm",
                "border-l border-l-primary-600 flex",
            ),
            ui_lib::message::MessageKind::Success => (
                "flex bg-syre-green-600 border border-syre-green-700 rounded-sm",
                "border-l border-l-green-700 flex",
            ),
            ui_lib::message::MessageKind::Warning => (
                "flex bg-syre-yellow-600 border border-syre-yellow-700 rounded-sm",
                "border-l border-l-yellow-700 flex",
            ),
            ui_lib::message::MessageKind::Error => (
                "flex bg-syre-red-500 border border-syre-red-700 rounded-sm",
                "border-l border-l-red-700 flex",
            ),
        };

        view! {
            <div class=class_main>
                <div class="grow">
                    <div class="relative flex gap-2">
                        <div class="text-lg grow px-2">{message.title().clone()}</div>
                    </div>
                </div>
                <div class=class_btn>
                    <button on:mousedown=close class="px-2 w-full h-full cursor-pointer">
                        <Icon icon=ui_lib::icon::Close />
                    </button>
                </div>
            </div>
        }
    }

    #[component]
    fn MessageStringBody(
        message: ui_lib::message::Message<ui_lib::message::Body<String>>,
    ) -> impl IntoView + 'static {
        let messages = expect_context::<ui_lib::message::Messages>();
        let show_body = RwSignal::new(false);

        let close = {
            let message_id = message.id();
            move |e: MouseEvent| {
                if e.button() != ui_lib::types::MouseButton::Primary {
                    return;
                }

                messages.update(|messages| messages.retain(|msg| msg.id() != message_id));
            }
        };

        let (class_main, class_btn) = match message.kind() {
            ui_lib::message::MessageKind::Info => (
                "flex bg-primary-500 border border-primary-600 rounded-sm",
                "border-l border-l-primary-600 flex",
            ),
            ui_lib::message::MessageKind::Success => (
                "flex bg-syre-green-600 border border-syre-green-700 rounded-sm",
                "border-l border-l-green-700 flex",
            ),
            ui_lib::message::MessageKind::Warning => (
                "flex bg-syre-yellow-600 border border-syre-yellow-700 rounded-sm",
                "border-l border-l-yellow-700 flex",
            ),
            ui_lib::message::MessageKind::Error => (
                "flex bg-syre-red-500 border border-syre-red-700 rounded-sm",
                "border-l border-l-red-700 flex",
            ),
        };

        view! {
            <div class=class_main>
                <div class="grow">
                    <div class="relative flex gap-2">
                        <div class="text-lg grow px-2">{message.title().clone()}</div>
                        <div>
                            <ToggleExpand expanded=show_body />
                        </div>
                    </div>

                    <div
                        class:hidden=move || !show_body()
                        class="pt-2 px-2 max-h-48 overflow-auto select-text scrollbar-thin break-all"
                    >
                        {(**message.body()).clone()}
                    </div>
                </div>
                <div class=class_btn>
                    <button on:mousedown=close class="px-2 w-full h-full cursor-pointer">
                        <Icon icon=ui_lib::icon::Close />
                    </button>
                </div>
            </div>
        }
    }

    #[component]
    fn MessageAnyBody<B: ui_lib::message::AsAnyView + ?Sized>(
        message: ui_lib::message::Message<ui_lib::message::Body<Arc<B>>>,
    ) -> impl IntoView + 'static {
        let messages = expect_context::<ui_lib::message::Messages>();
        let show_body = RwSignal::new(false);

        let close = {
            let message_id = message.id();
            move |e: MouseEvent| {
                if e.button() != ui_lib::types::MouseButton::Primary {
                    return;
                }

                messages.update(|messages| messages.retain(|msg| msg.id() != message_id));
            }
        };

        let (class_main, class_btn) = match message.kind() {
            ui_lib::message::MessageKind::Info => (
                "flex bg-primary-500 border border-primary-600 rounded-sm",
                "border-l border-l-primary-600 flex",
            ),
            ui_lib::message::MessageKind::Success => (
                "flex bg-syre-green-600 border border-syre-green-700 rounded-sm",
                "border-l border-l-green-700 flex",
            ),
            ui_lib::message::MessageKind::Warning => (
                "flex bg-syre-yellow-600 border border-syre-yellow-700 rounded-sm",
                "border-l border-l-yellow-700 flex",
            ),
            ui_lib::message::MessageKind::Error => (
                "flex bg-syre-red-500 border border-syre-red-700 rounded-sm",
                "border-l border-l-red-700 flex",
            ),
        };

        view! {
            <div class=class_main>
                <div class="grow">
                    <div class="relative flex gap-2">
                        <div class="text-lg grow px-2">{message.title().clone()}</div>
                        <div>
                            <ToggleExpand expanded=show_body />
                        </div>
                    </div>

                    <div
                        class:hidden=move || !show_body()
                        class="pt-2 px-2 max-h-48 overflow-auto select-text scrollbar-thin break-all"
                    >
                        {message.body().as_any_view()}
                    </div>
                </div>
                <div class=class_btn>
                    <button on:mousedown=close class="px-2 w-full h-full cursor-pointer">
                        <Icon icon=ui_lib::icon::Close />
                    </button>
                </div>
            </div>
        }
    }
}

use super::INPUT_DEBOUNCE;
use crate::{components::TruncateLeft, pages::project::state, types::MouseButton};
use description::Editor as Description;
use leptos::{ev::MouseEvent, *};
use name::Editor as Name;
use serde::Serialize;
use syre_core as core;
use syre_local as local;

#[component]
pub fn Editor() -> impl IntoView {
    let project = expect_context::<state::Project>();

    let show_delete_confirmation = move |e: MouseEvent| {
        if e.button() == MouseButton::Primary {
            todo!();
        }
    };

    view! {
        <div class="overflow-y-auto pr-2 pb-4 h-full">
            <div class="text-center pt-1 pb-2">
                <h3 class="font-primary">"Container"</h3>
            </div>
            <div>
                <form on:submit=|e| e.prevent_default()>
                    <div class="pb-1 px-1">
                        <label>
                            <span class="block">"Name"</span>
                            <Name />
                        </label>
                    </div>
                    <div class="pb-1 px-1">
                        <label>
                            <span class="block">"Description"</span>
                            <Description />
                        </label>
                    </div>
                </form>
                <div title=move || project.path().with(|path| path.to_string_lossy().to_string())>
                    <TruncateLeft>
                        {move || project.path().with(|path| path.to_string_lossy().to_string())}
                    </TruncateLeft>
                </div>
            </div>
            <div>
                <button on:mousedown=show_delete_confirmation>"Delete"</button>
            </div>
        </div>
    }
}

mod name {
    use super::{update_properties, INPUT_DEBOUNCE};
    use crate::{
        components::{form::debounced::InputText, message::Builder as Message},
        pages::project::state,
        types::Messages,
    };
    use leptos::*;

    #[component]
    pub fn Editor() -> impl IntoView {
        let project = expect_context::<state::Project>();
        let messages = expect_context::<Messages>();

        let oninput = {
            let project = project.clone();
            move |value: String| {
                let value = value.trim();
                if value.is_empty() {
                    return;
                }

                let mut properties = project.as_properties();
                properties.name = value.to_string();

                spawn_local({
                    let messages = messages.write_only();
                    async move {
                        if let Err(err) = update_properties(properties).await {
                            tracing::error!(?err);
                            let mut msg = Message::error("Could not save container.");
                            msg.body(format!("{err:?}"));
                            messages.update(|messages| messages.push(msg.build()));
                        }
                    }
                });
            }
        };

        view! {
            <InputText
                value=project.properties().name().read_only()
                oninput=Callback::new(oninput)
                debounce=INPUT_DEBOUNCE
                class="input-compact w-full"
            />
        }
    }
}

mod description {
    use super::{
        super::common::description::Editor as DescriptionEditor, update_properties, INPUT_DEBOUNCE,
    };
    use crate::{components::message::Builder as Message, pages::project::state, types::Messages};
    use leptos::*;

    #[component]
    pub fn Editor() -> impl IntoView {
        let project = expect_context::<state::Project>();
        let messages = expect_context::<Messages>();

        let oninput = {
            let project = project.clone();
            let messages = messages.write_only();
            move |value: Option<String>| {
                let mut properties = project.as_properties();
                properties.description = value;

                spawn_local({
                    let messages = messages.clone();
                    async move {
                        if let Err(err) = update_properties(properties).await {
                            tracing::error!(?err);
                            let mut msg = Message::error("Could not save container.");
                            msg.body(format!("{err:?}"));
                            messages.update(|messages| messages.push(msg.build()));
                        }
                    }
                });
            }
        };

        view! {
            <DescriptionEditor
                value=project.properties().description().read_only()
                oninput=Callback::new(oninput)
                debounce=INPUT_DEBOUNCE
                class="input-compact w-full align-top"
            />
        }
    }
}

async fn update_properties(update: core::project::Project) -> Result<(), local::error::IoSerde> {
    #[derive(Serialize)]
    struct Args {
        update: core::project::Project,
    }

    tauri_sys::core::invoke_result("project_properties_update", Args { update }).await
}

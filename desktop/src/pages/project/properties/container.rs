use crate::{invoke::invoke_result, pages::project::state};
use kind::Editor as Kind;
use leptos::*;
use name::Editor as Name;
use serde::Serialize;
use std::path::PathBuf;
use syre_core::types::ResourceId;
use syre_local_database as db;

const INPUT_DEBOUNCE: f64 = 350.0;

#[component]
pub fn Editor(container: state::Container) -> impl IntoView {
    let db::state::DataResource::Ok(properties) = container.properties().get_untracked() else {
        panic!("invalid state");
    };

    view! {
        <div>
            <form on:submit=|e| e.prevent_default()>
                <div>
                    <label>
                        "Name"
                        <Name
                            value=properties.name().get_untracked()
                            container=properties.rid().read_only()
                        />
                    </label>
                </div>
                <div>
                    <label>
                        "Type"
                        <Kind
                            value=properties.kind().get_untracked()
                            container=properties.rid().read_only()
                        />
                    </label>
                </div>
            </form>

        </div>
    }
}

mod name {
    use super::INPUT_DEBOUNCE;
    use crate::{components::message::Builder as Message, pages::project::state, types::Messages};
    use leptos::*;
    use serde::Serialize;
    use std::{ffi::OsString, path::PathBuf};
    use syre_core::types::ResourceId;
    use syre_desktop_lib as lib;
    use syre_local_database as db;

    #[component]
    pub fn Editor(
        /// Initial value.
        value: impl Into<String>,
        container: ReadSignal<ResourceId>,
    ) -> impl IntoView {
        let project = expect_context::<state::Project>();
        let graph = expect_context::<state::Graph>();
        let messages = expect_context::<Messages>();
        let (name_value, set_name_value) = create_signal(Into::<String>::into(value));
        let name_value = leptos_use::signal_debounced(name_value, INPUT_DEBOUNCE);
        let (error, set_error) = create_signal(false);

        create_effect({
            let project = project.clone();
            let graph = graph.clone();
            let container = container.clone();
            let messages = messages.write_only();
            move |_| {
                set_error(false);
                let name = name_value.get();
                if name.is_empty() {
                    set_error(true);
                    return;
                }

                let node = container.with(|rid| graph.find_by_id(rid).unwrap());
                let path = graph.path(&node).unwrap();

                let project = project.rid().get_untracked();
                let messages = messages.clone();
                spawn_local(async move {
                    if let Err(err) = rename_container(project, path, name).await {
                        let mut msg = Message::error("Could not save container");
                        msg.body(format!("{err:?}"));
                        messages.update(|messages| messages.push(msg.build()));
                    }
                });
            }
        });

        view! {
            <input
                name="name"
                class=("border-red", error)

                prop:value=name_value
                on:input=move |e| {
                    set_name_value(event_target_value(&e));
                }
            />
        }
    }

    async fn rename_container(
        project: ResourceId,
        container: impl Into<PathBuf>,
        name: impl Into<OsString>,
    ) -> Result<(), lib::command::container::error::Rename> {
        #[derive(Serialize)]
        struct RenameContainerArgs {
            project: ResourceId,
            container: PathBuf,
            #[serde(with = "db::serde_os_string")]
            name: OsString,
        }

        crate::invoke::invoke_result(
            "container_rename",
            RenameContainerArgs {
                project,
                container: container.into(),
                name: name.into(),
            },
        )
        .await
    }
}

mod kind {
    use super::{update_properties, INPUT_DEBOUNCE};
    use crate::{
        components::{form::InputDebounced, message::Builder as Message},
        pages::project::state,
        types::Messages,
    };
    use leptos::*;
    use syre_core::types::ResourceId;
    use syre_local_database as db;

    #[component]
    pub fn Editor(
        /// Initial value.
        value: Option<String>,
        container: ReadSignal<ResourceId>,
    ) -> impl IntoView {
        let project = expect_context::<state::Project>();
        let graph = expect_context::<state::Graph>();
        let messages = expect_context::<Messages>();

        // TODO: Handle errors with messages.
        // See https://github.com/leptos-rs/leptos/issues/2041
        let oninput = {
            // let messages = messages.write_only();
            move |kind: String| {
                let node = container.with(|rid| graph.find_by_id(rid).unwrap());
                let path = graph.path(&node).unwrap();
                let mut properties = node.properties().with_untracked(|properties| {
                    let db::state::DataResource::Ok(properties) = properties else {
                        panic!("invalid state");
                    };

                    properties.as_properties()
                });
                properties.kind = if kind.is_empty() { None } else { Some(kind) };

                let project = project.rid().get_untracked();
                // let messages = messages.clone();
                spawn_local(async move {
                    if let Err(err) = update_properties(project, path, properties).await {
                        tracing::error!(?err);
                        let mut msg = Message::error("Could not save container");
                        msg.body(format!("{err:?}"));
                        // messages.update(|messages| messages.push(msg.build()));
                    }
                });
            }
        };

        view! {
            <InputDebounced
                value=value.unwrap_or(String::new())
                debounce=INPUT_DEBOUNCE
                oninput=oninput
            />
        }
    }
}

async fn update_properties(
    project: ResourceId,
    container: impl Into<PathBuf>,
    properties: syre_core::project::ContainerProperties,
) -> Result<(), ()> {
    #[derive(Serialize)]
    struct UpdateContainerPropertiesArgs {
        project: ResourceId,
        container: PathBuf,
        properties: syre_core::project::ContainerProperties,
    }

    invoke_result(
        "container_properties_update",
        UpdateContainerPropertiesArgs {
            project,
            container: container.into(),
            properties,
        },
    )
    .await
}

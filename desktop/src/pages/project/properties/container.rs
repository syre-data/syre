use crate::{invoke::invoke_result, pages::project::state};
use leptos::{ev::SubmitEvent, *};
use serde::Serialize;
use std::{ffi::OsString, path::PathBuf};
use syre_core::types::ResourceId;
use syre_desktop_lib as lib;
use syre_local_database as db;

const INPUT_DEBOUNCE: f64 = 250.0;

#[component]
pub fn Editor(container: state::Container) -> impl IntoView {
    let project = expect_context::<state::Project>();
    let graph = expect_context::<state::Graph>();
    let (name_value, set_name_value) =
        create_signal(container.properties().with_untracked(|properties| {
            let db::state::DataResource::Ok(properties) = properties else {
                panic!("invalid state");
            };

            properties.name().get_untracked()
        }));
    let name_value = leptos_use::signal_debounced(name_value, INPUT_DEBOUNCE);
    let (name_error, set_name_error) = create_signal(None);

    let update_name = {
        let project = project.clone();
        let graph = graph.clone();
        let container = container.clone();
        move |e| {
            set_name_error(None);
            let name = event_target_value(&e);
            set_name_value(name.clone());
            if name.is_empty() {
                set_name_error(Some("Name is required".to_string()));
                return;
            }

            let db::state::DataResource::Ok(properties) = container.properties().get_untracked()
            else {
                panic!("invalid state");
            };

            let node = properties
                .rid()
                .with_untracked(|rid| graph.find_by_id(rid).unwrap());
            let path = graph.path(&node).unwrap();

            let project = project.rid().get_untracked();
            spawn_local(async move {
                if let Err(err) = rename_container(project, path, name).await {
                    set_name_error(Some(format!("{err:?}")));
                }
            });
        }
    };

    let submit = move |e: SubmitEvent| {
        e.prevent_default();
    };

    view! {
        <div>
            <form on:submit=submit>
                <div>
                    <label>
                        "Name" <input name="name" prop:value=name_value on:input=update_name/>
                    </label>
                    <div>{name_error}</div>
                </div>
            </form>

        </div>
    }
}

async fn rename_container(
    project: ResourceId,
    container: impl Into<PathBuf>,
    name: impl Into<OsString>,
) -> Result<(), lib::command::container::error::Rename> {
    invoke_result(
        "container_rename",
        RenameContainerArgs {
            project,
            container: container.into(),
            name: name.into(),
        },
    )
    .await
}

#[derive(Serialize)]
struct RenameContainerArgs {
    project: ResourceId,
    container: PathBuf,
    #[serde(with = "db::serde_os_string")]
    name: OsString,
}

//! Container editor widget.
use super::analysis_associations_editor::AnalysisAssociationsEditor;
use crate::app::{AppStateAction, AppStateReducer};
use crate::commands::container::{update_properties, UpdatePropertiesArgs};
use crate::components::canvas::{GraphStateAction, GraphStateReducer};
use syre_core::project::ContainerProperties;
use syre_core::types::ResourceId;
use syre_ui::types::Message;
use syre_ui::widgets::container::ContainerPropertiesEditor;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct ContainerEditorProps {
    pub rid: ResourceId,

    #[prop_or_default]
    pub onsave: Callback<()>,
}

#[function_component(ContainerEditor)]
pub fn container_editor(props: &ContainerEditorProps) -> Html {
    let app_state = use_context::<AppStateReducer>().unwrap();
    let graph_state = use_context::<GraphStateReducer>().unwrap();

    let container = graph_state
        .graph
        .get(&props.rid)
        .expect("`Container` not found");

    let properties = use_state(|| container.properties.clone());
    let dirty_state = use_state(|| false); // track if property changes come from user updates or
                                           // internal changes

    use_effect_with(container.clone(), {
        let dirty_state = dirty_state.setter();
        let properties = properties.setter();
        move |container| {
            properties.set(container.properties.clone());
            dirty_state.set(false);
        }
    });

    // Save changes on change
    use_effect_with(
        (
            props.rid.clone(),
            properties.clone(),
            (*dirty_state).clone(),
        ),
        {
            let app_state = app_state.dispatcher();
            let graph_state = graph_state.dispatcher();
            move |(rid, properties, dirty_state)| {
                if !*dirty_state {
                    return;
                }

                let rid = rid.clone();
                let properties = properties.clone();
                spawn_local(async move {
                    match update_properties(rid.clone(), (*properties).clone()).await {
                        Ok(_) => {
                            let update = UpdatePropertiesArgs {
                                rid,
                                properties: (*properties).clone(),
                            };

                            graph_state
                                .dispatch(GraphStateAction::UpdateContainerProperties(update));
                        }
                        Err(err) => {
                            let mut msg = Message::error("Could not save resource");
                            msg.set_details(format!("{err:?}"));
                            app_state.dispatch(AppStateAction::AddMessage(msg));
                        }
                    }
                });
            }
        },
    );

    let onchange = use_callback((), {
        let properties = properties.setter();
        let dirty_state = dirty_state.setter();

        move |update: ContainerProperties, _| {
            properties.set(update);
            dirty_state.set(true);
        }
    });

    html! {
        <div class={"syre-ui-editor px-xl"}>
            <ContainerPropertiesEditor
                properties={(*properties).clone()}
                onchange={onchange} />

            <AnalysisAssociationsEditor container={props.rid.clone()} />
            // TODO Allow Assets to be dropped here.
        </div>
    }
}

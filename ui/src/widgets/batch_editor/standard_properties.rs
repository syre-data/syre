//! Batch editor for [`StandardProperties`].
use std::rc::Rc;
use thot_core::project::Metadata;
use thot_core::project::StandardProperties;
use yew::prelude::*;

// **********************
// *** update builder ***
// **********************

#[derive(PartialEq, Default)]
pub struct StandardPropertiesUpdateBuilder<'a> {
    name: Option<&'a str>,
    kind: Option<&'a str>,
    description: Option<&'a str>,
    // tags: Option<Vec<String>>,
    // metadata: Option<Metadata>>,
}

impl<'a> StandardPropertiesUpdateBuilder<'a> {
    pub fn name(&self) -> Option<&'a str> {
        self.name
    }

    pub fn set_name(&mut self, name: &'a str) {
        self.name.insert(name);
    }

    pub fn clear_name(&mut self) {
        self.name.take();
    }

    pub fn kind(&self) -> Option<&'a str> {
        self.kind
    }

    pub fn set_kind(&mut self, kind: &'a str) {
        self.kind.insert(kind);
    }

    pub fn clear_kind(&mut self) {
        self.kind.take();
    }

    pub fn description(&self) -> Option<&'a str> {
        self.description
    }

    pub fn set_description(&mut self, description: &'a str) {
        self.description.insert(description);
    }

    pub fn clear_description(&mut self) {
        self.description.take();
    }
}

// ***************
// *** reducer ***
// ***************

enum StandardPropertiesUpdateStateAction {
    SetName(String),
    ClearName,
    SetKind(String),
    ClearKind,
    SetDescription(String),
    ClearDescription,
}

#[derive(PartialEq, Clone, Default)]
struct StandardPropertiesUpdateState {
    pub name: Option<String>,
    pub kind: Option<String>,
    pub description: Option<String>,
    pub tags: Option<Vec<String>>,
    pub metadata: Option<Metadata>,
}

impl Reducible for StandardPropertiesUpdateState {
    type Action = StandardPropertiesUpdateStateAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        let mut current = (*self).clone();
        match action {
            StandardPropertiesUpdateStateAction::SetName(value) => {
                let _ = current.name.insert(value);
            }

            StandardPropertiesUpdateStateAction::ClearName => {
                current.name.take();
            }

            StandardPropertiesUpdateStateAction::SetKind(value) => {
                let _ = current.kind.insert(value);
            }

            StandardPropertiesUpdateStateAction::ClearKind => {
                current.kind.take();
            }

            StandardPropertiesUpdateStateAction::SetDescription(value) => {
                let _ = current.description.insert(value);
            }

            StandardPropertiesUpdateStateAction::ClearDescription => {
                current.description.take();
            }
        }

        current.into()
    }
}

// *****************
// *** component ***
// *****************

#[derive(Properties, PartialEq)]
pub struct StandardPropertiesBatchEditorProps {
    pub values: Vec<StandardProperties>,
}

#[function_component(StandardPropertiesBatchEditor)]
pub fn standard_properties_batch_editor(props: &StandardPropertiesBatchEditorProps) -> Html {
    let updater_state = use_reducer(|| StandardPropertiesUpdateState::default());
    let name_ref = use_node_ref();
    let kind_ref = use_node_ref();
    let description_ref = use_node_ref();

    let onchange_name = {
        let updater_state = updater_state.clone();
        let elm = name_ref.clone();

        Callback::from(move |_: Event| {
            // update state
            let elm = elm
                .cast::<web_sys::HtmlInputElement>()
                .expect("could not cast `NodeRef` into element");

            let value = elm.value().trim().to_string();
            let action = if value.is_empty() {
                StandardPropertiesUpdateStateAction::ClearName
            } else {
                StandardPropertiesUpdateStateAction::SetName(value)
            };

            updater_state.dispatch(action);
        })
    };

    let onchange_kind = {
        let updater_state = updater_state.clone();
        let elm = kind_ref.clone();

        Callback::from(move |_: Event| {
            // update state
            let elm = elm
                .cast::<web_sys::HtmlInputElement>()
                .expect("could not cast `NodeRef` into element");

            let value = elm.value().trim().to_string();
            let action = if value.is_empty() {
                StandardPropertiesUpdateStateAction::ClearKind
            } else {
                StandardPropertiesUpdateStateAction::SetKind(value)
            };

            updater_state.dispatch(action);
        })
    };

    let onchange_description = {
        let updater_state = updater_state.clone();
        let elm = description_ref.clone();

        Callback::from(move |_: Event| {
            // update state
            let elm = elm
                .cast::<web_sys::HtmlTextAreaElement>()
                .expect("could not cast `NodeRef` into element");

            let value = elm.value().trim().to_string();
            let action = if value.is_empty() {
                StandardPropertiesUpdateStateAction::ClearDescription
            } else {
                StandardPropertiesUpdateStateAction::SetDescription(value)
            };

            updater_state.dispatch(action);
        })
    };

    html! {
        <form class={classes!("thot-ui-standard-properties-editor")}>
            <div class={classes!("form-field", "name")}>
                <label>
                    { "Name" }
                    <input
                        ref={name_ref}
                        placeholder={"(no change)"}
                        value={updater_state.name.clone()}
                        onchange={onchange_name} />
                </label>
            </div>

            <div class={classes!("form-field", "kind")}>
                <label>
                    { "Type" }
                    <input
                        ref={kind_ref}
                        placeholder={"(no type)"}
                        value={updater_state.kind.clone()}
                        onchange={onchange_kind} />
                </label>
            </div>

            <div class={classes!("form-field", "description")}>
                <label>{ "Description" }
                    <textarea
                        ref={description_ref}
                        placeholder={"(no description)"}
                        value={updater_state.description.clone()}
                        onchange={onchange_description}></textarea>
                </label>
            </div>

            // @todo
            // <div class={classes!("form-field", "tags")}>
            //     <label>
            //         { "Tags" }
            //         <TagsEditor
            //             value={properties_state.tags.clone()}
            //             onchange={onchange_tags} />
            //     </label>
            // </div>

            // @todo
            // <div class={classes!("form-field", "metadata")}>
            //     <h4>{ "Metadata" }</h4>
            //     <MetadataBatchEditor
            //         value={properties_state.metadata.clone()}
            //         onchange={onchange_metadata} />
            // </div>
    </form>
    }
}

#[cfg(test)]
#[path = "./standard_properties_test.rs"]
mod standard_properties_test;

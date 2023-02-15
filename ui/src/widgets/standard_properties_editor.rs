//! Properties editor for [`Contaier`](thot_core::project::Container)s.
use super::MetadataEditor;
use std::ops::{Deref, DerefMut};
use std::rc::Rc;
use thot_core::project::{Metadata, StandardProperties};
use yew::prelude::*;

// ************************
// *** Properties State ***
// ************************
enum StandardPropertiesStateAction {
    SetName(Option<String>),
    SetKind(Option<String>),
    SetDescription(Option<String>),
    SetTags(Vec<String>),
    SetMetadata(Metadata),
}

#[derive(PartialEq, Clone)]
struct StandardPropertiesState(StandardProperties);

impl From<StandardProperties> for StandardPropertiesState {
    fn from(props: StandardProperties) -> Self {
        Self(props)
    }
}

impl Into<StandardProperties> for StandardPropertiesState {
    fn into(self) -> StandardProperties {
        self.0
    }
}

impl Deref for StandardPropertiesState {
    type Target = StandardProperties;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for StandardPropertiesState {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Reducible for StandardPropertiesState {
    type Action = StandardPropertiesStateAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        let mut current = (*self).clone();

        match action {
            StandardPropertiesStateAction::SetName(name) => {
                current.name = name;
            }
            StandardPropertiesStateAction::SetKind(kind) => {
                current.kind = kind;
            }
            StandardPropertiesStateAction::SetDescription(description) => {
                current.description = description;
            }
            StandardPropertiesStateAction::SetTags(tags) => {
                current.tags = tags;
            }
            StandardPropertiesStateAction::SetMetadata(metadata) => {
                current.metadata = metadata;
            }
        }

        current.into()
    }
}

// ****************************
// *** Properties Component ***
// ****************************

/// Properties for [`StandardPropertiesEditor`].
#[derive(PartialEq, Properties)]
pub struct StandardPropertiesEditorProps {
    #[prop_or_default]
    pub class: Classes,

    /// Initial value.
    #[prop_or_else(StandardProperties::new)]
    pub properties: StandardProperties,

    /// Callback when value changes.
    #[prop_or_default]
    pub onchange: Callback<StandardProperties>,
}

/// [`StandardProperties`] editor.
#[function_component(StandardPropertiesEditor)]
pub fn standard_properties_editor(props: &StandardPropertiesEditorProps) -> Html {
    let properties_state =
        use_reducer(|| Into::<StandardPropertiesState>::into(props.properties.clone()));

    let name_ref = use_node_ref();
    let kind_ref = use_node_ref();
    let description_ref = use_node_ref();
    let tags_ref = use_node_ref();

    let onchange_name = {
        let properties_state = properties_state.clone();
        let elm = name_ref.clone();

        Callback::from(move |_: Event| {
            // update state
            let elm = elm
                .cast::<web_sys::HtmlInputElement>()
                .expect("could not cast `NodeRef` into element");

            let value = elm.value().trim().to_string();
            let value = if value.is_empty() { None } else { Some(value) };
            properties_state.dispatch(StandardPropertiesStateAction::SetName(value));
        })
    };

    let onchange_kind = {
        let properties_state = properties_state.clone();
        let elm = kind_ref.clone();

        Callback::from(move |_: Event| {
            // update state
            let elm = elm
                .cast::<web_sys::HtmlInputElement>()
                .expect("could not cast `NodeRef` into element");

            let value = elm.value().trim().to_string();
            let value = if value.is_empty() { None } else { Some(value) };
            properties_state.dispatch(StandardPropertiesStateAction::SetKind(value));
        })
    };

    let onchange_description = {
        let properties_state = properties_state.clone();
        let elm = description_ref.clone();

        Callback::from(move |_: Event| {
            // update state
            let elm = elm
                .cast::<web_sys::HtmlTextAreaElement>()
                .expect("could not cast `NodeRef` into element");

            let value = elm.value().trim().to_string();
            let value = if value.is_empty() { None } else { Some(value) };
            properties_state.dispatch(StandardPropertiesStateAction::SetDescription(value));
        })
    };

    let tags_val = properties_state.tags.join(", ");
    let onchange_tags = {
        let properties_state = properties_state.clone();
        let elm = tags_ref.clone();

        Callback::from(move |_: Event| {
            // update state
            let elm = elm
                .cast::<web_sys::HtmlInputElement>()
                .expect("could not cast `NodeRef` into element");

            let value = elm
                .value()
                .split(",")
                .into_iter()
                .filter_map(|t| {
                    let t = t.trim().to_string();
                    if t.is_empty() {
                        None
                    } else {
                        Some(t)
                    }
                })
                .collect::<Vec<String>>();

            properties_state.dispatch(StandardPropertiesStateAction::SetTags(value));
        })
    };

    let onchange_metadata = {
        let properties_state = properties_state.clone();

        Callback::from(move |value: Metadata| {
            properties_state.dispatch(StandardPropertiesStateAction::SetMetadata(value));
        })
    };

    {
        let properties_state = properties_state.clone();
        let onchange = props.onchange.clone();

        use_effect_with_deps(
            move |properties_state| {
                onchange.emit((**properties_state).clone().into());
            },
            properties_state,
        );
    }

    html! {
        <form>
            <div>
                <label>
                    { "Name" }
                    <input
                        ref={name_ref}
                        placeholder={"(no name)"}
                        value={properties_state.name.clone()}
                        onchange={onchange_name} />
                </label>
            </div>

            <div>
                <label>
                    { "Type" }
                    <input
                        ref={kind_ref}
                        placeholder={"(no type)"}
                        value={properties_state.kind.clone()}
                        onchange={onchange_kind} />
                </label>
            </div>

            <div>
                <label for={"container-properties-editor-description"}>{ "Description" }</label>
                <textarea
                    ref={description_ref}
                    placeholder={"(no description)"}
                    value={properties_state.description.clone()}
                    onchange={onchange_description}></textarea>
            </div>
            <div>
                <label>
                    { "Tags" }
                    <input
                        ref={tags_ref}
                        placeholder={"(no tags)"}
                        value={tags_val}
                        onchange={onchange_tags} />
                </label>
            </div>

            <div>
                <h4>{ "Metadata" }</h4>
                <MetadataEditor
                    value={properties_state.metadata.clone()}
                    onchange={onchange_metadata} />
            </div>
        </form>
    }
}

#[cfg(test)]
#[path = "./standard_properties_editor_test.rs"]
mod standard_properties_editor_test;

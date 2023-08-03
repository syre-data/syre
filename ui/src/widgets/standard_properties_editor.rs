//! Properties editor for [`Contaier`](thot_core::project::Container)s.
use super::{MetadataEditor, TagsEditor};
use std::ops::{Deref, DerefMut};
use std::rc::Rc;
use thot_core::project::{Metadata, StandardProperties};
use yew::prelude::*;

// ************************
// *** Properties State ***
// ************************
enum StandardPropertiesStateAction {
    SetName(String),
    ClearName,
    SetKind(String),
    ClearKind,
    SetDescription(String),
    ClearDescription,
    SetTags(Vec<String>),
    SetMetadata(Metadata),
    Update(StandardProperties),
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
            StandardPropertiesStateAction::SetName(value) => {
                let _ = current.name.insert(value);
            }

            StandardPropertiesStateAction::ClearName => {
                current.name.take();
            }

            StandardPropertiesStateAction::SetKind(value) => {
                let _ = current.kind.insert(value);
            }

            StandardPropertiesStateAction::ClearKind => {
                current.kind.take();
            }

            StandardPropertiesStateAction::SetDescription(value) => {
                let _ = current.description.insert(value);
            }

            StandardPropertiesStateAction::ClearDescription => {
                current.description.take();
            }

            StandardPropertiesStateAction::SetTags(tags) => {
                current.tags = tags;
            }

            StandardPropertiesStateAction::SetMetadata(metadata) => {
                current.metadata = metadata;
            }

            StandardPropertiesStateAction::Update(properties) => {
                return Self(properties).into();
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
#[tracing::instrument(skip(props))]
#[function_component(StandardPropertiesEditor)]
pub fn standard_properties_editor(props: &StandardPropertiesEditorProps) -> Html {
    let properties_state =
        use_reducer(|| Into::<StandardPropertiesState>::into(props.properties.clone()));

    let dirty_state = use_state(|| false);

    let name_ref = use_node_ref();
    let kind_ref = use_node_ref();
    let description_ref = use_node_ref();

    {
        let properties_state = properties_state.clone();
        let dirty_state = dirty_state.clone();

        use_effect_with_deps(
            move |properties| {
                dirty_state.set(false);
                properties_state
                    .dispatch(StandardPropertiesStateAction::Update(properties.clone()));
            },
            props.properties.clone(),
        );
    }

    let onchange_name = {
        let properties_state = properties_state.clone();
        let dirty_state = dirty_state.clone();
        let elm = name_ref.clone();

        Callback::from(move |_: Event| {
            // update state
            let elm = elm
                .cast::<web_sys::HtmlInputElement>()
                .expect("could not cast `NodeRef` into element");

            let value = elm.value().trim().to_string();
            let action = if value.is_empty() {
                StandardPropertiesStateAction::ClearName
            } else {
                StandardPropertiesStateAction::SetName(value)
            };

            properties_state.dispatch(action);
            dirty_state.set(true);
        })
    };

    let onchange_kind = {
        let properties_state = properties_state.clone();
        let dirty_state = dirty_state.clone();
        let elm = kind_ref.clone();

        Callback::from(move |_: Event| {
            // update state
            let elm = elm
                .cast::<web_sys::HtmlInputElement>()
                .expect("could not cast `NodeRef` into element");

            let value = elm.value().trim().to_string();
            let action = if value.is_empty() {
                StandardPropertiesStateAction::ClearKind
            } else {
                StandardPropertiesStateAction::SetKind(value)
            };

            properties_state.dispatch(action);
            dirty_state.set(true);
        })
    };

    let onchange_description = {
        let properties_state = properties_state.clone();
        let dirty_state = dirty_state.clone();
        let elm = description_ref.clone();

        Callback::from(move |_: Event| {
            // update state
            let elm = elm
                .cast::<web_sys::HtmlTextAreaElement>()
                .expect("could not cast `NodeRef` into element");

            let value = elm.value().trim().to_string();
            let action = if value.is_empty() {
                StandardPropertiesStateAction::ClearDescription
            } else {
                StandardPropertiesStateAction::SetDescription(value)
            };

            properties_state.dispatch(action);
            dirty_state.set(true);
        })
    };

    let onchange_tags = {
        let properties_state = properties_state.clone();
        let dirty_state = dirty_state.clone();

        Callback::from(move |value: Vec<String>| {
            properties_state.dispatch(StandardPropertiesStateAction::SetTags(value));
            dirty_state.set(true);
        })
    };

    let onchange_metadata = {
        let properties_state = properties_state.clone();
        let dirty_state = dirty_state.clone();

        Callback::from(move |value: Metadata| {
            properties_state.dispatch(StandardPropertiesStateAction::SetMetadata(value));
            dirty_state.set(true);
        })
    };

    {
        let properties_state = properties_state.clone();
        let dirty_state = dirty_state.clone();
        let onchange = props.onchange.clone();

        use_effect_with_deps(
            move |(properties_state, dirty_state)| {
                if !(**dirty_state) {
                    return;
                }
                onchange.emit((**properties_state).clone().into());
            },
            (properties_state, dirty_state),
        );
    }

    html! {
        <form class={classes!("thot-ui-standard-properties-editor")}>
            <div class={classes!("form-field", "name")}>
                <label>
                    <h3> { "Name" } </h3>
                    <input
                        ref={name_ref}
                        placeholder={"(no name)"}
                        value={properties_state.name.clone()}
                        onchange={onchange_name} />
                </label>
            </div>

            <div class={classes!("form-field", "kind")}>
                <label>
                    <h3> { "Type" } </h3>
                    <input
                        ref={kind_ref}
                        placeholder={"(no type)"}
                        value={properties_state.kind.clone()}
                        onchange={onchange_kind} />
                </label>
            </div>

            <div class={classes!("form-field", "description")}>
                <label>
                    <h3> { "Description" } </h3>
                    <textarea
                        ref={description_ref}
                        placeholder={"(no description)"}
                        value={properties_state.description.clone()}
                        onchange={onchange_description}></textarea>
                </label>
            </div>

            <div class={classes!("form-field", "tags")}>
                <label>
                    <h3> { "Tags" } </h3>
                    <TagsEditor
                        value={properties_state.tags.clone()}
                        onchange={onchange_tags} />
                </label>
            </div>

            <div class={classes!("form-field", "metadata")}>
                <MetadataEditor
                    value={properties_state.metadata.clone()}
                    onchange={onchange_metadata} />
            </div>
        </form>
    }
}

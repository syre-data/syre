//! Properties editor for [`Contaier`](thot_core::project::Container)s.
use super::super::{MetadataEditor, TagsEditor};
use std::ops::{Deref, DerefMut};
use std::rc::Rc;
use thot_core::project::{ContainerProperties, Metadata};
use yew::prelude::*;

// ************************
// *** Properties State ***
// ************************
enum ContainerPropertiesStateAction {
    SetName(String),
    SetKind(String),
    ClearKind,
    SetDescription(String),
    ClearDescription,
    SetTags(Vec<String>),
    SetMetadata(Metadata),
    Update(ContainerProperties),
}

#[derive(PartialEq, Clone)]
struct ContainerPropertiesState(ContainerProperties);

impl From<ContainerProperties> for ContainerPropertiesState {
    fn from(props: ContainerProperties) -> Self {
        Self(props)
    }
}

impl Into<ContainerProperties> for ContainerPropertiesState {
    fn into(self) -> ContainerProperties {
        self.0
    }
}

impl Deref for ContainerPropertiesState {
    type Target = ContainerProperties;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ContainerPropertiesState {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Reducible for ContainerPropertiesState {
    type Action = ContainerPropertiesStateAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        let mut current = (*self).clone();
        match action {
            ContainerPropertiesStateAction::SetName(value) => {
                current.name = value;
            }

            ContainerPropertiesStateAction::SetKind(value) => {
                let _ = current.kind.insert(value);
            }

            ContainerPropertiesStateAction::ClearKind => {
                current.kind.take();
            }

            ContainerPropertiesStateAction::SetDescription(value) => {
                let _ = current.description.insert(value);
            }

            ContainerPropertiesStateAction::ClearDescription => {
                current.description.take();
            }

            ContainerPropertiesStateAction::SetTags(tags) => {
                current.tags = tags;
            }

            ContainerPropertiesStateAction::SetMetadata(metadata) => {
                current.metadata = metadata;
            }

            ContainerPropertiesStateAction::Update(properties) => {
                return Self(properties).into();
            }
        }

        current.into()
    }
}

// ****************************
// *** Properties Component ***
// ****************************

/// Properties for [`ContainerPropertiesEditor`].
#[derive(PartialEq, Properties)]
pub struct ContainerPropertiesEditorProps {
    #[prop_or_default]
    pub class: Classes,

    /// Initial value.
    pub properties: ContainerProperties,

    /// Callback when value changes.
    #[prop_or_default]
    pub onchange: Callback<ContainerProperties>,
}

/// [`ContainerProperties`] editor.
#[tracing::instrument(skip(props))]
#[function_component(ContainerPropertiesEditor)]
pub fn Container_properties_editor(props: &ContainerPropertiesEditorProps) -> Html {
    let properties_state =
        use_reducer(|| Into::<ContainerPropertiesState>::into(props.properties.clone()));

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
                    .dispatch(ContainerPropertiesStateAction::Update(properties.clone()));
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
            properties_state.dispatch(ContainerPropertiesStateAction::SetName(value));
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
                ContainerPropertiesStateAction::ClearKind
            } else {
                ContainerPropertiesStateAction::SetKind(value)
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
                ContainerPropertiesStateAction::ClearDescription
            } else {
                ContainerPropertiesStateAction::SetDescription(value)
            };

            properties_state.dispatch(action);
            dirty_state.set(true);
        })
    };

    let onchange_tags = {
        let properties_state = properties_state.clone();
        let dirty_state = dirty_state.clone();

        Callback::from(move |value: Vec<String>| {
            properties_state.dispatch(ContainerPropertiesStateAction::SetTags(value));
            dirty_state.set(true);
        })
    };

    let onchange_metadata = {
        let properties_state = properties_state.clone();
        let dirty_state = dirty_state.clone();

        Callback::from(move |value: Metadata| {
            properties_state.dispatch(ContainerPropertiesStateAction::SetMetadata(value));
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
        <form class={classes!("thot-ui-Container-properties-editor")}>
            <div class={classes!("form-field", "name")}>
                <label>
                    <h3> { "Name" } </h3>
                    <input
                        ref={name_ref}
                        placeholder={"(no name)"}
                        min={"1"}
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

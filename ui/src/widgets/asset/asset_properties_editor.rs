//! Properties editor for [`Contaier`](syre_core::project::Container)s.
use super::super::{MetadataEditor, TagsEditor};
use std::ops::{Deref, DerefMut};
use std::rc::Rc;
use syre_core::project::{AssetProperties, Metadata};
use yew::prelude::*;

// ************************
// *** Properties State ***
// ************************
enum AssetPropertiesStateAction {
    SetName(String),
    ClearName,
    SetKind(String),
    ClearKind,
    SetDescription(String),
    ClearDescription,
    SetTags(Vec<String>),
    SetMetadata(Metadata),
    Update(AssetProperties),
}

#[derive(PartialEq, Clone)]
struct AssetPropertiesState(AssetProperties);

impl From<AssetProperties> for AssetPropertiesState {
    fn from(props: AssetProperties) -> Self {
        Self(props)
    }
}

impl Into<AssetProperties> for AssetPropertiesState {
    fn into(self) -> AssetProperties {
        self.0
    }
}

impl Deref for AssetPropertiesState {
    type Target = AssetProperties;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for AssetPropertiesState {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Reducible for AssetPropertiesState {
    type Action = AssetPropertiesStateAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        let mut current = (*self).clone();
        match action {
            AssetPropertiesStateAction::SetName(value) => {
                let _ = current.name.insert(value);
            }

            AssetPropertiesStateAction::ClearName => {
                current.name.take();
            }

            AssetPropertiesStateAction::SetKind(value) => {
                let _ = current.kind.insert(value);
            }

            AssetPropertiesStateAction::ClearKind => {
                current.kind.take();
            }

            AssetPropertiesStateAction::SetDescription(value) => {
                let _ = current.description.insert(value);
            }

            AssetPropertiesStateAction::ClearDescription => {
                current.description.take();
            }

            AssetPropertiesStateAction::SetTags(tags) => {
                current.tags = tags;
            }

            AssetPropertiesStateAction::SetMetadata(metadata) => {
                current.metadata = metadata;
            }

            AssetPropertiesStateAction::Update(properties) => {
                return Self(properties).into();
            }
        }

        current.into()
    }
}

// ****************************
// *** Properties Component ***
// ****************************

/// Properties for [`AssetPropertiesEditor`].
#[derive(PartialEq, Properties)]
pub struct AssetPropertiesEditorProps {
    #[prop_or_default]
    pub class: Classes,

    /// Initial value.
    #[prop_or_else(AssetProperties::new)]
    pub properties: AssetProperties,

    /// Callback when value changes.
    #[prop_or_default]
    pub onchange: Callback<AssetProperties>,
}

/// [`AssetProperties`] editor.
#[function_component(AssetPropertiesEditor)]
pub fn asset_properties_editor(props: &AssetPropertiesEditorProps) -> Html {
    let properties_state =
        use_reducer(|| Into::<AssetPropertiesState>::into(props.properties.clone()));

    let dirty_state = use_state(|| false);

    let name_ref = use_node_ref();
    let kind_ref = use_node_ref();
    let description_ref = use_node_ref();

    {
        let properties_state = properties_state.clone();
        let dirty_state = dirty_state.clone();

        use_effect_with(props.properties.clone(), move |properties| {
            dirty_state.set(false);
            properties_state.dispatch(AssetPropertiesStateAction::Update(properties.clone()));
        });
    }

    let onchange_name = use_callback((), {
        let properties_state = properties_state.dispatcher();
        let dirty_state = dirty_state.setter();
        let elm = name_ref.clone();

        move |_: Event, _| {
            // update state
            let elm = elm
                .cast::<web_sys::HtmlInputElement>()
                .expect("could not cast `NodeRef` into element");

            let value = elm.value().trim().to_string();
            let action = if value.is_empty() {
                AssetPropertiesStateAction::ClearName
            } else {
                AssetPropertiesStateAction::SetName(value)
            };

            properties_state.dispatch(action);
            dirty_state.set(true);
        }
    });

    let onchange_kind = use_callback((), {
        let properties_state = properties_state.clone();
        let dirty_state = dirty_state.clone();
        let elm = kind_ref.clone();

        move |_: Event, _| {
            // update state
            let elm = elm
                .cast::<web_sys::HtmlInputElement>()
                .expect("could not cast `NodeRef` into element");

            let value = elm.value().trim().to_string();
            let action = if value.is_empty() {
                AssetPropertiesStateAction::ClearKind
            } else {
                AssetPropertiesStateAction::SetKind(value)
            };

            properties_state.dispatch(action);
            dirty_state.set(true);
        }
    });

    let onchange_description = use_callback((), {
        let properties_state = properties_state.clone();
        let dirty_state = dirty_state.clone();
        let elm = description_ref.clone();

        move |_: Event, _| {
            // update state
            let elm = elm
                .cast::<web_sys::HtmlTextAreaElement>()
                .expect("could not cast `NodeRef` into element");

            let value = elm.value().trim().to_string();
            let action = if value.is_empty() {
                AssetPropertiesStateAction::ClearDescription
            } else {
                AssetPropertiesStateAction::SetDescription(value)
            };

            properties_state.dispatch(action);
            dirty_state.set(true);
        }
    });

    let onchange_tags = use_callback((), {
        let properties_state = properties_state.dispatcher();
        let dirty_state = dirty_state.setter();

        move |value: Vec<String>, _| {
            properties_state.dispatch(AssetPropertiesStateAction::SetTags(value));
            dirty_state.set(true);
        }
    });

    let onchange_metadata = {
        let properties_state = properties_state.clone();
        let dirty_state = dirty_state.clone();

        Callback::from(move |value: Metadata| {
            properties_state.dispatch(AssetPropertiesStateAction::SetMetadata(value));
            dirty_state.set(true);
        })
    };

    use_effect_with((properties_state.clone(), dirty_state.clone()), {
        let onchange = props.onchange.clone();

        move |(properties_state, dirty_state)| {
            if !(**dirty_state) {
                return;
            }
            onchange.emit((**properties_state).clone().into());
        }
    });

    html! {
        <form class={classes!("syre-ui-asset-properties-editor")}>
            <div class={classes!("form-field", "name")}>
                <label>
                    <h3> { "Name" } </h3>
                    <input
                        ref={name_ref}
                        placeholder={"(no name)"}
                        value={properties_state.name.clone().unwrap_or("".into())}
                        onchange={onchange_name} />
                </label>
            </div>

            <div class={classes!("form-field", "kind")}>
                <label>
                    <h3> { "Type" } </h3>
                    <input
                        ref={kind_ref}
                        placeholder={"(no type)"}
                        value={properties_state.kind.clone().unwrap_or("".into())}
                        onchange={onchange_kind} />
                </label>
            </div>

            <div class={classes!("form-field", "description")}>
                <label>
                    <h3> { "Description" } </h3>
                    <textarea
                        ref={description_ref}
                        placeholder={"(no description)"}
                        value={properties_state.description.clone().unwrap_or("".into())}
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

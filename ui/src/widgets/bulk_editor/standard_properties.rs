//! Bulk editor for [`StandardProperties`].
use super::tags::TagsBulkEditor;
use std::ops::{Deref, DerefMut};
use std::rc::Rc;
use thot_core::project::Metadata;
use thot_core::project::StandardProperties;
use yew::prelude::*;

// ***************
// *** reducer ***
// ***************

#[derive(PartialEq, Clone)]
enum BulkValue<T>
where
    T: PartialEq + Clone,
{
    Equal(T),
    Mixed,
}

/// State of a field.
///
/// # Fields
/// 0. Value.
/// 1. Dirty state. `true` indicates the value of the field is different from the original.
#[derive(PartialEq, Clone)]
struct FieldState<T>(T, bool)
where
    T: PartialEq + Clone;

impl<T> FieldState<T>
where
    T: PartialEq + Clone,
{
    pub fn new(value: T) -> Self {
        Self(value, false)
    }

    pub fn new_dirty(value: T) -> Self {
        Self(value, true)
    }

    /// Returns the value of the field.
    pub fn value(&self) -> &T {
        &self.0
    }

    /// Indicates if the field is dirty.
    pub fn dirty(&self) -> bool {
        self.1
    }

    /// Sets the field to be dirty.
    pub fn set_dirty(&mut self) {
        self.1 = true;
    }
}

impl<T> Deref for FieldState<T>
where
    T: PartialEq + Clone,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for FieldState<T>
where
    T: PartialEq + Clone,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

enum StandardPropertiesUpdateStateAction {
    /// Set all values from properties.
    SetValues(Vec<StandardProperties>),
    SetName(String),
    ClearName,
    SetKind(String),
    ClearKind,
    SetDescription(String),
    ClearDescription,
    AddTag(String),
    RemoveTag(String),
}

#[derive(PartialEq, Clone)]
struct StandardPropertiesUpdateState {
    name: FieldState<BulkValue<Option<String>>>,
    kind: FieldState<BulkValue<Option<String>>>,
    description: FieldState<BulkValue<Option<String>>>,
    tags: FieldState<Vec<String>>,
    // metadata: Option<Metadata>,
}

impl StandardPropertiesUpdateState {
    pub fn new(properties: &Vec<StandardProperties>) -> Self {
        let n_props = properties.len();
        let mut names = Vec::with_capacity(n_props);
        let mut kinds = Vec::with_capacity(n_props);
        let mut descriptions = Vec::with_capacity(n_props);
        for prop in properties.iter() {
            names.push(prop.name.clone());
            kinds.push(prop.kind.clone());
            descriptions.push(prop.description.clone());
        }

        names.sort();
        names.dedup();
        kinds.sort();
        kinds.dedup();
        descriptions.sort();
        descriptions.dedup();

        let name = match names.len() {
            1 => BulkValue::Equal(names[0].clone()),
            _ => BulkValue::Mixed,
        };

        let kind = match kinds.len() {
            1 => BulkValue::Equal(kinds[0].clone()),
            _ => BulkValue::Mixed,
        };

        let description = match descriptions.len() {
            1 => BulkValue::Equal(descriptions[0].clone()),
            _ => BulkValue::Mixed,
        };

        let mut tags = properties
            .iter()
            .map(|props| props.tags.clone())
            .flatten()
            .collect::<Vec<String>>();

        tags.sort();
        tags.dedup();

        Self {
            name: FieldState::new(name),
            kind: FieldState::new(kind),
            description: FieldState::new(description),
            tags: FieldState::new(tags),
        }
    }

    pub fn name(&self) -> &FieldState<BulkValue<Option<String>>> {
        &self.name
    }

    pub fn kind(&self) -> &FieldState<BulkValue<Option<String>>> {
        &self.kind
    }

    pub fn description(&self) -> &FieldState<BulkValue<Option<String>>> {
        &self.description
    }

    pub fn tags(&self) -> &FieldState<Vec<String>> {
        &self.tags
    }
}

impl Reducible for StandardPropertiesUpdateState {
    type Action = StandardPropertiesUpdateStateAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        let mut current = (*self).clone();
        match action {
            StandardPropertiesUpdateStateAction::SetValues(properties) => {
                current = Self::new(&properties);
            }

            StandardPropertiesUpdateStateAction::SetName(value) => {
                current.name = FieldState::new_dirty(BulkValue::Equal(Some(value)));
            }

            StandardPropertiesUpdateStateAction::ClearName => {
                current.name = FieldState::new_dirty(BulkValue::Equal(None));
            }

            StandardPropertiesUpdateStateAction::SetKind(value) => {
                current.kind = FieldState::new_dirty(BulkValue::Equal(Some(value)));
            }

            StandardPropertiesUpdateStateAction::ClearKind => {
                current.kind = FieldState::new_dirty(BulkValue::Equal(None));
            }

            StandardPropertiesUpdateStateAction::SetDescription(value) => {
                current.description = FieldState::new_dirty(BulkValue::Equal(Some(value)));
            }

            StandardPropertiesUpdateStateAction::ClearDescription => {
                current.description = FieldState::new_dirty(BulkValue::Equal(None));
            }

            StandardPropertiesUpdateStateAction::AddTag(tag) => {
                if !current.tags.contains(&tag) {
                    current.tags.push(tag);
                    current.tags.sort();
                    current.tags.set_dirty();
                }
            }

            StandardPropertiesUpdateStateAction::RemoveTag(tag) => {
                if let Ok(index) = current.tags.binary_search(&tag) {
                    current.tags.remove(index);
                    current.tags.set_dirty();
                }
            }
        }

        current.into()
    }
}

// *****************
// *** component ***
// *****************

#[derive(Properties, PartialEq)]
pub struct StandardPropertiesBulkEditorProps {
    pub properties: Vec<StandardProperties>,

    #[prop_or_default]
    onchange_name: Callback<Option<String>>,

    #[prop_or_default]
    onchange_kind: Callback<Option<String>>,

    #[prop_or_default]
    onchange_description: Callback<Option<String>>,

    #[prop_or_default]
    onadd_tag: Callback<String>,

    #[prop_or_default]
    onremove_tag: Callback<String>,
}

#[function_component(StandardPropertiesBulkEditor)]
pub fn standard_properties_bulk_editor(props: &StandardPropertiesBulkEditorProps) -> Html {
    assert!(
        props.properties.len() > 1,
        "bulk editor should not be used with fewer than two items."
    );

    let updater_state = use_reducer(|| StandardPropertiesUpdateState::new(&props.properties));
    let name_ref = use_node_ref();
    let kind_ref = use_node_ref();
    let description_ref = use_node_ref();

    {
        let properties = props.properties.clone();
        let updater_state = updater_state.clone();

        use_effect_with_deps(
            move |properties| {
                updater_state.dispatch(StandardPropertiesUpdateStateAction::SetValues(
                    properties.clone(),
                ));
            },
            properties,
        );
    }

    // -----------------------
    // --- change handlers ---
    // -----------------------

    let onchange_name = {
        let onchange_name = props.onchange_name.clone();
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
                StandardPropertiesUpdateStateAction::SetName(value.clone())
            };

            updater_state.dispatch(action);
            onchange_name.emit(value);
        })
    };

    let onchange_kind = {
        let onchange_kind = props.onchange_kind.clone();
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
                StandardPropertiesUpdateStateAction::SetKind(value.clone())
            };

            updater_state.dispatch(action);
            onchange_kind.emit(value.clone());
        })
    };

    let onchange_description = {
        let onchange_description = props.onchange_description.clone();
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
                StandardPropertiesUpdateStateAction::SetDescription(value.clone())
            };

            updater_state.dispatch(action);
            onchange_description.emit(value);
        })
    };

    let onadd_tag = {
        let onadd_tag = props.onadd_tag.clone();
        let updater_state = updater_state.clone();
        Callback::from(move |tag: String| {
            updater_state.dispatch(StandardPropertiesUpdateStateAction::AddTag(tag.clone()));
            onadd_tag.emit(tag);
        })
    };

    let onremove_tag = {
        let onremove_tag = props.onremove_tag.clone();
        let updater_state = updater_state.clone();
        Callback::from(move |tag: String| {
            updater_state.dispatch(StandardPropertiesUpdateStateAction::RemoveTag(tag.clone()));
            onremove_tag.emit(tag);
        })
    };

    // ------------
    // --- html ---
    // ------------

    html! {
        <form class={classes!("thot-ui-standard-properties-editor")}>
            <div class={classes!("form-field", "name")}>
                <label>
                    { "Name" }
                    <input
                        ref={name_ref}
                        placeholder={value_placeholder(updater_state.name())}
                        value={value_string(updater_state.name())}
                        onchange={onchange_name} />
                </label>
            </div>

            <div class={classes!("form-field", "kind")}>
                <label>
                    { "Type" }
                    <input
                        ref={kind_ref}
                        placeholder={value_placeholder(updater_state.kind())}
                        value={value_string(updater_state.kind())}
                        onchange={onchange_kind} />
                </label>
            </div>

            <div class={classes!("form-field", "description")}>
                <label>{ "Description" }
                    <textarea
                        ref={description_ref}
                        placeholder={value_placeholder(updater_state.description())}
                        value={value_string(updater_state.description())}
                        onchange={onchange_description}></textarea>
                </label>
            </div>

            <div class={classes!("form-field", "tags")}>
                <label>
                    { "Tags" }
                    <TagsBulkEditor
                        tags={(*updater_state.tags).clone()}
                        onadd={onadd_tag}
                        onremove={onremove_tag} />
                </label>
            </div>

            // @todo
            // <div class={classes!("form-field", "metadata")}>
            //     <h4>{ "Metadata" }</h4>
            //     <MetadataBulkEditor
            //         value={properties_state.metadata.clone()}
            //         onchange={onchange_metadata} />
            // </div>
    </form>
    }
}

// ***************
// *** helpers ***
// ***************

fn value_string(value: &BulkValue<Option<String>>) -> Option<String> {
    match value {
        BulkValue::Equal(val) => val.clone(),
        BulkValue::Mixed => None,
    }
}

fn value_placeholder<T>(value: &BulkValue<T>) -> &'static str
where
    T: PartialEq + Clone,
{
    match value {
        BulkValue::Equal(_) => "",
        BulkValue::Mixed => "(mixed)",
    }
}

#[cfg(test)]
#[path = "./standard_properties_test.rs"]
mod standard_properties_test;

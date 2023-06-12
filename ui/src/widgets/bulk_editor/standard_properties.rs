//! Bulk editor for [`StandardProperties`].
use std::rc::Rc;
use thot_core::project::Metadata;
use thot_core::project::StandardProperties;
use yew::prelude::*;

// **********************
// *** update builder ***
// **********************

// #[derive(PartialEq, Default)]
// pub struct StandardPropertiesUpdateBuilder<'a> {
//     name: Option<&'a str>,
//     kind: Option<&'a str>,
//     description: Option<&'a str>,
//     tags: Option<Vec<String>>,
//     // metadata: Option<Metadata>>,
// }

// impl<'a> StandardPropertiesUpdateBuilder<'a> {
//     pub fn name(&self) -> Option<&'a str> {
//         self.name
//     }

//     pub fn set_name(&mut self, name: &'a str) {
//         self.name.insert(name);
//     }

//     pub fn clear_name(&mut self) {
//         self.name.take();
//     }

//     pub fn kind(&self) -> Option<&'a str> {
//         self.kind
//     }

//     pub fn set_kind(&mut self, kind: &'a str) {
//         self.kind.insert(kind);
//     }

//     pub fn clear_kind(&mut self) {
//         self.kind.take();
//     }

//     pub fn description(&self) -> Option<&'a str> {
//         self.description
//     }

//     pub fn set_description(&mut self, description: &'a str) {
//         self.description.insert(description);
//     }

//     pub fn clear_description(&mut self) {
//         self.description.take();
//     }
// }

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
struct FieldState<T>(BulkValue<T>, bool)
where
    T: PartialEq + Clone;

impl<T> FieldState<T>
where
    T: PartialEq + Clone,
{
    pub fn new(value: BulkValue<T>) -> Self {
        Self(value, false)
    }

    pub fn new_dirty(value: BulkValue<T>) -> Self {
        Self(value, true)
    }

    /// Returns the value of the field.
    pub fn value(&self) -> &BulkValue<T> {
        &self.0
    }

    /// Indicates if the field is dirty.
    pub fn dirty(&self) -> bool {
        self.1
    }
}

enum StandardPropertiesUpdateStateAction {
    SetName(String),
    ClearName,
    SetKind(String),
    ClearKind,
    SetDescription(String),
    ClearDescription,
}

#[derive(PartialEq, Clone)]
struct StandardPropertiesUpdateState {
    name: FieldState<Option<String>>,
    kind: FieldState<Option<String>>,
    description: FieldState<Option<String>>,
    // pub tags: Option<Vec<String>>,
    // pub metadata: Option<Metadata>,
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

        Self {
            name: FieldState::new(name),
            kind: FieldState::new(kind),
            description: FieldState::new(description),
        }
    }

    pub fn name(&self) -> &FieldState<Option<String>> {
        &self.name
    }

    pub fn kind(&self) -> &FieldState<Option<String>> {
        &self.kind
    }

    pub fn description(&self) -> &FieldState<Option<String>> {
        &self.description
    }
}

impl Reducible for StandardPropertiesUpdateState {
    type Action = StandardPropertiesUpdateStateAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        let mut current = (*self).clone();
        match action {
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

    // -----------------------
    // --- change handlers ---
    // -----------------------

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
                        placeholder={value_placeholder(updater_state.name().value())}
                        value={value_string(updater_state.name().value())}
                        onchange={onchange_name} />
                </label>
            </div>

            <div class={classes!("form-field", "kind")}>
                <label>
                    { "Type" }
                    <input
                        ref={kind_ref}
                        placeholder={value_placeholder(updater_state.name().value())}
                        value={value_string(updater_state.kind().value())}
                        onchange={onchange_kind} />
                </label>
            </div>

            <div class={classes!("form-field", "description")}>
                <label>{ "Description" }
                    <textarea
                        ref={description_ref}
                        placeholder={value_placeholder(updater_state.description().value())}
                        value={value_string(updater_state.description().value())}
                        onchange={onchange_description}></textarea>
                </label>
            </div>

            // TODO
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

//! Inline metadata editor.
use super::Metadatum;
use super::MetadatumEditor;
use serde_json::Value as JsValue;
use std::rc::Rc;
use thot_core::project::Metadata;
use wasm_bindgen::JsCast;
use yew::prelude::*;

// *****************************
// *** Metadata Editor State ***
// *****************************

enum MetadataEditorStateAction {
    /// Add a new [`MetadatumEditor`].
    AddMetadatum,

    /// Remove the [`MetadatumEditor`] at the provided index.
    /// If an editor does not exist at the given index no action is performed.
    RemoveMetadatum(usize),
}

#[derive(PartialEq, Debug)]
struct MetadataEditorState {
    pub fields: Vec<Metadatum>,
    pub init_active_editors: Vec<usize>,
}

impl MetadataEditorState {
    pub fn new(metadata: Metadata) -> Self {
        let fields = metadata
            .into_iter()
            .map(|(k, v)| (Some(k), v))
            .collect::<Vec<Metadatum>>();

        Self {
            fields,
            init_active_editors: Vec::new(),
        }
    }
}

impl Reducible for MetadataEditorState {
    type Action = MetadataEditorStateAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        match action {
            MetadataEditorStateAction::AddMetadatum => {
                let mut fields = self.fields.clone();
                let mut init_active_editors = self.init_active_editors.clone();
                fields.push((None, JsValue::Null));
                init_active_editors.push(fields.len() - 1);

                Self {
                    fields,
                    init_active_editors,
                }
            }
            MetadataEditorStateAction::RemoveMetadatum(index) => {
                if index >= self.fields.len() {
                    return self;
                }

                let mut fields = self.fields.clone();
                fields.remove(index);

                Self {
                    fields,
                    init_active_editors: self.init_active_editors.clone(),
                }
            }
        }
        .into()
    }
}

// *****************
// *** Component ***
// *****************

#[derive(Properties, PartialEq)]
pub struct MetadataEditorProps {
    #[prop_or_default]
    pub class: Classes,

    /// Displayed if inactive and no value.
    #[prop_or_default]
    pub children: Children,

    #[prop_or(Metadata::new())]
    pub value: Metadata,

    /// Callback triggered when the value of a single `Metadatum` is changed.
    ///
    /// # Fields
    /// 1. New value
    #[prop_or_default]
    pub onchange: Option<Callback<Metadata>>,
}

#[function_component(MetadataEditor)]
pub fn metadata_editor(props: &MetadataEditorProps) -> Html {
    let editor_state = use_reducer(|| MetadataEditorState::new(props.value.clone()));

    let add_metadatum = {
        let editor_state = editor_state.clone();

        Callback::from(move |_: MouseEvent| {
            editor_state.dispatch(MetadataEditorStateAction::AddMetadatum)
        })
    };

    let remove_metadatum = {
        let editor_state = editor_state.clone();
        let onchange = props.onchange.clone();

        Callback::from(move |e: web_sys::MouseEvent| {
            // get index to remove
            let btn_remove = e
                .target()
                .expect("button could not be found")
                .dyn_ref::<web_sys::HtmlButtonElement>()
                .expect("could not cast target to button")
                .clone();

            let index: usize = btn_remove
                .dataset()
                .get("index")
                .expect("`index` not set on dataset")
                .parse()
                .expect("could not parse `index` as int");

            if index >= editor_state.fields.len() {
                panic!("metadatum index exceeded `fields` size");
            }

            // remove field
            editor_state.dispatch(MetadataEditorStateAction::RemoveMetadatum(index));

            if editor_state.fields[index].0.is_some() {
                if let Some(onchange) = onchange.clone() {
                    // calculate updated value
                    let value = fields_to_metadata(editor_state.fields.clone());
                    onchange.emit(value);
                }
            }
        })
    };

    let on_change = |index: usize| -> Callback<Metadatum> {
        let editor_state = editor_state.clone();
        let onchange = props.onchange.clone();

        Callback::from(move |metadatum| {
            if let Some(onchange) = onchange.clone() {
                // update changed field
                let mut fields = editor_state.fields.clone();
                fields[index] = metadatum;

                // calculate updated value
                let md = fields_to_metadata(fields);
                onchange.emit(md);
            }
        })
    };

    // css
    let mut class = props.class.clone();
    class.push("metadata");

    html! {
        <div {class}>
            <button onclick={add_metadatum}>{ "+" }</button>

            { editor_state.fields.iter().enumerate()
                .map(|(index, (name, value))| {
                    let active = editor_state.init_active_editors.contains(&index);

                    html! {
                        <div key={index} class={classes!("metadatum-controller")}>
                            <MetadatumEditor
                                name={name.clone()}
                                value={value.clone()}
                                {active}
                                onchange={on_change(index)} />

                            <button
                                onclick={remove_metadatum.clone()}
                                data-index={index.to_string()}>{
                                "X"
                            }</button>
                        </div>
                    }
                }).collect::<Html>()
            }
        </div>
    }
}

// ************************
// *** helper functions ***
// ************************

/// Calculates the [`Metadata`] value from a collection of [`MetadatumField`]s.
fn fields_to_metadata(fields: Vec<Metadatum>) -> Metadata {
    fields
        .into_iter()
        .filter_map(|(k, v)| {
            // filter any data with empty keys
            let Some(k) = k else {
                return None;
            };

            let k = k.trim().to_string();
            if k.is_empty() {
                return None;
            }

            return Some((k, v));
        })
        .collect::<Metadata>()
}

#[cfg(test)]
#[path = "./metadata_editor_test.rs"]
mod metadata_editor_test;

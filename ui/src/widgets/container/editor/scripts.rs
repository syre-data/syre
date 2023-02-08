//! [`ScriptAssociation`](thot_core::project::ScriptAssociation) editor
//! for a [`Container`](thot_core::project::Container).
use thot_core::project::container::ScriptMap;
use yew::prelude::*;

/// Properties for [`ScriptAssociationsEditor`].
#[derive(Properties, PartialEq)]
pub struct ScriptAssociationsEditorProps {
    #[prop_or_default]
    pub class: Classes,
    pub associations: ScriptMap,
}

/// [`ScriptAssociation`](thot_core::project::ScriptAssociation)s editor.
#[function_component(ScriptAssociationsEditor)]
pub fn script_associations_editor(props: &ScriptAssociationsEditorProps) -> Html {
    html! {
        {"scripts"}
    }
}

#[cfg(test)]
#[path = "./scripts_test.rs"]
mod scripts_test;

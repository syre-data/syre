//! Editor for [`ScriptAssociation`]s.
use std::collections::HashMap;
use std::rc::Rc;
use thot_core::project::container::ScriptMap;
use thot_core::project::RunParameters;
use thot_core::types::ResourceId;
use yew::prelude::*;

pub type NameMap = HashMap<ResourceId, String>;

// **************************
// *** Association Editor ***
// **************************

pub enum ScriptAssociationStateAction {
    SetPriority(i32),
    SetAutorun(bool),
}

#[derive(PartialEq, Clone)]
pub struct ScriptAssociationState {
    run_parameters: RunParameters,
}

impl Reducible for ScriptAssociationState {
    type Action = ScriptAssociationStateAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        let mut current = (*self).clone();
        match action {
            ScriptAssociationStateAction::SetPriority(priority) => {
                current.run_parameters.priority = priority;
            }
            ScriptAssociationStateAction::SetAutorun(autorun) => {
                current.run_parameters.autorun = autorun;
            }
        }

        current.into()
    }
}

#[derive(PartialEq, Properties)]
pub struct ScriptAssociationEditorProps {
    pub name: String,
    pub run_parameters: RunParameters,

    #[prop_or_default]
    pub onchange: Option<Callback<RunParameters>>,
}

#[function_component(ScriptAssociationEditor)]
pub fn script_association_editor(props: &ScriptAssociationEditorProps) -> Html {
    let association_state = use_reducer(|| ScriptAssociationState {
        run_parameters: props.run_parameters.clone(),
    });
    let priority_ref = use_node_ref();
    let autorun_ref = use_node_ref();

    let onchange_priority = {
        let onchange = props.onchange.clone();
        let association_state = association_state.clone();
        let priority_ref = priority_ref.clone();

        Callback::from(move |_: Event| {
            let priority_ref = priority_ref
                .cast::<web_sys::HtmlInputElement>()
                .expect("could not cast `NodeRef` into element");

            let priority = priority_ref
                .value()
                .parse()
                .expect("could not parse input as number");

            association_state.dispatch(ScriptAssociationStateAction::SetPriority(priority));
            if let Some(onchange) = onchange.as_ref() {
                onchange.emit(association_state.run_parameters.clone());
            }
        })
    };

    let onchange_autorun = {
        let onchange = props.onchange.clone();
        let association_state = association_state.clone();
        let autorun_ref = autorun_ref.clone();

        Callback::from(move |_: Event| {
            let autorun_ref = autorun_ref
                .cast::<web_sys::HtmlInputElement>()
                .expect("could not cast `NodeRef` into element");

            let autorun = autorun_ref.checked();
            association_state.dispatch(ScriptAssociationStateAction::SetAutorun(autorun));
            if let Some(onchange) = onchange.as_ref() {
                onchange.emit(association_state.run_parameters.clone());
            }
        })
    };

    html! {
        <div class={classes!("thot-ui-script-association-editor")}>
            <label class={classes!("script-association-script")}>
                { &props.name }
            </label>
            <input
                ref={priority_ref}
                class={classes!("script-association-priority")}
                type={"number"}
                value={association_state.run_parameters.priority.to_string()}
                onchange={onchange_priority} />

            <input
                ref={autorun_ref}
                class={classes!("script-association-autorun")}
                type={"checkbox"}
                checked={association_state.run_parameters.autorun}
                onchange={onchange_autorun} />
        </div>
    }
}

// ***************************
// *** Associations Editor ***
// ***************************

#[derive(PartialEq, Properties)]
pub struct ScriptAssociationsEditorProps {
    /// [`ScriptAssociation`]s to edit.
    pub associations: ScriptMap,

    /// Map of [`Script`] [`ResourceId`]s to display names.
    #[prop_or_default]
    pub name_map: Option<NameMap>,

    /// Called when the value changes.
    pub onchange: Callback<ScriptMap>,
}

#[function_component(ScriptAssociationsEditor)]
pub fn script_associations_editor(props: &ScriptAssociationsEditorProps) -> Html {
    let onchange_association = move |script: ResourceId| {
        let onchange = props.onchange.clone();
        let associations = props.associations.clone();

        Callback::from(move |run_parameters: RunParameters| {
            let mut associations = associations.clone();
            associations.insert(script.clone(), run_parameters);
            onchange.emit(associations);
        })
    };

    let remove_script = move |script: ResourceId| {
        let onchange = props.onchange.clone();
        let associations = props.associations.clone();

        Callback::from(move |_: MouseEvent| {
            let mut associations = associations.clone();
            associations.remove(&script.clone());
            onchange.emit(associations);
        })
    };

    html! {
        <form class="thot-ui-script-associations-editor">
            <ol>
                { props.associations.iter().map(|(script, run_parameters)|{
                     let script_name = script.clone().to_string();
                     let name = if let Some(name_map) = props.name_map.as_ref() {
                         name_map.get(&script).map(|name| name.clone()).unwrap_or(script_name)
                     } else {
                         script_name
                     };
                     html! {
                        <li key={script.clone()}
                            class={classes!("script-association")} >

                            <ScriptAssociationEditor
                                {name}
                                run_parameters={run_parameters.clone()}
                                onchange={onchange_association(script.clone())} />

                            <div>
                                <button
                                    type="button"
                                    onclick={remove_script(script.clone())}>

                                    { "X" }
                                </button>
                            </div>
                        </li>
                    }
                }).collect::<Html>() }
            </ol>
        </form>
    }
}

#[cfg(test)]
#[path = "./script_associations_editor_test.rs"]
mod script_associations_editor_test;

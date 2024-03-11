//! Editor for [`ScriptAssociation`]s.
use std::rc::Rc;
use syre_core::project::container::AnalysisMap;
use syre_core::project::RunParameters;
use syre_core::types::{ResourceId, ResourceMap};
use yew::prelude::*;
use yew_icons::{Icon, IconId};

pub type NameMap = ResourceMap<String>;

// **************************
// *** Association Editor ***
// **************************

pub enum AnalysisAssociationStateAction {
    SetValue(RunParameters),
    SetPriority(i32),
    SetAutorun(bool),
}

#[derive(PartialEq, Clone)]
pub struct AnalysisAssociationState {
    run_parameters: RunParameters,
}

impl Reducible for AnalysisAssociationState {
    type Action = AnalysisAssociationStateAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        let mut current = (*self).clone();
        match action {
            AnalysisAssociationStateAction::SetValue(params) => {
                current.run_parameters = params;
            }

            AnalysisAssociationStateAction::SetPriority(priority) => {
                current.run_parameters.priority = priority;
            }

            AnalysisAssociationStateAction::SetAutorun(autorun) => {
                current.run_parameters.autorun = autorun;
            }
        }

        current.into()
    }
}

#[derive(PartialEq, Properties)]
pub struct AnalysisAssociationEditorProps {
    pub name: String,
    pub run_parameters: RunParameters,

    #[prop_or_default]
    pub onchange: Callback<RunParameters>,
}

#[function_component(AnalysisAssociationEditor)]
pub fn analysis_association_editor(props: &AnalysisAssociationEditorProps) -> Html {
    let dirty_state = use_state(|| false); // track if changes are from user interaction
    let association_state = use_reducer(|| AnalysisAssociationState {
        run_parameters: props.run_parameters.clone(),
    });

    let priority_ref = use_node_ref();
    let autorun_ref = use_node_ref();

    {
        let association_state = association_state.clone();
        let dirty_state = dirty_state.clone();

        use_effect_with(props.run_parameters.clone(), move |run_parameters| {
            association_state.dispatch(AnalysisAssociationStateAction::SetValue(
                run_parameters.clone(),
            ));

            dirty_state.set(false);
        });
    }

    {
        let onchange = props.onchange.clone();
        let dirty_state = dirty_state.clone();
        let association_state = association_state.clone();

        use_effect_with(association_state, move |association_state| {
            if !*dirty_state {
                return;
            }

            onchange.emit(association_state.run_parameters.clone());
            dirty_state.set(false);
        });
    }

    let onchange_priority = {
        let dirty_state = dirty_state.clone();
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

            association_state.dispatch(AnalysisAssociationStateAction::SetPriority(priority));
            dirty_state.set(true);
        })
    };

    let onchange_autorun = {
        let dirty_state = dirty_state.clone();
        let association_state = association_state.clone();
        let autorun_ref = autorun_ref.clone();

        Callback::from(move |_: Event| {
            let autorun_ref = autorun_ref
                .cast::<web_sys::HtmlInputElement>()
                .expect("could not cast `NodeRef` into element");

            let autorun = autorun_ref.checked();
            association_state.dispatch(AnalysisAssociationStateAction::SetAutorun(autorun));
            dirty_state.set(true);
        })
    };

    html! {
        <div class={classes!("syre-ui-script-association-editor")}>
            <label class={classes!("script-association-script")}
                title={props.name.clone()}>
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
    pub associations: AnalysisMap,

    /// Map of [`Script`] [`ResourceId`]s to display names.
    #[prop_or_default]
    pub name_map: Option<NameMap>,

    /// Called when the value changes.
    pub onchange: Callback<AnalysisMap>,
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
        <form class="syre-ui-script-associations-editor">
            <ol>
                { props.associations.iter().map(|(script, run_parameters)|{
                     let script_name = script.clone().to_string();
                     let name = if let Some(name_map) = props.name_map.as_ref() {
                         name_map.get(&script).map(|name| name.clone()).unwrap_or(script_name)
                     } else {
                         script_name
                     };
                     html! {
                        <li key={script.clone()} class={"script-association"} >
                            <AnalysisAssociationEditor
                                {name}
                                run_parameters={run_parameters.clone()}
                                onchange={onchange_association(script.clone())} />

                            <button class={"add-button"} type={"button"} onclick={remove_script(script.clone())}>
                                <Icon class={"syre-ui-icon syre-ui-add-remove-icon"}
                                    icon_id={IconId::HeroiconsSolidMinus} />
                            </button>
                        </li>
                    }
                }).collect::<Html>() }
            </ol>
        </form>
    }
}

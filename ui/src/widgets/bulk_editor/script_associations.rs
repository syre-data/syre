//! Bulk editor for [`ScriptAssociation`]s.
use super::types::BulkValue;
use std::rc::Rc;
use syre_core::project::RunParameters;
use syre_core::types::{ResourceId, ResourceMap};
use yew::prelude::*;
use yew_icons::{Icon, IconId};

const PLACEHOLDER: &'static str = "(mixed)";

pub type NameMap = ResourceMap<String>;
pub type ScriptBulkMap = ResourceMap<Vec<RunParameters>>;

#[derive(PartialEq, Clone, Debug)]
pub struct RunParametersUpdate {
    pub script: ResourceId,
    pub priority: Option<i32>,
    pub autorun: Option<bool>,
}

impl RunParametersUpdate {
    pub fn new(script: ResourceId) -> Self {
        Self {
            script,
            priority: None,
            autorun: None,
        }
    }

    pub fn set_priority(&mut self, priority: i32) {
        self.priority = Some(priority);
    }

    pub fn unset_priority(&mut self) {
        self.priority = None;
    }

    pub fn set_autorun(&mut self, autorun: bool) {
        self.autorun = Some(autorun);
    }

    pub fn unset_autorun(&mut self) {
        self.autorun = None;
    }
}

// **************************
// *** Association Editor ***
// **************************

pub enum ScriptAssociationStateAction {
    SetValue(Vec<RunParameters>),
}

#[derive(PartialEq, Clone)]
pub struct ScriptAssociationState {
    priority: BulkValue<i32>,
    autorun: BulkValue<bool>,
}

impl ScriptAssociationState {
    pub fn new(params: Vec<RunParameters>) -> Self {
        assert!(params.len() > 0, "no run parameters provided");

        let mut priority = params
            .iter()
            .map(|p| p.priority.clone())
            .collect::<Vec<_>>();
        let mut autorun = params.iter().map(|p| p.autorun.clone()).collect::<Vec<_>>();
        priority.sort();
        priority.dedup();
        autorun.sort();
        autorun.dedup();

        let priority = if priority.len() == 1 {
            BulkValue::Equal(priority[0])
        } else {
            BulkValue::Mixed
        };

        let autorun = if autorun.len() == 1 {
            BulkValue::Equal(autorun[0])
        } else {
            BulkValue::Mixed
        };

        Self { priority, autorun }
    }

    pub fn priority(&self) -> &BulkValue<i32> {
        &self.priority
    }

    pub fn autorun(&self) -> &BulkValue<bool> {
        &self.autorun
    }
}

impl Reducible for ScriptAssociationState {
    type Action = ScriptAssociationStateAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        match action {
            ScriptAssociationStateAction::SetValue(params) => {
                let current = Self::new(params);
                current.into()
            }
        }
    }
}

#[derive(PartialEq, Properties)]
pub struct ScriptAssociationEditorProps {
    pub name: String,
    pub run_parameters: Vec<RunParameters>,

    #[prop_or_default]
    pub onchange_priority: Callback<i32>,

    #[prop_or_default]
    pub onchange_autorun: Callback<bool>,
}

#[function_component(ScriptAssociationEditor)]
pub fn script_association_editor(props: &ScriptAssociationEditorProps) -> Html {
    let association_state =
        use_reducer(|| ScriptAssociationState::new(props.run_parameters.clone()));

    let priority_ref = use_node_ref();
    let autorun_ref = use_node_ref();

    {
        let association_state = association_state.clone();

        use_effect_with(props.run_parameters.clone(), move |params| {
            association_state.dispatch(ScriptAssociationStateAction::SetValue(params.clone()));
        });
    }

    {
        let association_state = association_state.clone();
        let autorun_ref = autorun_ref.clone();

        use_effect_with(
            (association_state, autorun_ref),
            move |(association_state, autorun_ref)| {
                if association_state.autorun() == &BulkValue::Mixed {
                    let input = autorun_ref
                        .cast::<web_sys::HtmlInputElement>()
                        .expect("could not cast node ref to input element");

                    input.set_indeterminate(true);
                }
            },
        );
    }

    // ***********************
    // *** change handlers ***
    // ***********************

    let onchange_priority = {
        let onchange_priority = props.onchange_priority.clone();
        let priority_ref = priority_ref.clone();

        Callback::from(move |_: Event| {
            let priority_ref = priority_ref
                .cast::<web_sys::HtmlInputElement>()
                .expect("could not cast `NodeRef` into element");

            let priority = priority_ref
                .value()
                .parse()
                .expect("could not parse input as number");

            onchange_priority.emit(priority);
        })
    };

    let onchange_autorun = {
        let onchange_autorun = props.onchange_autorun.clone();
        let autorun_ref = autorun_ref.clone();

        Callback::from(move |_: Event| {
            let autorun_ref = autorun_ref
                .cast::<web_sys::HtmlInputElement>()
                .expect("could not cast `NodeRef` into element");

            onchange_autorun.emit(autorun_ref.checked())
        })
    };

    let priority = match association_state.priority() {
        BulkValue::Mixed => "".to_string(),
        BulkValue::Equal(priority) => priority.to_string(),
    };

    let placeholder_priority = match association_state.priority() {
        BulkValue::Mixed => PLACEHOLDER,
        BulkValue::Equal(_) => "",
    };

    let autorun = match association_state.autorun() {
        BulkValue::Mixed => true,
        BulkValue::Equal(autorun) => autorun.clone(),
    };

    html! {
        <div class={"syre-ui-script-association-editor"}>
            <label class={"script-association-script"}>
                { &props.name }
            </label>
            <input
                ref={priority_ref}
                class={"script-association-priority"}
                type={"number"}
                placeholder={placeholder_priority}
                value={priority}
                onchange={onchange_priority} />

            <input
                ref={autorun_ref}
                class={"script-association-autorun"}
                type={"checkbox"}
                checked={autorun}
                onchange={onchange_autorun} />
        </div>
    }
}

// ***************************
// *** Associations Editor ***
// ***************************

#[derive(PartialEq, Properties)]
pub struct ScriptAssociationsBulkEditorProps {
    /// [`ScriptAssociation`]s to edit.
    pub associations: ScriptBulkMap,

    /// Map of [`Script`] [`ResourceId`]s to display names.
    #[prop_or_default]
    pub name_map: Option<NameMap>,

    /// Called when an association is removed.
    pub onremove: Callback<ResourceId>,

    /// Called when the value of an association changes.
    pub onchange: Callback<RunParametersUpdate>,
}

#[function_component(ScriptAssociationsBulkEditor)]
pub fn script_associations_bulk_editor(props: &ScriptAssociationsBulkEditorProps) -> Html {
    let onchange_priority = move |script: ResourceId| {
        let onchange = props.onchange.clone();

        Callback::from(move |priority: i32| {
            let mut update = RunParametersUpdate::new(script.clone());
            update.set_priority(priority);
            onchange.emit(update);
        })
    };

    let onchange_autorun = move |script: ResourceId| {
        let onchange = props.onchange.clone();
        let script = script.clone();

        Callback::from(move |autorun: bool| {
            let mut update = RunParametersUpdate::new(script.clone());
            update.set_autorun(autorun);
            onchange.emit(update);
        })
    };

    let remove_script = move |script: ResourceId| {
        let onremove = props.onremove.clone();

        Callback::from(move |_: MouseEvent| {
            onremove.emit(script.clone());
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
                        <li key={script.clone()}
                            class={classes!("script-association")} >

                            <ScriptAssociationEditor
                                {name}
                                run_parameters={run_parameters.clone()}
                                onchange_priority={onchange_priority(script.clone())}
                                onchange_autorun={onchange_autorun(script.clone())} />

                            <button class={"remove-button"} type={"button"}
                                onclick={remove_script(script.clone())}>

                                <Icon class={"syre-ui-icon syre-ui-add-remove-icon"}
                                    icon_id={IconId::HeroiconsSolidMinus}/>
                            </button>
                        </li>
                    }
                }).collect::<Html>() }
            </ol>
        </form>
    }
}

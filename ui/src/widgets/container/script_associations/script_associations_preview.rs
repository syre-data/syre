//! ScriptAssociation preview for
//! [`Container`](crate::widgets::container::container_tree::Container)s in the `Container` tree.
use super::script_associations_editor::NameMap;
use crate::constants;
use syre_core::project::container::ScriptMap;
use syre_core::project::{RunParameters, ScriptAssociation};
use syre_core::types::{ResourceId, ResourceMap};
use yew::prelude::*;
use yew_icons::{Icon, IconId};

#[derive(PartialEq, Properties)]
struct ScriptAssociationPreviewProps {
    pub name: String,
    pub run_parameters: RunParameters,

    #[prop_or_default]
    pub onchange: Callback<RunParameters>,

    #[prop_or_default]
    pub onremove: Callback<()>,
}

#[function_component(ScriptAssociationPreview)]
fn script_association_preview(props: &ScriptAssociationPreviewProps) -> Html {
    let parameter_state = use_state(|| props.run_parameters.clone());
    use_effect_with(props.run_parameters.clone(), {
        let parameter_state = parameter_state.setter();
        move |run_parameters| {
            parameter_state.set(run_parameters.clone());
        }
    });

    use_effect_with(
        (parameter_state.clone(), props.onchange.clone()),
        move |(parameter_state, onchange)| {
            onchange.emit((**parameter_state).clone());
        },
    );

    let onremove = use_callback(props.onremove.clone(), move |e: MouseEvent, onremove| {
        e.stop_propagation();
        onremove.emit(());
    });

    let toggle_autorun = use_callback(
        parameter_state.clone(),
        move |e: MouseEvent, parameter_state| {
            e.stop_propagation();
            parameter_state.set(RunParameters {
                autorun: !parameter_state.autorun,
                priority: parameter_state.priority,
            });
        },
    );

    html! {
        <div class={"syre-ui-script-association-preview"}
            data-priority={props.run_parameters.priority.to_string()}
            data-autorun={props.run_parameters.autorun.to_string()}>

            <span class={"script-name"} title={props.name.clone()}>{ &props.name }</span>
            <span class={"script-priority"}>{ props.run_parameters.priority }</span>
            <span class={"script-autorun"}
                onclick={toggle_autorun}>

                if props.run_parameters.autorun {
                    <Icon icon_id={IconId::FontAwesomeSolidStar}
                        class={"syre-ui-icon"} />
                } else {
                    <Icon icon_id={IconId::FontAwesomeRegularStar}
                        class={"syre-ui-icon"} />
                }
            </span>
            <button class={"syre-ui-remove-resource btn-icon"}
                type={"button"}
                onclick={onremove}>

                <Icon icon_id={IconId::HeroiconsSolidMinus}
                    class={"syre-ui-icon syre-ui-add-remove-icon"} />
            </button>
        </div>
    }
}

#[derive(PartialEq, Properties)]
pub struct ScriptAssociationsPreviewProps {
    pub scripts: ScriptMap,
    pub names: ResourceMap<String>,

    /// Map from `Script` id to name.
    #[prop_or_default]
    pub name_map: Option<NameMap>,

    /// Callback run when a script association is modified.
    #[prop_or_default]
    pub onchange: Callback<ScriptAssociation>,

    /// Callback when an association is requesting removal.
    #[prop_or_default]
    pub onremove: Callback<ResourceId>,
}

#[function_component(ScriptAssociationsPreview)]
pub fn script_associations_preview(props: &ScriptAssociationsPreviewProps) -> Html {
    let mut scripts = props.scripts.iter().collect::<Vec<_>>();
    scripts.sort_by(
        |(_, RunParameters { priority: p1, .. }), (_, RunParameters { priority: p2, .. })| {
            p2.cmp(p1)
        },
    );

    let onchange = move |script: ResourceId| {
        let onchange = props.onchange.clone();
        let script = script.clone();
        Callback::from(move |params| {
            onchange.emit(ScriptAssociation::new_with_params(script.clone(), params));
        })
    };

    let onremove = move |script: ResourceId| {
        let onremove = props.onremove.clone();
        let script = script.clone();
        Callback::from(move |_| {
            onremove.emit(script.clone());
        })
    };

    html! {
        <div class={classes!("syre-ui-script-associations-preview")}>
            if props.scripts.len() == 0 {
                { "(no scripts)" }
            } else {
                <ol class={classes!("syre-ui-script-associations-list")}>
                    { scripts.into_iter().map(|(script, run_parameters)| {
                        let name = props.names.get(script).expect("script name not found.");
                        html! {
                            <li>
                                <ScriptAssociationPreview
                                    name={name.clone()}
                                    run_parameters={run_parameters.clone()}
                                    onchange={onchange(script.clone())}
                                    onremove={onremove(script.clone())}/>
                            </li>
                        }
                    }).collect::<Html>() }
                </ol>
            }
        </div>
    }
}

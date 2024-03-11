//! Analysis associationpreview for
//! [`Container`](crate::widgets::container::container_tree::Container)s in the `Container` tree.
use super::associations_editor::NameMap;
use syre_core::project::container::AnalysisMap;
use syre_core::project::{AnalysisAssociation, RunParameters};
use syre_core::types::{ResourceId, ResourceMap};
use yew::prelude::*;
use yew_icons::{Icon, IconId};

#[derive(PartialEq, Properties)]
struct AnalysisAssociationPreviewProps {
    pub name: String,
    pub run_parameters: RunParameters,

    #[prop_or_default]
    pub class: Classes,

    #[prop_or_default]
    pub onchange: Callback<RunParameters>,

    #[prop_or_default]
    pub onremove: Callback<()>,
}

#[function_component(AnalysisAssociationPreview)]
fn analysis_association_preview(props: &AnalysisAssociationPreviewProps) -> Html {
    let parameter_state = use_state(|| props.run_parameters.clone());
    use_effect_with(props.run_parameters.clone(), {
        let parameter_state = parameter_state.setter();
        move |run_parameters| {
            parameter_state.set(run_parameters.clone());
        }
    });

    let onremove = use_callback(props.onremove.clone(), move |e: MouseEvent, onremove| {
        e.stop_propagation();
        onremove.emit(());
    });

    let toggle_autorun = use_callback(
        (props.onchange.clone(), parameter_state.clone()),
        move |e: MouseEvent, (onchange, parameter_state)| {
            e.stop_propagation();

            let params = RunParameters {
                autorun: !parameter_state.autorun,
                priority: parameter_state.priority,
            };

            onchange.emit(params);
        },
    );

    let class = classes!("syre-ui-analysis-association-preview", props.class.clone());
    html! {
        <div {class}
            data-priority={props.run_parameters.priority.to_string()}
            data-autorun={props.run_parameters.autorun.to_string()}>

            <span class={"analysis-name"} title={props.name.clone()}>{ &props.name }</span>
            <span class={"analysis-priority"}>{ props.run_parameters.priority }</span>
            <span class={"analysis-autorun"}
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
pub struct AnalysisAssociationsPreviewProps {
    pub analyses: AnalysisMap,
    pub names: ResourceMap<String>,

    /// Map from analysis id to name.
    #[prop_or_default]
    pub name_map: Option<NameMap>,

    /// Callback run when an analysis association is modified.
    #[prop_or_default]
    pub onchange: Callback<AnalysisAssociation>,

    /// Callback when an association is requesting removal.
    #[prop_or_default]
    pub onremove: Callback<ResourceId>,
}

#[function_component(AnalysisAssociationsPreview)]
pub fn analysis_associations_preview(props: &AnalysisAssociationsPreviewProps) -> Html {
    let mut analyses = props.analyses.iter().collect::<Vec<_>>();
    analyses.sort_by(
        |(_, RunParameters { priority: p1, .. }), (_, RunParameters { priority: p2, .. })| {
            p2.cmp(p1)
        },
    );

    let onchange = move |analysis: ResourceId| {
        let onchange = props.onchange.clone();
        let analysis = analysis.clone();
        move |params| {
            onchange.emit(AnalysisAssociation::new_with_params(
                analysis.clone(),
                params,
            ));
        }
    };

    let onremove = move |analysis: ResourceId| {
        let onremove = props.onremove.clone();
        let analysis = analysis.clone();
        Callback::from(move |_| {
            onremove.emit(analysis.clone());
        })
    };

    html! {
        <div class={classes!("syre-ui-analysis-associations-preview")}>
            if props.analyses.len() == 0 {
                { "(no analyses)" }
            } else {
                <ol class={classes!("syre-ui-analysis-associations-list")}>
                    { analyses.into_iter().map(|(analysis, run_parameters)| {
                        let mut class = classes!();
                        let name = props
                            .names
                            .get(analysis)
                            .map(|name| name.clone());

                        let name = if let Some(name) = name {
                            name
                        } else {
                            class.push("no-name");
                            analysis.to_string()
                        };

                        html! {
                            <li>
                                <AnalysisAssociationPreview
                                    {class}
                                    {name}
                                    run_parameters={run_parameters.clone()}
                                    onchange={onchange(analysis.clone())}
                                    onremove={onremove(analysis.clone())}/>
                            </li>
                        }
                    }).collect::<Html>() }
                </ol>
            }
        </div>
    }
}

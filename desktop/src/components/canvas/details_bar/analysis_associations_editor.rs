//! Edit a [`Container`]'s [`AnalysisAssociation`]s.
use crate::app::ProjectsStateReducer;
use crate::commands::container::{update_analysis_associations, UpdateAnalysisAssociationsArgs};
use crate::components::canvas::{CanvasStateReducer, GraphStateAction, GraphStateReducer};
use crate::lib::DisplayName;
use syre_core::project::container::AnalysisMap;
use syre_core::project::RunParameters;
use syre_core::types::ResourceId;
use syre_local::types::analysis::{AnalysisKind, Store as AnalysisStore};
use syre_ui::widgets::container::analysis_associations::{
    AddAnalysisAssociation, NameMap, ScriptAssociationsEditor as ContainerAssociationsEditor,
};
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

#[derive(Properties, PartialEq, Debug)]
pub struct AnalysisAssociationsEditorProps {
    pub container: ResourceId,

    /// Called after save.
    #[prop_or_default]
    pub onsave: Option<Callback<()>>,
}

#[tracing::instrument]
#[function_component(AnalysisAssociationsEditor)]
pub fn analysis_associations_editor(props: &AnalysisAssociationsEditorProps) -> HtmlResult {
    let projects_state = use_context::<ProjectsStateReducer>().unwrap();
    let canvas_state = use_context::<CanvasStateReducer>().unwrap();
    let graph_state = use_context::<GraphStateReducer>().unwrap();

    let container = graph_state
        .graph
        .get(&props.container)
        .expect("`Container not found");

    let dirty_state = use_state(|| false); // track if changes come from user or are internal
    let associations = use_state(|| container.analyses.clone());

    let project_analyses = use_state(|| {
        projects_state
            .project_analyses
            .get(&canvas_state.project)
            .expect("`Project`'s analyses not loaded")
            .clone()
    });

    let remaining_analyses = use_state(|| {
        project_analyses
            .keys()
            .into_iter()
            .filter_map(|analysis| {
                if associations.contains_key(&analysis) {
                    None
                } else {
                    Some((
                        analysis.clone(),
                        get_analysis_name(&project_analyses, analysis),
                    ))
                }
            })
            .collect::<Vec<_>>()
    });

    use_effect_with(projects_state.clone(), {
        let project_analyses = project_analyses.setter();
        move |projects_state| {
            project_analyses.set(
                projects_state
                    .project_analyses
                    .get(&canvas_state.project)
                    .expect("`Project`'s `Scripts` not loaded")
                    .clone(),
            );
        }
    });

    use_effect_with((project_analyses.clone(), associations.clone()), {
        let remaining_analyses = remaining_analyses.setter();
        move |(project_analyses, associations)| {
            remaining_analyses.set(
                project_analyses
                    .keys()
                    .into_iter()
                    .filter_map(|analysis| {
                        if associations.contains_key(analysis) {
                            None
                        } else {
                            Some((
                                analysis.clone(),
                                get_analysis_name(&project_analyses, analysis),
                            ))
                        }
                    })
                    .collect::<Vec<_>>(),
            );
        }
    });

    // Update associations based on container
    use_effect_with(container.clone(), {
        let dirty_state = dirty_state.setter();
        let associations = associations.setter();

        move |container| {
            associations.set(container.analyses.clone());
            dirty_state.set(false);
        }
    });

    // Update remaining analyses based on associations
    use_effect_with((associations.clone(), project_analyses.clone()), {
        let remaining_analyses = remaining_analyses.setter();
        move |(associations, project_analyses)| {
            let analyses = project_analyses
                .keys()
                .into_iter()
                .filter_map(|analysis| {
                    if associations.contains_key(analysis) {
                        None
                    } else {
                        Some((
                            analysis.clone(),
                            get_analysis_name(&project_analyses, analysis),
                        ))
                    }
                })
                .collect::<Vec<_>>();

            remaining_analyses.set(analyses);
        }
    });

    // Save associations on change
    use_effect_with(
        (
            props.container.clone(),
            associations.clone(),
            dirty_state.clone(),
        ),
        {
            let graph_state = graph_state.dispatcher();
            move |(container, associations, dirty_state)| {
                if !**dirty_state {
                    return;
                }

                let container = container.clone();
                let graph_state = graph_state.clone();
                let associations = associations.clone();

                spawn_local(async move {
                    match update_analysis_associations(container.clone(), (*associations).clone())
                        .await
                    {
                        Ok(_) => {}
                        Err(err) => {
                            tracing::debug!(?err);
                            panic!("{err:?}");
                        }
                    }

                    let update = UpdateAnalysisAssociationsArgs {
                        rid: container,
                        associations: (*associations).clone(),
                    };

                    graph_state.dispatch(GraphStateAction::UpdateContainerAnalysisAssociations(
                        update,
                    ));
                });
            }
        },
    );

    let name_map = (*associations)
        .clone()
        .into_keys()
        .filter_map(|assoc| {
            project_analyses
                .get(&assoc)
                .map(|analysis| (assoc, analysis.display_name()))
        })
        .collect::<NameMap>();

    let onadd = {
        let dirty_state = dirty_state.clone();
        let associations = associations.clone();

        Callback::from(move |analysis: ResourceId| {
            let mut assocs = (*associations).clone();
            assocs.insert(analysis, RunParameters::new());
            associations.set(assocs);
            dirty_state.set(true);
        })
    };

    let onchange = {
        let dirty_state = dirty_state.clone();
        let associations = associations.clone();

        Callback::from(move |assocs: AnalysisMap| {
            associations.set(assocs);
            dirty_state.set(true);
        })
    };

    Ok(html! {
        <div class={classes!("analysis-associations-editor-widget")}>
            <AddAnalysisAssociation
                scripts={(*remaining_analyses).clone()}
                {onadd} />

            <ContainerAssociationsEditor
                associations={(*associations).clone()}
                {name_map}
                {onchange} />
        </div>
    })
}

fn get_analysis_name(analysis_store: &AnalysisStore, analysis: &ResourceId) -> String {
    match analysis_store.get(&analysis).unwrap() {
        AnalysisKind::Script(script) => script
            .name
            .clone()
            .unwrap_or(script.path.to_string_lossy().to_string()),

        AnalysisKind::ExcelTemplate(template) => template
            .name
            .clone()
            .unwrap_or(template.template.path.to_string_lossy().to_string()),
    }
}

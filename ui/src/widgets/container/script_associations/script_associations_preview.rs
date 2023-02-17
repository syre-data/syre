//! ScriptAssociation preview for
//! [`Container`](crate::widgets::container::container_tree::Container)s in the `Container` tree.
use super::script_associations_editor::NameMap;
use thot_core::project::container::ScriptMap;
use thot_core::project::RunParameters;
use thot_core::types::ResourceMap;
use yew::prelude::*;

#[derive(PartialEq, Properties)]
struct ScriptAssociationPreviewProps {
    pub name: String,
    pub run_parameters: RunParameters,
}

#[function_component(ScriptAssociationPreview)]
fn script_association_preview(props: &ScriptAssociationPreviewProps) -> Html {
    // @todo: Add `RunParameters` functionality.
    html! {
        <>
            <span>{ &props.name }</span>
            <span>{ props.run_parameters.priority }</span>
            <span>{ props.run_parameters.autorun }</span>
        </>
    }
}

#[derive(PartialEq, Properties)]
pub struct ScriptAssociationsPreviewProps {
    pub scripts: ScriptMap,
    pub names: ResourceMap<String>,

    /// MAp from `Script` id to name.
    #[prop_or_default]
    pub name_map: Option<NameMap>,
}

#[function_component(ScriptAssociationsPreview)]
pub fn script_associations_preview(props: &ScriptAssociationsPreviewProps) -> Html {
    html! {
        <div class={classes!("thot-ui-script-associations-preview")}>
            if props.scripts.len() == 0 {
                { "(no scripts)" }
            } else {
                <ol class={classes!("thot-ui-script-associations-list")}>
                    { props.scripts.iter().map(|(script, run_parameters)| {
                        let name = props.names.get(script).expect("script name not found.");
                        html! {
                            <li>
                                <ScriptAssociationPreview name={name.clone()} run_parameters={run_parameters.clone()} />
                            </li>
                        }
                    }).collect::<Html>() }
                </ol>
            }
        </div>
    }
}

#[cfg(test)]
#[path = "./script_associations_preview_test.rs"]
mod script_associations_preview_test;

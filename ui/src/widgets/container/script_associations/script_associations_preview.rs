//! ScriptAssociation preview for
//! [`Container`](crate::widgets::container::container_tree::Container)s in the `Container` tree.
use super::script_associations_editor::NameMap;
use thot_core::project::container::ScriptMap;
use yew::prelude::*;

#[derive(PartialEq, Properties)]
pub struct ScriptAssociationsPreviewProps {
    pub scripts: ScriptMap,

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
                    { props.scripts.iter().map(|(script, run_parameters)| { html! {
                        // @todo: Use `Script` names if available.
                        // @todo: Add `RunParameters` functionality.
                        <li>
                            { script.to_string() }
                        </li>
                    }}).collect::<Html>() }
                </ol>
            }
        </div>
    }
}

#[cfg(test)]
#[path = "./script_associations_preview_test.rs"]
mod script_associations_preview_test;

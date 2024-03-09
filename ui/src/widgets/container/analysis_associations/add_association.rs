//! Add a [`ScriptAssociation`] to a [`Container`].
use std::str::FromStr;
use syre_core::types::ResourceId;
use yew::prelude::*;
use yew_icons::{Icon, IconId};

#[derive(Properties, PartialEq)]
pub struct AddScriptAssociationProps {
    /// Available `Script`s.
    ///
    /// # Fields
    /// 1. Id
    /// 2. Name
    pub scripts: Vec<(ResourceId, String)>, // TODO Use indexmap::IndexSet.
    pub onadd: Callback<ResourceId>,
}

#[function_component(AddAnalysisAssociation)]
pub fn add_analysis_association(props: &AddScriptAssociationProps) -> Html {
    let active = use_state(|| false);
    let analysis_ref = use_node_ref();

    let set_active = {
        let active = active.clone();
        move |is_active: bool| {
            let active = active.clone();

            Callback::from(move |_: MouseEvent| {
                active.set(is_active);
            })
        }
    };

    let add_association = {
        let onadd = props.onadd.clone();
        let active = active.clone();
        let analysis_ref = analysis_ref.clone();

        Callback::from(move |_: MouseEvent| {
            let analysis_elm = analysis_ref
                .cast::<web_sys::HtmlSelectElement>()
                .expect("could not cast `NodeRef` to `HtmlSelectElement`");

            let analysis = analysis_elm.value();
            let analysis =
                ResourceId::from_str(analysis.as_str()).expect("could not parse to `ResoruceId`");

            onadd.emit(analysis);
            active.set(false);
        })
    };

    html! {
        <>
            <div class={"analysis-association-header"}>
                <h3>
                    { "Analyses" }
                </h3>
                <button class={"add-button"} type={"button"} onclick={set_active(true)}>
                    <Icon class={"syre-ui-icon syre-ui-add-remove-icon"}
                        icon_id={IconId::HeroiconsSolidPlus }/>
                </button>
            </div>
            if *active {
                <div>
                    <select ref={analysis_ref}>
                        { props.scripts.iter().map(|(rid, name)| {
                            html! {
                                <option value={rid.clone()}>{ &name }</option>
                            }
                        }).collect::<Html>() }
                    </select>
                    <div class={"analysis-add-cancel-buttons"}>
                        <button onclick={add_association}>{ "Add" }</button>
                        <button onclick={set_active(false)}>{ "Cancel" }</button>
                    </div>
                </div>
            }
        </>
    }
}

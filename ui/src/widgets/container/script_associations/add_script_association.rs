//! Add a [`ScriptAssociation`] to a [`Container`].
use std::str::FromStr;
use thot_core::project::Script as CoreScript;
use thot_core::types::ResourceId;
use yew::prelude::*;
use yew_icons::{Icon, IconId};

#[derive(Properties, PartialEq)]
pub struct AddScriptAssociationProps {
    /// Available `Script`s.
    pub scripts: Vec<CoreScript>, // TODO Use indexmap::IndexSet.
    pub onadd: Callback<ResourceId>,
}

#[function_component(AddScriptAssociation)]
pub fn add_script_association(props: &AddScriptAssociationProps) -> Html {
    let active = use_state(|| false);
    let script_ref = use_node_ref();

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
        let script_ref = script_ref.clone();

        Callback::from(move |_: MouseEvent| {
            let script_elm = script_ref
                .cast::<web_sys::HtmlSelectElement>()
                .expect("could not cast `NodeRef` to `HtmlSelectElement`");

            let script = script_elm.value();
            let script =
                ResourceId::from_str(script.as_str()).expect("could not parse to `ResoruceId`");

            onadd.emit(script);
            active.set(false);
        })
    };

    html! {
        <>
            <div class={classes!("script-association-header")}>
                <h3>
                    { "Scripts" }
                </h3>
                <button classes={ "add-button" } type="button" onclick={set_active(true)}>
                    <Icon class={ classes!("thot-ui-add-remove-icon")} icon_id={ IconId::HeroiconsSolidPlus }/>
                </button>
            </div>
            if *active {
                <div>
                    <select ref={script_ref}>
                        { props.scripts.iter().map(|script| {
                            let name = match script.name.clone() {
                                Some(name) => name,
                                None => script.path.as_path().to_str().expect("could not convrt `path` to `str`").to_string()
                            };

                            html! {
                                <option value={script.rid.clone()}>{ &name }</option>
                            }
                        }).collect::<Html>() }
                    </select>
                    <div class={classes!("script-add-cancel-buttons")}>
                        <button onclick={add_association}>{ "Add" }</button>
                        <button onclick={set_active(false)}>{ "Cancel" }</button>
                    </div>
                </div>
            }
        </>
    }
}

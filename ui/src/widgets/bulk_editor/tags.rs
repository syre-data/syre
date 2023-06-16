//! Tags bulk editor.
use yew::prelude::*;
use yew_icons::{Icon, IconId};

#[derive(PartialEq, Properties)]
pub struct TagsBulkEditorProps {
    pub value: Vec<String>,

    #[prop_or_default]
    pub onadd: Option<Callback<String>>,

    #[prop_or_default]
    pub onremove: Option<Callback<String>>,
}

#[function_component(TagsBulkEditor)]
pub fn tags_bulk_editor(props: &TagsBulkEditorProps) -> Html {
    let input_ref = use_node_ref();
    let onadd = match props.onadd.as_ref() {
        None => None,
        Some(onadd) => {
            let onadd = onadd.clone();
            let input_ref = input_ref.clone();
            Some(Callback::from(move |_: MouseEvent| {
                let elm = input_ref
                    .cast::<web_sys::HtmlInputElement>()
                    .expect("could not cast `NodeRef` into element");

                let value = elm.value().trim().to_string();
                if value.is_empty() {
                    return;
                }

                onadd.emit(value);
                elm.set_value("");
            }))
        }
    };

    let onremove = {
        let Some(onremove) = props.onremove.as_ref() else {
            panic!("`onremove` not provided");
        };

        move |tag: String| -> Callback<MouseEvent> {
            let onremove = onremove.clone();
            Callback::from(move |e: MouseEvent| {
                e.stop_propagation();
                onremove.emit(tag.clone());
            })
        }
    };

    html! {
        <div class={classes!("thot-ui-bulk-tag-editor")}>
            if let Some(onadd) = onadd {
                <div>
                    <input ref={input_ref}/>
                    <button class={classes!("add-button")} type="button" onclick={onadd}>
                        <Icon class={classes!("thot-ui-add-remove-icon")} icon_id={ IconId::HeroiconsSolidPlus }/>
                    </button>
                </div>
            }
            <div>
                <ul>
                    {props.value.iter().map(|tag| html! {
                        <li>
                            <span>{ tag }</span>
                            if props.onremove.is_some() {
                                <button class={classes!("remove-button")} type="button" onclick={onremove(tag.into())}>
                                    <Icon class={classes!("thot-ui-add-remove-icon")} icon_id={IconId::HeroiconsSolidMinus}/>
                                </button>
                            }
                        </li>
                    }).collect::<Html>()}
                </ul>
            </div>
        </div>
    }
}

#[cfg(test)]
#[path = "./tags_test.rs"]
mod tags_test;

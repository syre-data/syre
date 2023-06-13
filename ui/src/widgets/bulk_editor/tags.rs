//! Tags bulk editor.
use yew::prelude::*;

#[derive(PartialEq, Properties)]
pub struct TagsBulkEditorProps {
    pub tags: Vec<String>,

    #[prop_or_default]
    pub onadd: Option<Callback<String>>,

    #[prop_or_default]
    pub onremove: Option<Callback<String>>,
}

#[function_component(TagsBulkEditor)]
pub fn tags_bulk_editor(props: &TagsBulkEditorProps) -> Html {
    let onadd = match props.onadd.as_ref() {
        None => None,
        Some(onadd) => {
            let onadd = onadd.clone();
            Some(Callback::from(move |_: MouseEvent| {
                onadd.emit("new tag".into());
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
        <div>
            <div>
                <input />
                if let Some(onadd) = onadd {
                    <button onclick={onadd}>{ "+" }</button>
                }
            </div>
            <div>
                <ul>
                    {props.tags.iter().map(|tag| html! {
                        <li>
                            <span>{ tag }</span>
                            if props.onremove.is_some() {
                                <button onclick={onremove(tag.into())}>{ "-" }</button>
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

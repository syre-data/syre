//! Inline editor for tags.
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct TagsEditorProps {
    /// Tags
    #[prop_or(Vec::new())]
    pub value: Vec<String>,

    /// Callback when value is changed.
    #[prop_or_default]
    pub onchange: Callback<Vec<String>>,
}

#[function_component(TagsEditor)]
pub fn tags_editor(props: &TagsEditorProps) -> Html {
    let input_ref = use_node_ref();

    let onchange = use_callback(props.onchange.clone(), {
        let input_ref = input_ref.clone();
        move |_: Event, onchange| {
            let input = input_ref
                .cast::<web_sys::HtmlInputElement>()
                .expect("could not cast node ref into input");

            let value = input
                .value()
                .split(",")
                .into_iter()
                .filter_map(|t| {
                    let t = t.trim().to_string();
                    if t.is_empty() {
                        None
                    } else {
                        Some(t)
                    }
                })
                .collect::<Vec<String>>();

            onchange.emit(value);
        }
    });

    html! {
        <input
            ref={input_ref}
            placeholder={"(no tags)"}
            value={props.value.clone().join(", ")}
            {onchange} />
    }
}

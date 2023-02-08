//! Inline editor for tags.
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct TagsEditorProps {
    /// Displayed when inactive with no value.
    #[prop_or_default]
    pub children: Children,

    /// Tags
    #[prop_or(Vec::new())]
    pub value: Vec<String>,

    /// Callback when value is changed.
    #[prop_or_default]
    pub onchange: Option<Callback<Vec<String>>>,
}

#[function_component(TagsEditor)]
pub fn tags_editor(props: &TagsEditorProps) -> Html {
    let active = use_state(|| false);
    let input_ref = use_node_ref();

    let activate = {
        let active = active.clone();

        Callback::from(move |_: MouseEvent| {
            active.set(true);
        })
    };

    let onsave = {
        let active = active.clone();
        let input_ref = input_ref.clone();
        let onchange = props.onchange.clone();

        Callback::from(move |_e: MouseEvent| {
            active.set(false);

            if let Some(onchange) = onchange.clone() {
                let input = input_ref
                    .cast::<web_sys::HtmlInputElement>()
                    .expect("could not cast node ref into input");

                let value = input
                    .value()
                    .split(',')
                    .map(|tag| tag.trim().to_string())
                    .collect();

                onchange.emit(value);
            }
        })
    };

    let oncancel = {
        let active = active.clone();

        Callback::from(move |_: MouseEvent| {
            active.set(false);
        })
    };

    html! {
        <div ondblclick={activate}>
            if *active {
                <input ref={input_ref} value={props.value.join(", ")} />
                <button onclick={onsave}>{ "Ok" }</button>
                <button onclick={oncancel}>{ "Cancel" }</button>
            } else if !props.value.is_empty() {
                { props.value.iter().map(|tag| html!{ <span class={classes!("tag")}>{ tag }</span> }).collect::<Html>() }
            } else if !props.children.is_empty() {
                { for props.children.iter() }
            } else {
                { "(no tags)" }
            }
        </div>
    }
}

#[cfg(test)]
#[path = "./tags_editor_test.rs"]
mod tags_editor_test;

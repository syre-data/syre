//! Inline `textarea`.
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct InlineTextareaProps<'a> {
    /// Displayed when empty.
    #[prop_or_default]
    pub children: Children,

    #[prop_or_default]
    pub value: Option<String>,

    #[prop_or_default]
    pub placeholder: Option<&'a str>,

    /// Maximum length of the preview,
    /// shown when inactive, or
    /// `None` for no max.
    #[prop_or_default]
    pub max_preview: Option<usize>,

    #[prop_or_default]
    pub onchange: Option<Callback<String>>,
}

#[function_component(InlineTextarea)]
pub fn inline_textarea(props: &InlineTextareaProps<'static>) -> Html {
    let active = use_state(|| false);
    let input_ref = use_node_ref();

    let activate = {
        let active = active.clone();
        let input_ref = input_ref.clone();

        Callback::from(move |_: web_sys::MouseEvent| {
            active.set(true);

            if let Some(input) = input_ref.cast::<web_sys::HtmlTextAreaElement>() {
                input.focus().expect("could not focus textarea");
            }
        })
    };

    let onsave = {
        let active = active.clone();
        let input_ref = input_ref.clone();
        let onchange = props.onchange.clone();

        Callback::from(move |e: MouseEvent| {
            active.set(false);

            if let Some(onchange) = onchange.clone() {
                e.stop_propagation();

                let input = input_ref
                    .cast::<web_sys::HtmlTextAreaElement>()
                    .expect("could not cast node ref into textarea");

                onchange.emit(input.value());
            }
        })
    };

    let oncancel = {
        let active = active.clone();

        Callback::from(move |e: MouseEvent| {
            e.stop_propagation();
            active.set(false);
        })
    };

    let mut class = classes!("inline-form-element", "inline-textarea");
    if *active {
        class.push("active");
    } else if props.value.is_some() {
        class.push("preview");
    } else if !props.children.is_empty() {
        class.push("empty");
    } else {
        class.push("not-set");
    }

    html! {
        <div {class} ondblclick={activate}>
            if *active {
                <textarea ref={input_ref} placeholder={props.placeholder} value={props.value.clone()}></textarea>

                <div class={classes!("inline-form-element-controls")}>
                    <button onclick={onsave}>{ "Ok" }</button>
                    <button onclick={oncancel}>{ "Cancel" }</button>
                </div>
            } else if let Some(preview) = &props.value {
                if let Some(max) = props.max_preview {
                    { &preview[0..max] }
                } else {
                    { &preview }
                }
            } else if !props.children.is_empty() {
                { for props.children.iter() }
            } else {
                { "(not set)" }
            }
        </div>
    }
}

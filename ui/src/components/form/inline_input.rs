//! Form element that is enabled on a double click.
use std::fmt::Display;
use yew::html::IntoPropValue;
use yew::prelude::*;
use yew::virtual_dom::AttrValue;

#[derive(Properties, PartialEq)]
pub struct InlineInputProps<'a, T>
where
    T: PartialEq + IntoPropValue<Option<AttrValue>>,
{
    /// Displayed if inactive with no value.
    #[prop_or_default]
    pub children: Children,

    /// Placeholder text.
    #[prop_or_default]
    pub placeholder: Option<&'a str>,

    /// Type of the input element.
    #[prop_or("text")]
    pub r#type: &'a str,

    /// Data model.
    #[prop_or_default]
    pub value: Option<T>,

    /// Callback to trigger on value change.
    #[prop_or_default]
    pub onchange: Option<Callback<T>>,
}

#[function_component(InlineInput)]
pub fn inline_input<T>(props: &InlineInputProps<'static, T>) -> Html
where
    T: PartialEq + Display + From<String> + IntoPropValue<Option<AttrValue>> + Clone + 'static,
{
    let active = use_state(|| false);
    let input_ref = use_node_ref();

    {
        let active = active.clone();
        let input_ref = input_ref.clone();

        use_effect_with_deps(
            move |_active| {
                if let Some(input) = input_ref.cast::<web_sys::HtmlInputElement>() {
                    input.focus().expect("could not focus input");
                }
            },
            active,
        );
    }

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

        Callback::from(move |_: MouseEvent| {
            active.set(false);

            if let Some(onchange) = onchange.clone() {
                let input = input_ref
                    .cast::<web_sys::HtmlInputElement>()
                    .expect("could not `NodeRef` to `HtmlInputElement`.");

                let value = Into::<T>::into(input.value());
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

    let value: Option<AttrValue> = match &props.value {
        None => None,
        Some(v) => v.clone().into_prop_value(),
    };

    let mut class = classes!("inline-form-element", "inline-input");
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
                <input ref={input_ref} type={props.r#type} placeholder={props.placeholder} {value} />

                <div class={classes!("inline-form-element-controls")}>
                    <button onclick={onsave}>{ "Ok" }</button>
                    <button onclick={oncancel}>{ "Cancel" }</button>
                </div>
            } else if let Some(val) = &props.value {
                { val }
            } else if !props.children.is_empty() {
                { for props.children.iter() }
            } else {
                { "(not set)" }
            }
       </div>
    }
}

#[cfg(test)]
#[path = "./inline_input_test.rs"]
mod inline_input_test;

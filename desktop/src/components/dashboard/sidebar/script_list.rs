//! Script list.
use yew::prelude::*;

#[function_component(ScriptList)]
pub fn script_list() -> Html {
    html! {
        { "Scripts" }
    }
}

#[cfg(test)]
#[path = "./script_list_test.rs"]
mod script_list_test;

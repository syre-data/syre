//! Excel template builder.
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct ExcelTemplateProps {}

#[function_component(ExcelTemplate)]
pub fn excel_template(props: &ExcelTemplateProps) -> Html {
    html! {
        { "Excel Template Builder"}
    }
}

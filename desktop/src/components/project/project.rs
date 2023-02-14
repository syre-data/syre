//! Project component with suspense.
use super::set_data_root::SetDataRoot;
use crate::components::container::ContainerTreeController;
use crate::hooks::use_project;
use thot_core::types::ResourceId;
use thot_ui::components::ShadowBox;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct ProjectProps {
    pub rid: ResourceId,
}

#[function_component(Project)]
pub fn project(props: &ProjectProps) -> HtmlResult {
    let project = use_project(&props.rid);
    let Some(project) = project.as_ref() else {
        panic!("`Project` not loaded");
    };

    let select_data_root_visible = use_state(|| false);
    let show_select_data_root = {
        let select_data_root_visible = select_data_root_visible.clone();

        move |visible: bool| {
            let select_data_root_visible = select_data_root_visible.clone();

            Callback::from(move |_: MouseEvent| {
                select_data_root_visible.set(visible);
            })
        }
    };

    let hide_select_data_root = {
        let select_data_root_visible = select_data_root_visible.clone();

        Callback::from(move |_: ()| {
            select_data_root_visible.set(false);
        })
    };

    Ok(html! {
        <>
        <div class={classes!("project")}>
            <div class={classes!("header")}>
                <h1 class={classes!("title", "inline-block")}>{
                    &project.name
                }</h1>
                <span>{ "\u{2699}" }</span>
            </div>
            <div class={classes!("content")}>
                if let Some(root) = project.data_root.clone() {
                    <ContainerTreeController {root} />
                } else {
                    <div class={classes!("align-center")}>
                        <h2>{ "Data container not set" }</h2>
                        <button onclick={show_select_data_root(true)}>{ "Set" }</button>
                    </div>
                }
            </div>
        </div>

        // @todo: Make portal.
        if *select_data_root_visible {
            <ShadowBox title={"Set data root"} onclose={show_select_data_root(false)}>
                <SetDataRoot
                    project={project.rid.clone()}
                    onsuccess={hide_select_data_root.clone()}
                    oncancel={hide_select_data_root} />
            </ShadowBox>
        }
        </>
    })
}

#[cfg(test)]
#[path = "./project_test.rs"]
mod project_test;

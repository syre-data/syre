//! Project component with suspense.
use super::set_data_root::SetDataRoot;
use crate::components::canvas::{CanvasStateAction, CanvasStateReducer};
use crate::components::container::ContainerTreeController;
use crate::hooks::use_project;
use thot_core::types::ResourceId;
use thot_ui::components::ShadowBox;
use wasm_bindgen::JsCast;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct ProjectProps {
    pub rid: ResourceId,
}

#[function_component(Project)]
pub fn project(props: &ProjectProps) -> HtmlResult {
    let canvas_state =
        use_context::<CanvasStateReducer>().expect("`CanvasStateReducer` context not found");

    let project_ref = use_node_ref();
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

    let clear_selection = {
        let canvas_state = canvas_state.clone();
        let project_ref = project_ref.clone();

        Callback::from(move |e: MouseEvent| {
            let project_elm = project_ref
                .cast::<web_sys::HtmlElement>()
                .expect("could not cast node to element");

            let Some(target) = e.target() else {
                return;
            };

            let target = target
                .dyn_ref::<web_sys::HtmlElement>()
                .expect("could not cast target to element");

            canvas_state.dispatch(CanvasStateAction::ClearSelected);
        })
    };

    Ok(html! {
        <>
        <div ref={project_ref}
            class={classes!("project")}
            onclick={clear_selection} >

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

//! Gets a `Project`'s `Script`s.
use crate::app::ProjectsStateReducer;
use thot_core::project::Scripts as ProjectScripts;
use thot_core::types::ResourceId;
use yew::prelude::*;

#[hook]
pub fn use_project_scripts(project: ResourceId) -> UseStateHandle<Option<ProjectScripts>> {
    let projects_state =
        use_context::<ProjectsStateReducer>().expect("`ProjectsStateReducer` context not found");

    let project_scripts = use_state(|| projects_state.project_scripts.get(&project).cloned());

    {
        let project = project.clone();
        let projects_state = projects_state.clone();
        let project_scripts = project_scripts.clone();

        use_effect_with_deps(
            move |projects_state| {
                project_scripts.set(projects_state.project_scripts.get(&project).cloned());
            },
            projects_state,
        );
    }

    project_scripts
}

// #[hook]
// pub fn use_project_scripts(project: ResourceId) -> SuspensionResult<CoreScripts> {
//     let scripts: UseStateHandle<Option<CoreScripts>> = use_state(|| None);

//     if let Some(prj_scripts) = (*scripts).clone() {
//         return Ok(prj_scripts.into());
//     }

//     let (s, handle) = Suspension::new();
//     {
//         let scripts = scripts.clone();
//         let rid = project.clone();

//         spawn_local(async move {
//             let prj_scripts = invoke(
//                 "get_project_scripts",
//                 swb::to_value(&ResourceIdArgs { rid })
//                     .expect("could not convert `ResourceIdArgs` to JsValue"),
//             )
//             .await;

//             let prj_scripts: CoreScripts = swb::from_value(prj_scripts)
//                 .expect("could not convert result of `get_project_scripts` to `Scripts`");

//             scripts.set(Some(prj_scripts));
//             handle.resume();
//         });
//     }

//     Err(s)
// }

#[cfg(test)]
#[path = "./project_scripts_test.rs"]
mod project_scripts_test;

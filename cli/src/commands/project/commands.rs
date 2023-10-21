use super::AddScriptArgs;
use crate::Result;
use std::env;
use thot_core::error::{Error as CoreError, ResourceError};
use thot_local::project::script;

pub fn add_script(args: AddScriptArgs, verbose: bool) -> Result {
    // format project
    let project = match args.project.as_ref() {
        None => env::current_dir().unwrap(),
        Some(p) => p.clone(),
    };

    let project = match thot_local::project::project::project_id(&project)? {
        Some(project) => project,
        None => {
            return Err(CoreError::ResourceError(ResourceError::does_not_exist(
                "path is not a project",
            ))
            .into())
        }
    };

    script::init(project, args.path.clone())?;
    if verbose {
        match args.project.as_ref() {
            None => println!("Added script at {:?} to the current project.", &args.path),
            Some(p) => println!(
                "Added script at {:?} to the project at project {:?}.",
                &args.path, &p
            ),
        }
    }

    Ok(())
}

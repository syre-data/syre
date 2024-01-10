use super::{AssociateScriptArgs, InitArgs, NewArgs};
use crate::common::abs_path;
use crate::Result;
use std::env;
use thot_core::error::{Error as CoreError, Project as CoreProjectError, ResourceError};
use thot_core::project::ScriptAssociation;
use thot_local::error::{ContainerError as LocalContainerError, Error as LocalError};
use thot_local::loader::container::Loader as ContainerLoader;
use thot_local::project::resources::Scripts;
use thot_local::project::{container, project, script};

pub fn init(args: InitArgs, verbose: bool) -> Result {
    let path = match args.path {
        None => env::current_dir()?,
        Some(path) => path,
    };

    let mut builder = container::InitOptions::init();
    builder.recurse(!args.no_recurse);
    if args.no_assets {
        builder.without_assets();
    } else {
        builder.with_assets();
    }

    let rid = builder.build(&path)?;
    if verbose {
        println!("Initialized {path:?} as a Container with {rid:?}");
    }

    Ok(())
}

pub fn new(args: NewArgs, verbose: bool) -> Result {
    let path = abs_path(args.path)?;
    let builder = container::InitOptions::new();
    builder.build(&path)?;
    if verbose {
        println!("Initialized `{path:?}` as a Container");
    }

    Ok(())
}

/// Add a script association to the container.
pub fn associate_script(args: AssociateScriptArgs, verbose: bool) -> Result {
    // validate container and script, if required
    let cont = match args.container {
        None => env::current_dir()?,
        Some(p) => p,
    };

    if !container::path_is_container(&cont) {
        return Err(
            LocalError::ContainerError(LocalContainerError::PathNotAContainer(cont)).into(),
        );
    }

    let prj_path = project::project_root_path(&cont)?;
    let project = match thot_local::project::project::project_id(&prj_path)? {
        Some(project) => project,
        None => {
            return Err(CoreError::ResourceError(ResourceError::does_not_exist(
                "path is not a project",
            ))
            .into())
        }
    };

    let scripts = Scripts::load_from(&prj_path)?;
    let script = scripts
        .values()
        .filter_map(|script| {
            if script.path.as_path() == args.path.as_path() {
                Some(script.rid.clone())
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    let script_id = match script.as_slice() {
        [] => {
            if !args.register {
                return Err(CoreError::Project(CoreProjectError::NotRegistered(
                    None,
                    Some(args.path),
                ))
                .into());
            } else {
                let script_id = script::init(project, args.path.clone())?;
                if verbose {
                    println!(
                        "Path initialized as a Script with {script_id:?} for Project {prj_path:?}"
                    );
                }

                script_id
            }
        }
        [script_id] => script_id.clone(),
        _ => panic!("path registered as script multiple times"),
    };

    let mut container = ContainerLoader::load(&cont)?;
    let mut assoc = ScriptAssociation::new(script_id);
    if args.priority.is_some() {
        assoc.priority = args.priority.unwrap();
    }

    if args.autorun.is_some() {
        assoc.autorun = args.autorun.unwrap();
    }

    container.add_script_association(assoc)?;
    container.save()?;
    Ok(())
}

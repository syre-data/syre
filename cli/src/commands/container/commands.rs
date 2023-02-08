use super::{AddAssetArgs, AddChildArgs, AddScriptArgs, InitArgs, NewArgs, NewChildArgs};
use crate::common::abs_path;
use crate::result::Result;
use settings_manager::local_settings::LocalSettings;
use std::env;
use thot_core::result::{Error as CoreError, ProjectError as CoreProjectError};
use thot_local::project::resources::{Container, ScriptAssociation};
use thot_local::project::{container, project, script};
use thot_local::result::{ContainerError as LocalContainerError, Error as LocalError};

pub fn init(args: InitArgs, verbose: bool) -> Result {
    let path = match args.path {
        None => env::current_dir()?,
        Some(path) => path,
    };

    let rid = container::init(path.as_path())?;
    if verbose {
        println!("Initialized {:?} as a Container with {:?}", &path, &rid);
    }

    Ok(())
}

pub fn new(args: NewArgs, verbose: bool) -> Result {
    let rid = container::new(args.name.as_path())?;
    if verbose {
        println!("Created new Container at {:?} with {:?}", &args.name, &rid);
    }

    Ok(())
}

pub fn add_child(args: AddChildArgs, verbose: bool) -> Result {
    let child = abs_path(args.path)?;
    let parent = match args.parent {
        None => env::current_dir()?,
        Some(p) => abs_path(p)?,
    };

    let _rid = container::init_child(&child, Some(&parent))?;
    if verbose {
        println!("Added {:?} as a child to {:?}", &child, &parent);
    }

    Ok(())
}

pub fn new_child(args: NewChildArgs, verbose: bool) -> Result {
    let parent = match args.parent {
        None => env::current_dir()?,
        Some(p) => abs_path(p)?,
    };

    let mut child = args.path;
    if child.is_relative() {
        let cwd = env::current_dir()?;
        child = cwd.join(child);
    }

    let _rid = container::new_child(&child, Some(&parent))?;
    if verbose {
        println!("Created {:?} as a child of {:?}", &child, &parent);
    }

    Ok(())
}

pub fn add_asset(args: AddAssetArgs, verbose: bool) -> Result {
    let asset = abs_path(args.path)?;
    let parent = match args.parent {
        None => env::current_dir()?,
        Some(p) => abs_path(p)?,
    };

    let mut container = Container::load(&parent)?;
    container.add_asset(&asset)?;
    container.save()?;

    if verbose {
        println!("Added {:?} as an Asset to {:?}", &asset, &parent);
    }

    Ok(())
}

/// Add a script association to the container.
pub fn add_script(args: AddScriptArgs, verbose: bool) -> Result {
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
    let script_id;
    if !script::path_is_registered(&args.path, Some(&prj_path))? {
        if !args.register {
            return Err(CoreError::ProjectError(CoreProjectError::NotRegistered(
                None,
                Some(args.path),
            ))
            .into());
        } else {
            script_id = script::init(&args.path, Some(&prj_path))?;
            if verbose {
                println!(
                    "Path initialized as a Script with {:?} for Project {:?}",
                    &script_id, &prj_path
                );
            }
        }
    } else {
        script_id = script::id_by_path(&args.path, Some(&prj_path))?;
    }

    let mut container = Container::load(&cont)?;
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

#[cfg(test)]
#[path = "./commands_test.rs"]
mod commands_test;

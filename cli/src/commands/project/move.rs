use crate::result::Result;
use clap::Args;
use std::env;
use std::path::PathBuf;
use thot_core::result::{Error as CoreError, ProjectError as CoreProjectError};
use thot_local::common::canonicalize_path;
use thot_local::project::project;
use thot_local::system::projects;

#[derive(Debug, Args)]
pub struct MoveArgs {
    #[clap(parse(from_os_str))]
    to: PathBuf,

    #[clap(long, parse(from_os_str))]
    from: Option<PathBuf>,
}

/// Move a Thot project to a new location.
pub fn main(args: MoveArgs, verbose: bool) -> Result {
    // parse to and from args
    let from = match args.from {
        Some(path) => match canonicalize_path(path) {
            Ok(path) => path,
            Err(err) => return Err(err.into()),
        },
        None => match env::current_dir() {
            Ok(dir) => match project::project_root_path(dir.as_path()) {
                Ok(path) => path,
                Err(err) => return Err(err.into()),
            },
            Err(err) => return Err(err.into()),
        },
    };

    let prj = projects::project_by_path(from.as_path())?;
    let rid = match prj {
        None => {
            return Err(
                CoreError::ProjectError(CoreProjectError::NotRegistered(None, Some(from))).into(),
            )
        }
        Some(p) => p.rid,
    };

    let to = args.to;

    if verbose {
        println!("Moving project located at {:?} to {:?}", from, to);
    }

    // move project
    if let Err(err) = project::mv(&rid, to.as_path()) {
        return Err(err.into());
    }

    Ok(())
}

#[cfg(test)]
#[path = "./move_test.rs"]
mod move_test;

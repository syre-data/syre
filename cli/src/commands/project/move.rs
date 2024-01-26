use crate::Result;
use clap::Args;
use std::path::PathBuf;
use std::{env, fs};
use thot_core::error::{Error as CoreError, Project as CoreProjectError};
use thot_local::error::{Error as LocalError, Project as ProjectError};
use thot_local::project::project;
use thot_local::system::projects;

#[derive(Debug, Args)]
pub struct MoveArgs {
    to: PathBuf,

    #[clap(long)]
    from: Option<PathBuf>,
}

/// Move a Thot project to a new location.
pub fn main(args: MoveArgs, verbose: bool) -> Result {
    // parse to and from args
    let from = match args.from {
        Some(path) => fs::canonicalize(path)?,
        None => match env::current_dir() {
            Ok(dir) => match project::project_root_path(dir.as_path()) {
                Some(path) => path,
                None => {
                    return Err(LocalError::Project(ProjectError::PathNotInProject(
                        dir.as_path().to_path_buf(),
                    ))
                    .into())
                }
            },
            Err(err) => return Err(err.into()),
        },
    };

    let pid = match projects::get_id(&from)? {
        Some(pid) => pid,
        None => {
            return Err(
                CoreError::Project(CoreProjectError::NotRegistered(None, Some(from))).into(),
            )
        }
    };

    project::mv(&pid, args.to.as_path())?;
    if verbose {
        println!("Project moved from `{from:?}` to `{:?}`", args.to);
    }

    Ok(())
}

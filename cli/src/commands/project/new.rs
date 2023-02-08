use crate::result::Result;
use clap::Args;
use std::env;
use std::path::{Path, PathBuf};
use thot_local::project::project;

#[derive(Debug, Args)]
pub struct NewArgs {
    name: String,

    #[clap(short, long, parse(from_os_str))]
    root: Option<PathBuf>,
}

/// Creates a new Thot project.
pub fn main(args: NewArgs, verbose: bool) -> Result {
    // get root path
    let root = match args.root {
        Some(p) => p,
        None => env::current_dir()?,
    };

    let root = root.join(Path::new(&args.name));

    // create project
    if let Err(err) = project::new(root.as_path()) {
        return Err(err.into());
    };

    // set project properties
    // project and root container: creator, name, permissions, lid

    if verbose {
        println!("New project created at {:?}", &root);
    }

    Ok(())
}

#[cfg(test)]
#[path = "./new_test.rs"]
mod new_test;

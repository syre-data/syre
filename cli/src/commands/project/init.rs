use crate::result::Result;
use clap::Args;
use std::env;
use std::path::PathBuf;
use thot_local::project::project;

#[derive(Debug, Args)]
pub struct InitArgs {
    #[clap(short, long, parse(from_os_str))]
    root: Option<PathBuf>,
}

/// Initializes a new Thot project.
pub fn main(args: InitArgs, verbose: bool) -> Result {
    // get root passed in or default to current folder
    let root = match args.root {
        Some(root) => root,
        None => match env::current_dir() {
            Ok(dir) => dir,
            Err(err) => return Err(err.into()),
        },
    };

    if verbose {
        println!("Initializing Thot project in {}", root.display());
    }

    if let Err(err) = project::init(root.as_path()) {
        return Err(err.into());
    }

    Ok(())
}

#[cfg(test)]
#[path = "./init_test.rs"]
mod init_test;

use crate::Result;
use clap::Args;
use std::env;
use std::path::PathBuf;
use thot_local::project;

#[derive(Debug, Args)]
pub struct InitArgs {
    #[clap(short, long)]
    root: Option<PathBuf>,

    /// Relative path to the data root.
    #[clap(long, default_value = "data")]
    data_root: PathBuf,

    /// Relative path to the analysis root.
    #[clap(long, default_value = "analysis")]
    analysis_root: PathBuf,
}

/// Initializes a new Thot project.
/// Initilizes and registers the project.
/// Initializes any existing folders as the `Container` graph moving them into the data root folder.
/// If analysis root exists, registers all valid files as scripts.
///
/// Searches for `data` folder to be used as the `Project`'s data root,
/// and `analysis` folder for the analysis root, creating them if they don't exist.
/// If the data root folder does not exist, move the contents of the folder into it.
/// e.g.
/// **Original**
/// my_project/
///     |- root_data.csv
///     |- group_1/
///     |   |- group_1_data.csv
///
/// **Initialized**
/// my_project/
///     |- analysis/
///     |- data/
///     |   |- root_data.csv
///     |   |- group_1
///     |   |   |- group_1_data.csv
///
pub fn main(args: InitArgs, verbose: bool) -> Result {
    let root = match args.root {
        Some(root) => root.clone(),
        None => match env::current_dir() {
            Ok(dir) => dir,
            Err(err) => return Err(err.into()),
        },
    };

    project::init(&root, &args.data_root, &args.analysis_root)?;
    if verbose {
        println!("Initialized {root:?} as a Thot project.");
    }

    Ok(())
}

#[cfg(test)]
#[path = "./init_test.rs"]
mod init_test;

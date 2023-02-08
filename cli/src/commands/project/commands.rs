use super::AddScriptArgs;
use crate::result::Result;
use thot_local::project::script;

pub fn add_script(args: AddScriptArgs, verbose: bool) -> Result {
    // format project
    let project = match &args.project {
        None => None,
        Some(p) => Some(p.as_path()),
    };

    script::init(&args.path, project)?;
    if verbose {
        match &args.project {
            None => println!("Added script at {:?} to the current project.", &args.path),
            Some(p) => println!(
                "Added script at {:?} to the project at project {:?}.",
                &args.path, &p
            ),
        }
    }

    Ok(())
}

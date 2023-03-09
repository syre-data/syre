//! Thot project runner.
use super::resources::script_groups::{ScriptGroups, ScriptSet};
use super::ThotEnv;
use crate::error::RunnerError;
use crate::project::{Container, Script};
use crate::types::ResourceId;
use crate::{Error, Result};
use std::collections::HashSet;
use std::result::Result as StdResult;
use std::{process, str};

// ********************************
// *** Script Execution Context ***
// ********************************

/// Identifies the context in which a script was run.
#[derive(Clone, Debug)]
pub struct ScriptExecutionContext {
    /// [`ResourceId`] of the [`Script`] being executed.
    pub script: ResourceId,

    /// [`ResourceId`] of the [`Container`] the script was executed on.
    pub container: ResourceId,
}

// *************
// *** Hooks ***
// *************

/// Retrieves a [`Script`] from its [`ResouceId`].
pub type GetScriptHook = fn(&ResourceId) -> Result<Script>;

/// Used to handle script errors during execution.
///
/// # Arguments
/// 1. [`ScriptExecutionContext`]
/// 2. [`RunnerError`] that caused the script to fail.
/// 3. Verbose
///
/// # Returns
/// A [`Result`](StdResult) indicating whether to contiue execution (`Ok`) or
/// halt (`Err`).
pub type ScriptErrorHook =
    fn(ScriptExecutionContext, RunnerError, bool) -> StdResult<(), RunnerError>;

/// Handles post-processing of the [`Asset`](crate::project::Asset)s added
/// during execution.
///
/// # Arguments
/// 1. [`ScriptExecutionContext`]
/// 2. `HashSet` of the [`Asset`](crate::project::Asset)s added from the
///     script's execution.
/// 3. Verbose
pub type AssetsAddedHook = fn(ScriptExecutionContext, HashSet<ResourceId>, bool);

/// A generic runner hook.
///
/// # Arguments
/// 1. [`ScriptExecutionContext`]
/// 2. Verbose
pub type RunnerHook = fn(ScriptExecutionContext, bool);

// ********************
// *** Runner Hooks ***
// ********************

/// Hooks to link into the execution cycle of a [`Runner`].
pub struct RunnerHooks {
    /// Retrieve a [`Script`] from its [`ResourceId`].
    pub get_script: GetScriptHook,

    /// Run when a script errors.
    /// Should return `Ok` if evaluation should continue, or
    /// `Err` to defer to the `ignore_errors` state of the execution.
    /// See [`Runner::run_scripts`].
    pub script_error: Option<ScriptErrorHook>,

    /// Runs before every script.
    pub pre_script: Option<RunnerHook>,

    /// Run after a script exits successfully and evaluation will continue.
    /// i.e. This handle does not run if the srcipt errors and the error is
    /// not successfully handled by `script_error` or ignored.
    pub post_script: Option<RunnerHook>,

    /// Run after a script finishes.
    /// This runs before `post_script` and regardless of the success of the script.
    pub assets_added: Option<AssetsAddedHook>,
}

impl RunnerHooks {
    pub fn new(get_script: GetScriptHook) -> Self {
        Self {
            get_script,
            script_error: None,
            pre_script: None,
            post_script: None,
            assets_added: None,
        }
    }
}

// **************
// *** Runner ***
// **************

#[cfg_attr(doc, aquamarine::aquamarine)]
/// ```mermaid
///
/// ```
pub struct Runner {
    pub hooks: RunnerHooks,
}

impl Runner {
    pub fn new(hooks: RunnerHooks) -> Self {
        Self { hooks }
    }

    /// Analyze a tree
    ///
    /// # Arguments
    /// + `root`: Root of the `Container` tree to evaluate.
    /// + `max_tasks`: Maximum number of analysis tasks to run at once.
    ///     If `None` runs as many as possible.
    pub fn run(&self, root: Container, max_tasks: Option<usize>) -> Result {
        self.evaluate_tree(root)
    }

    /// Evaluates a `Container` tree.
    fn evaluate_tree(&self, root: Container) -> Result {
        // recurse on children
        let children = root.children.values().filter_map(|c| c.clone());
        for child in children {
            // @todo: Handle error.
            let child = child.lock().expect("could not lock child `Container`");
            self.evaluate_tree((*child).clone())?;
        }

        self.evaluate_container(root, None, false, false)
    }

    /// Evaluates a single container.
    ///
    /// # Arguments
    /// + `container`: The [`Container`] to evaluate.
    /// + `scripts_filter`: `None` to run all scripts set to `autorun`.
    ///     Otherwise a [`HashSet`] of the scripts to run.
    /// + `ignore_errors`: Whether to continue running on a script error.
    /// + `verbose`: Output state.
    fn evaluate_container(
        &self,
        container: Container,
        script_filter: Option<HashSet<ResourceId>>,
        ignore_errors: bool,
        verbose: bool,
    ) -> Result {
        let mut scripts = container.scripts.clone();
        if let Some(filter) = script_filter {
            // filter scripts
            scripts.retain(|rid, _script| filter.contains(rid));
        }

        // batch and sort scripts by priority
        let mut script_groups: Vec<(i32, ScriptSet)> = ScriptGroups::from(scripts).into();
        script_groups.sort_by(|(p0, _), (p1, _)| p0.cmp(p1));

        for (_priority, script_group) in script_groups {
            let get_script = self.hooks.get_script;
            let scripts = script_group
                .into_iter()
                .filter(|s| s.autorun)
                .map(|assoc| {
                    let rid = assoc.script;
                    get_script(&rid)
                        .expect(&format!("could not retrieve `Script` with id `{}`", rid))
                })
                .collect();

            self.run_scripts(scripts, &container, ignore_errors, verbose)?;
        }

        Ok(())
    }

    #[cfg_attr(doc, aquamarine::aquamarine)]
    /// Runs a group of scripts.
    ///
    /// ```mermaid
    ///flowchart TD
    ///    %% happy path
    ///    run_scripts("run_scripts(scripts: Vec&lt;Script&gt;, container: Container, ...)") -- "for script in scripts" --> pre_script("pre_script(ctx: ScriptExecutionContext, verbose: bool)")
    ///    pre_script --> run_script("run_script(script: Script, container: Container, ...)")
    ///    run_script -- "Result&lt;Ok, Err&gt;" --> assets_added("assets_added(ScriptExecutionContext, assets: HashSet<RerourceId>, verboes: bool)")
    ///    assets_added -- "Ok(())" --> post_script("post_script(ctx: ScriptExecutionContext, verbose: bool)")
    ///    post_script --> pre_script
    ///    post_script -- "complete" --> exit("Ok(())")

    ///    %% error path
    ///    assets_added -- "Err(RunnerError)" --> script_error("script_error(ctx: ScriptExecutionContext, err: RunnerError, verbose: bool)")
    ///    script_error -- "Ok(())" --> post_script
    ///    script_error -- "Err(_)" --> ignore_errors("ignore_errors")
    ///    ignore_errors -- "true" --> post_script
    ///    ignore_errors -- "false" ---> break("return Err(_)")
    /// ```
    fn run_scripts(
        &self,
        scripts: Vec<Script>,
        container: &Container,
        ignore_errors: bool,
        verbose: bool,
    ) -> Result {
        for script in scripts {
            let exec_ctx = ScriptExecutionContext {
                script: script.rid.clone(),
                container: container.rid.clone(),
            };

            if let Some(pre_script) = self.hooks.pre_script {
                pre_script(exec_ctx.clone(), verbose);
            }

            let run_res = self.run_script(script, &container);

            if let Some(assets_added) = self.hooks.assets_added {
                let assets = HashSet::new(); // @todo: Collect `ResourceId`s of `Assets`.
                assets_added(exec_ctx.clone(), assets, verbose);
            }

            match run_res {
                Err(Error::RunnerError(err)) => {
                    if let Some(script_error) = self.hooks.script_error {
                        match script_error(exec_ctx.clone(), err, verbose) {
                            Ok(()) => {}
                            Err(err) => {
                                if !ignore_errors {
                                    return Err(err.into());
                                }
                            }
                        }
                    } else {
                        if !ignore_errors {
                            return Err(err.into());
                        }
                    }
                }
                Err(err) => return Err(err.into()), // do not ignore non `RunnerError`s
                Ok(_) => {}
            }

            if let Some(post_script) = self.hooks.post_script {
                post_script(exec_ctx, verbose);
            }
        }

        Ok(())
    }

    /// Runs an individual script.
    ///
    /// # Returns
    /// [`Output`](process:Output) from the script.
    ///
    /// # Errors
    /// + [`RunnerError`]: The script returned a `status` other than `0`.
    fn run_script(&self, script: Script, container: &Container) -> Result<process::Output> {
        let out = process::Command::new(&script.env.cmd)
            .arg(script.path.as_path())
            .args(&script.env.args)
            .env(
                ThotEnv::container_id_key(),
                container.rid.clone().to_string(),
            )
            .envs(&script.env.env)
            .output()
            .expect("failed to execute command");

        if !out.status.success() {
            let stderr = str::from_utf8(out.stderr.as_slice())
                .expect("stderr should work")
                .to_string();

            return Err(RunnerError::ScriptError(
                script.rid.clone(),
                container.rid.clone(),
                stderr,
            )
            .into());
        }

        Ok(out)
    }
}

#[cfg(test)]
#[path = "./runner_test.rs"]
mod runner_test;

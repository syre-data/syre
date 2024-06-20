//! Syre project runner.
// use super::resources::script_groups::{ScriptGroups, ScriptSet};
use super::{Runnable, CONTAINER_ID_KEY};
use crate::error::Runner as RunnerError;
use crate::graph::ResourceTree;
use crate::project::Container;
use crate::types::ResourceId;
use std::collections::{HashMap, HashSet};
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
pub type GetScriptHook = fn(&ResourceId) -> StdResult<Box<dyn Runnable>, String>;

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
pub type ScriptErrorHook = fn(ScriptExecutionContext, RunnerError, bool) -> Result;

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

type Result<T = ()> = std::result::Result<T, RunnerError>;
type ContainerTree = ResourceTree<Container>;

// TODO Make builder.
#[cfg_attr(doc, aquamarine::aquamarine)]
/// ```mermaid
///
/// ```
pub struct Runner {
    hooks: RunnerHooks,
}

impl Runner {
    pub fn new(hooks: RunnerHooks) -> Self {
        Self { hooks }
    }

    /// Analyze a tree.
    ///
    /// # Arguments
    /// 1. Container tree to evaluate.
    pub fn run(&self, tree: &mut ContainerTree) -> Result {
        let root = tree.root().clone();
        let mut analyzer = TreeRunner::new(tree, &root, &self.hooks);
        analyzer.run()
    }

    /// Analyze a tree using restricted parallelization.
    ///
    /// # Arguments
    /// 1. Container tree to evaluate.
    /// 2. Maximum number of analysis tasks to run at once.
    pub fn with_tasks(&self, tree: &mut ContainerTree, tasks: usize) -> Result {
        let root = tree.root().clone();
        let mut analyzer = TreeRunner::with_tasks(tree, &root, &self.hooks, tasks);
        analyzer.run()
    }

    /// Analyze a subtree.
    ///
    /// # Arguments
    /// 1. Container tree to evaluate.
    /// 2. Root of subtree.
    pub fn from(&self, tree: &mut ContainerTree, root: &ResourceId) -> Result {
        let mut analyzer = TreeRunner::new(tree, root, &self.hooks);
        analyzer.run()
    }

    /// Analyze a subtree using restricted parallelization.
    ///
    /// # Arguments
    /// 1. Container tree to evaluate.
    /// 2. Root of subtree.
    /// 3. Maximum number of analysis tasks to run at once.
    pub fn with_tasks_from(
        &self,
        tree: &mut ContainerTree,
        root: &ResourceId,
        tasks: usize,
    ) -> Result {
        let mut analyzer = TreeRunner::with_tasks(tree, root, &self.hooks, tasks);
        analyzer.run()
    }
}

struct TreeRunner<'a> {
    tree: &'a mut ContainerTree,
    root: &'a ResourceId,
    hooks: &'a RunnerHooks,
    max_tasks: Option<usize>,
    ignore_errors: bool,
    verbose: bool,
    analyses: HashMap<ResourceId, Box<dyn Runnable>>,
}

impl<'a> TreeRunner<'a> {
    pub fn new(tree: &'a mut ContainerTree, root: &'a ResourceId, hooks: &'a RunnerHooks) -> Self {
        Self {
            tree,
            root,
            hooks,
            max_tasks: None,
            ignore_errors: false,
            verbose: false,
            analyses: HashMap::new(),
        }
    }

    pub fn with_tasks(
        tree: &'a mut ContainerTree,
        root: &'a ResourceId,
        hooks: &'a RunnerHooks,
        max_tasks: usize,
    ) -> Self {
        Self {
            tree,
            root,
            hooks,
            max_tasks: Some(max_tasks),
            ignore_errors: false,
            verbose: false,
            analyses: HashMap::new(),
        }
    }

    pub fn run(&mut self) -> Result {
        let get_script = self.hooks.get_script;
        let mut script_errors = HashMap::new();
        for (_, container) in self.tree.iter_nodes() {
            for sid in container
                .analyses
                .iter()
                .map(|association| association.analysis())
            {
                if self.analyses.contains_key(sid) {
                    continue;
                }

                match get_script(sid) {
                    Ok(script) => {
                        self.analyses.insert(sid.clone(), script);
                    }

                    Err(err) => {
                        script_errors.insert(sid.clone(), err);
                    }
                }
            }
        }

        if !script_errors.is_empty() {
            return Err(RunnerError::LoadScripts(script_errors));
        }

        self.evaluate_tree(self.root)
    }

    /// Evaluates a `Container` tree.
    ///
    /// # Arguments
    /// 1. Container tree to evaluate.
    /// 2. Root of subtree.
    /// 3. Maximum number of analysis tasks to run at once.
    #[tracing::instrument(skip(self))]
    fn evaluate_tree(&self, root: &ResourceId) -> Result {
        // recurse on children
        let Some(children) = self.tree.children(root).cloned() else {
            return Err(RunnerError::ContainerNotFound(root.clone()));
        };

        for child in children {
            self.evaluate_tree(&child)?;
        }

        self.evaluate_container(root)
    }

    /// Evaluates a single container.
    ///
    /// # Arguments
    /// 1. The [`ContainerTree`].
    /// 1. The [`Container`] to evaluate.
    /// 2. `None` to run all scripts set to `autorun`.
    ///     Otherwise a [`HashSet`] of the scripts to run.
    /// + `ignore_errors`: Whether to continue running on a script error.
    /// + `verbose`: Output state.
    #[tracing::instrument(skip(self))]
    fn evaluate_container(&self, container: &ResourceId) -> Result {
        let Some(container) = self.tree.get(container) else {
            return Err(RunnerError::ContainerNotFound(container.clone()));
        };

        // batch and sort scripts by priority
        let mut analysis_groups = HashMap::new();
        for association in container.analyses.iter() {
            let group = analysis_groups
                .entry(association.priority)
                .or_insert(vec![]);

            group.push(association);
        }

        let mut analysis_groups = analysis_groups.into_iter().collect::<Vec<_>>();
        analysis_groups.sort_by(|(p0, _), (p1, _)| p0.cmp(p1));

        for (_priority, script_group) in analysis_groups {
            let scripts = script_group
                .into_iter()
                .filter(|s| s.autorun)
                .map(|assoc| self.analyses.get(&assoc.analysis()).unwrap())
                .collect();

            self.run_scripts(scripts, &container)?;
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
    #[tracing::instrument(skip(self, scripts))]
    fn run_scripts(&self, scripts: Vec<&Box<dyn Runnable>>, container: &Container) -> Result {
        for script in scripts {
            let exec_ctx = ScriptExecutionContext {
                script: script.id().clone(),
                container: container.rid().clone(),
            };

            if let Some(pre_script) = self.hooks.pre_script {
                pre_script(exec_ctx.clone(), self.verbose);
            }

            let run_res = self.run_script(script, &container);

            if let Some(assets_added) = self.hooks.assets_added {
                let assets = HashSet::new(); // TODO: Collect `ResourceId`s of `Assets`.
                assets_added(exec_ctx.clone(), assets, self.verbose);
            }

            match run_res {
                Ok(_) => {}

                Err(err) => {
                    if let Some(script_error) = self.hooks.script_error {
                        match script_error(exec_ctx.clone(), err, self.verbose) {
                            Ok(()) => {}
                            Err(err) => {
                                if !self.ignore_errors {
                                    return Err(err.into());
                                }
                            }
                        }
                    } else {
                        if !self.ignore_errors {
                            return Err(err.into());
                        }
                    }
                }
            }

            if let Some(post_script) = self.hooks.post_script {
                post_script(exec_ctx, self.verbose);
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
    #[tracing::instrument(skip(self, script))]
    fn run_script(
        &self,
        script: &Box<dyn Runnable>,
        container: &Container,
    ) -> Result<process::Output> {
        let mut out = script.command();
        let out = match out
            .env(CONTAINER_ID_KEY, container.rid().clone().to_string())
            .output()
        {
            Ok(out) => out,
            Err(err) => {
                tracing::debug!(?err);
                return Err(RunnerError::CommandError {
                    script: script.id().clone(),
                    container: container.rid().clone(),
                    cmd: format!("{out:?}"),
                }
                .into());
            }
        };

        if !out.status.success() {
            let stderr = str::from_utf8(out.stderr.as_slice()).unwrap().to_string();

            return Err(RunnerError::ScriptError {
                script: script.id().clone(),
                container: container.rid().clone(),
                description: stderr,
            }
            .into());
        }

        Ok(out)
    }
}

//! Syre project runner.
use super::{Runnable, CONTAINER_ID_KEY};
use crate::{graph::ResourceTree, project::Container, types::ResourceId};
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    result::Result as StdResult,
    {process, str},
};

/// Identifies the context in which an analysis was run.
#[derive(Clone, Debug)]
pub struct AnalysisExecutionContext {
    /// [`ResourceId`] of the analysis being executed.
    pub analysis: ResourceId,

    /// [`ResourceId`] of the [`Container`] the analysis was executed on.
    pub container: ResourceId,
}

// /// Retrieves an analysis from its project and [`ResouceId`].
// ///
// /// # Arguments
// /// 1. Project's id.
// /// 2. Analysis' id.
// pub type GetAnalysisHook = fn(&ResourceId, &ResourceId) -> StdResult<Box<dyn Runnable>, String>;

// /// Used to handle analysis errors during execution.
// ///
// /// # Arguments
// /// 1. [`AnalysisExecutionContext`]
// /// 2. [`Error`] that caused the analysis to fail.
// /// 3. Verbose
// ///
// /// # Returns
// /// A [`Result`](StdResult) indicating whether to contiue execution (`Ok`) or
// /// halt (`Err`).
// pub type AnalysisErrorHook = fn(AnalysisExecutionContext, Error, bool) -> Result;

// /// Handles post-processing of the [`Asset`](crate::project::Asset)s added
// /// during execution.
// ///
// /// # Arguments
// /// 1. [`AnalysisExecutionContext`]
// /// 2. `HashSet` of the [`Asset`](crate::project::Asset)s added from the
// ///     analysis' execution.
// /// 3. Verbose
// pub type AssetsAddedHook = fn(AnalysisExecutionContext, HashSet<ResourceId>, bool);

// /// A generic runner hook.
// ///
// /// # Arguments
// /// 1. [`AnalysisExecutionContext`]
// /// 2. Verbose
// pub type RunnerHook = fn(AnalysisExecutionContext, bool);

// /// Hooks to link into the execution cycle of a [`Runner`].
// pub struct RunnerHooks {
//     /// Retrieve a [`Analysis`] from its [`ResourceId`].
//     pub get_analysis: GetAnalysisHook,

//     /// Run when a analysis errors.
//     /// Should return `Ok` if evaluation should continue, or
//     /// `Err` to defer to the `ignore_errors` state of the execution.
//     /// See [`Runner::run_analyses`].
//     pub analysis_error: Option<AnalysisErrorHook>,

//     /// Runs before every analysis.
//     pub pre_analysis: Option<RunnerHook>,

//     /// Run after a analysis exits successfully and evaluation will continue.
//     /// i.e. This handle does not run if the srcipt errors and the error is
//     /// not successfully handled by `analysis_error` or ignored.
//     pub post_analysis: Option<RunnerHook>,

//     /// Run after a analysis finishes.
//     /// This runs before `post_analysis` and regardless of the success of the analysis.
//     pub assets_added: Option<AssetsAddedHook>,
// }

// impl RunnerHooks {
//     pub fn new(get_analysis: GetAnalysisHook) -> Self {
//         Self {
//             get_analysis,
//             analysis_error: None,
//             pre_analysis: None,
//             post_analysis: None,
//             assets_added: None,
//         }
//     }
// }

type Result<T = ()> = std::result::Result<T, Error>;
type ContainerTree = ResourceTree<Container>;

pub trait RunnerHooks {
    /// Retrieve an analysis from its [`ResourceId`].
    fn get_analysis(&self, analysis: ResourceId) -> StdResult<Box<dyn Runnable>, String>;

    /// Run when an analysis errors.
    /// Should return `Ok` if evaluation should continue, or
    /// `Err` to defer to the `ignore_errors` state of the execution.
    ///
    /// # Notes
    /// + Default implmentation ignores errors.
    ///
    /// # See also
    /// [`Runner::run_analyses`].
    fn analysis_error(&self, ctx: AnalysisExecutionContext, err: Error) -> Result {
        Ok(())
    }

    /// Runs before every analysis.
    fn pre_analysis(&self, ctx: AnalysisExecutionContext) {}

    /// Run after an analysis exits successfully and evaluation will continue.
    /// i.e. This handle does not run if the analysis errors and the error is
    /// not successfully handled by `analysis_error` or ignored.
    fn post_analysis(&self, ctx: AnalysisExecutionContext) {}

    /// Run after an analysis finishes.
    /// This runs before `post_analysis` and regardless of the success of the analysis.
    fn assets_added(&self, ctx: AnalysisExecutionContext, assets: Vec<ResourceId>) {}
}

// TODO Make builder.
#[cfg_attr(doc, aquamarine::aquamarine)]
/// ```mermaid
///
/// ```
pub struct Runner {
    hooks: Box<dyn RunnerHooks>,
}

impl Runner {
    pub fn new(hooks: Box<dyn RunnerHooks>) -> Self {
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
    hooks: &'a Box<dyn RunnerHooks>,
    max_tasks: Option<usize>,
    ignore_errors: bool,
    verbose: bool,
    analyses: HashMap<ResourceId, Box<dyn Runnable>>,
}

impl<'a> TreeRunner<'a> {
    pub fn new(
        tree: &'a mut ContainerTree,
        root: &'a ResourceId,
        hooks: &'a Box<dyn RunnerHooks>,
    ) -> Self {
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
        hooks: &'a Box<dyn RunnerHooks>,
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
        let mut analysis_errors = HashMap::new();
        for (_, container) in self.tree.iter_nodes() {
            for aid in container
                .analyses
                .iter()
                .map(|association| association.analysis())
            {
                if self.analyses.contains_key(aid) {
                    continue;
                }

                match self.hooks.get_analysis(aid.clone()) {
                    Ok(analysis) => {
                        self.analyses.insert(aid.clone(), analysis);
                    }

                    Err(err) => {
                        analysis_errors.insert(aid.clone(), err);
                    }
                }
            }
        }

        if !analysis_errors.is_empty() {
            return Err(Error::LoadAnalyses(analysis_errors));
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
            return Err(Error::ContainerNotFound(root.clone()));
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
    /// 2. `None` to run all analyses set to `autorun`.
    ///     Otherwise a [`HashSet`] of the analyses to run.
    /// + `ignore_errors`: Whether to continue running on a analysis error.
    /// + `verbose`: Output state.
    #[tracing::instrument(skip(self))]
    fn evaluate_container(&self, container: &ResourceId) -> Result {
        let Some(container) = self.tree.get(container) else {
            return Err(Error::ContainerNotFound(container.clone()));
        };

        // batch and sort analyses by priority
        let mut analysis_groups = HashMap::new();
        for association in container.analyses.iter() {
            let group = analysis_groups
                .entry(association.priority)
                .or_insert(vec![]);

            group.push(association);
        }

        let mut analysis_groups = analysis_groups.into_iter().collect::<Vec<_>>();
        analysis_groups.sort_by(|(p0, _), (p1, _)| p0.cmp(p1));

        for (_priority, analysis_group) in analysis_groups {
            let analyses = analysis_group
                .into_iter()
                .filter(|s| s.autorun)
                .map(|assoc| self.analyses.get(assoc.analysis()).unwrap())
                .collect();

            self.run_analyses(analyses, &container)?;
        }

        Ok(())
    }

    #[cfg_attr(doc, aquamarine::aquamarine)]
    /// Runs a group of analyses.
    ///
    /// ```mermaid
    ///flowchart TD
    ///    %% happy path
    ///    run_analyses("run_analyses(analyses: Vec&ltAnalysis&gt;, container: Container, ...)") -- "for analysis in analyses" --> pre_analysis("pre_analysis(ctx: AnalysisExecutionContext, verbose: bool)")
    ///    pre_analysis --> run_analyses("run_analyses(analysis: Analysis, container: Container, ...)")
    ///    run_analysis -- "Result&lt;Ok, Err&gt;" --> assets_added("assets_added(AnalysisExecutionContext, assets: HashSet<RerourceId>, verboes: bool)")
    ///    assets_added -- "Ok(())" --> post_analysis("post_analysis(ctx: AnalysisExecutionContext, verbose: bool)")
    ///    post_analysis --> pre_analysis
    ///    post_analysis -- "complete" --> exit("Ok(())")

    ///    %% error path
    ///    assets_added -- "Err(Error)" --> analysis_error("analysis_error(ctx: AnalysisExecutionContext, err: Error, verbose: bool)")
    ///    analysis_error -- "Ok(())" --> post_analysis
    ///    analysis_error -- "Err(_)" --> ignore_errors("ignore_errors")
    ///    ignore_errors -- "true" --> post_analysis
    ///    ignore_errors -- "false" ---> break("return Err(_)")
    /// ```
    #[tracing::instrument(skip(self, analyses))]
    fn run_analyses(&self, analyses: Vec<&Box<dyn Runnable>>, container: &Container) -> Result {
        for analysis in analyses {
            let exec_ctx = AnalysisExecutionContext {
                analysis: analysis.id().clone(),
                container: container.rid().clone(),
            };

            self.hooks.pre_analysis(exec_ctx.clone());

            let run_res = self.run_analysis(analysis, &container);
            let assets = Vec::new(); // TODO: Collect `ResourceId`s of `Assets`.
            self.hooks.assets_added(exec_ctx.clone(), assets);
            match run_res {
                Ok(_) => {}
                Err(err) => match self.hooks.analysis_error(exec_ctx.clone(), err) {
                    Ok(()) => {}
                    Err(err) => {
                        if !self.ignore_errors {
                            return Err(err.into());
                        }
                    }
                },
            }

            self.hooks.post_analysis(exec_ctx);
        }

        Ok(())
    }

    /// Runs an individual analysis.
    ///
    /// # Returns
    /// [`Output`](process:Output) from the analysis.
    ///
    /// # Errors
    /// + [`Error`]: The analysis returned a `status` other than `0`.
    #[tracing::instrument(skip(self, analysis))]
    fn run_analysis(
        &self,
        analysis: &Box<dyn Runnable>,
        container: &Container,
    ) -> Result<process::Output> {
        let mut out = analysis.command();
        let out = match out
            .env(CONTAINER_ID_KEY, container.rid().clone().to_string())
            .output()
        {
            Ok(out) => out,
            Err(err) => {
                tracing::debug!(?err);
                return Err(Error::CommandError {
                    analysis: analysis.id().clone(),
                    container: container.rid().clone(),
                    cmd: format!("{out:?}"),
                }
                .into());
            }
        };

        if !out.status.success() {
            let stderr = str::from_utf8(out.stderr.as_slice()).unwrap().to_string();

            return Err(Error::AnalysisError {
                analysis: analysis.id().clone(),
                container: container.rid().clone(),
                description: stderr,
            }
            .into());
        }

        Ok(out)
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("{0:?}")]
    LoadAnalyses(HashMap<ResourceId, String>),

    /// The `Container` could not be found in the graph.
    #[error("Container {0} not found")]
    ContainerNotFound(ResourceId),

    /// An error occured when running the analysis
    /// on the specified `Container`.
    #[error("Analysis `{analysis}` running over Container `{container}` errored: {description}")]
    AnalysisError {
        analysis: ResourceId,
        container: ResourceId,
        description: String,
    },

    #[error("error running `{cmd}` from analysis `{analysis}` on container `{container}`")]
    CommandError {
        analysis: ResourceId,
        container: ResourceId,
        cmd: String,
    },
}

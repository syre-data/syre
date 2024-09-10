use std::{fs, io, path::PathBuf};
use syre_core::{
    self as core,
    project::Project,
    runner::RunnerHooks,
    types::{ResourceId, UserId, UserPermissions},
};
use syre_desktop_lib as lib;
use syre_local::{
    self as local,
    project::{
        project,
        resources::{
            Analyses as LocalAnalyses, Container as LocalContainer, Project as LocalProject,
        },
    },
    types::AnalysisKind,
};
use syre_local_database as db;
use syre_local_runner as runner;

#[tauri::command]
pub fn create_project(user: ResourceId, path: PathBuf) -> syre_local::Result<Project> {
    project::init(&path)?;

    // create analysis folder
    let analysis_root = "analysis";
    let mut analysis = path.to_path_buf();
    analysis.push(analysis_root);
    fs::create_dir(&analysis).unwrap();

    let mut project = LocalProject::load_from(path)?;
    let settings = project.settings_mut();
    settings.creator = Some(UserId::Id(user.clone()));
    settings
        .permissions
        .insert(user.clone(), UserPermissions::all());
    project.analysis_root = Some(PathBuf::from(analysis_root));
    project.save()?;

    let mut root = LocalContainer::new(project.data_root_path());
    root.settings_mut().creator = Some(UserId::Id(user.clone()));
    root.save()?;

    Ok(project.into())
}

/// # Returns
/// Tuple of (project path, project data, project graph).
#[tauri::command]
pub fn project_resources(
    db: tauri::State<db::Client>,
    project: ResourceId,
) -> Option<(
    PathBuf,
    db::state::ProjectData,
    db::state::FolderResource<db::state::Graph>,
)> {
    let resources = db.project().resources(project).unwrap();
    assert!(if let Some((_, data, _)) = resources.as_ref() {
        data.properties().is_ok()
    } else {
        true
    });

    resources
}

/// # Arguments
/// + `path`: Relative path from the analysis root.
#[tauri::command]
pub fn project_analysis_remove(
    db: tauri::State<db::Client>,
    project: ResourceId,
    path: PathBuf,
) -> Result<(), lib::command::project::error::AnalysesUpdate> {
    use lib::command::project::error::AnalysesUpdate;

    let (project_path, project) = db.project().get_by_id(project).unwrap().unwrap();
    let mut analyses = match LocalAnalyses::load_from(&project_path) {
        Ok(analyses) => analyses,
        Err(err) => return Err(AnalysesUpdate::AnalysesFile(err)),
    };

    analyses.retain(|_, analysis| match analysis {
        AnalysisKind::Script(script) => script.path != path,
        AnalysisKind::ExcelTemplate(template) => template.template.path != path,
    });

    if let Err(err) = analyses.save() {
        return Err(AnalysesUpdate::AnalysesFile(err.kind().into()));
    }

    if let db::state::DataResource::Ok(properties) = project.properties() {
        let path = project_path
            .join(properties.analysis_root.as_ref().unwrap())
            .join(path);

        if let Err(err) = fs::remove_file(&path) {
            return Err(AnalysesUpdate::RemoveFile(err.kind()));
        };
    }
    Ok(())
}

#[tauri::command]
pub fn analyze_project(
    db: tauri::State<db::Client>,
    project: ResourceId,
    root: PathBuf,
    max_tasks: Option<usize>,
) -> Result<(), lib::command::project::error::Analyze> {
    use lib::command::project::error;

    let (project_path, project_data, graph) =
        db.project().resources(project.clone()).unwrap().unwrap();
    let db::state::FolderResource::Present(graph) = graph else {
        return Err(error::Analyze::GraphAbsent);
    };

    let runner_hooks = match runner::Runner::from(project_path, &project_data) {
        Ok(hooks) => hooks,
        Err(err) => return Err(err.into()),
    };
    let runner_hooks = Box::new(runner_hooks) as Box<dyn RunnerHooks>;
    let runner = core::runner::Runner::new(runner_hooks);
    let Ok(mut graph) = graph_state_to_container_tree(graph) else {
        return Err(error::Analyze::InvalidGraph);
    };
    let root = graph.get_path(&root).unwrap().unwrap().rid().clone();
    match max_tasks {
        None => runner.from(&project, &mut graph, &root)?,
        Some(max_tasks) => runner.with_tasks(&project, &mut graph, max_tasks)?,
    }

    Ok(())
}

fn graph_state_to_container_tree(
    graph: db::state::Graph,
) -> Result<core::graph::ResourceTree<core::project::Container>, InvalidGraph> {
    let db::state::Graph { nodes, children } = graph;
    let mut nodes = nodes
        .into_iter()
        .map(|node| node.as_container())
        .collect::<Vec<_>>();
    if nodes.iter().any(|node| node.is_none()) {
        return Err(InvalidGraph);
    }
    let nodes = nodes
        .into_iter()
        .map(|node| {
            let node = node.unwrap();
            (node.rid().clone(), core::graph::ResourceNode::new(node))
        })
        .collect::<Vec<_>>();

    let edges = children
        .into_iter()
        .enumerate()
        .map(|(idx, children)| {
            let children = children
                .into_iter()
                .map(|idx| nodes[idx].0.clone())
                .collect();

            (nodes[idx].0.clone(), children)
        })
        .collect::<core::graph::tree::EdgeMap>();

    let nodes = nodes.into_iter().collect::<core::types::ResourceMap<_>>();
    Ok(core::graph::ResourceTree::from_parts(nodes, edges).unwrap())
}

#[derive(Debug)]
struct InvalidGraph;

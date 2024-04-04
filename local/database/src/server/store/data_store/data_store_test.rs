use super::super::command::{graph::Command as GraphCommand, project::Command as ProjectCommand};
use super::project::Record as ProjectRecord;
use super::*;
use std::path::PathBuf;
use std::thread;
use syre_core::graph::ResourceTree;
use syre_core::project::{asset::Builder as Asset, container::Builder as Container};

type ContainerTree = ResourceTree<syre_core::project::Container>;

#[test]
fn datastore_search_should_work() {
    // *************
    // *** setup ***
    // *************

    let pid = ResourceId::new();
    let project = ProjectRecord::new(
        "fireworks".to_string(),
        Some("Testing which fireworks recipe has the quietest explosion.".to_string()),
        PathBuf::from("/Users/user/Documents/test"),
    );

    let mut recipe_comp_png = Asset::with_path("recipe_comparison.png");
    recipe_comp_png
        .set_kind("recipe-comparison")
        .add_tag("image");
    let recipe_comp_png = recipe_comp_png.build();
    let recipe_comp_png_id = recipe_comp_png.rid.clone();

    let mut recipe_comp_csv = Asset::with_path("recipe_comparison.csv");
    recipe_comp_csv
        .set_kind("recipe-comparison")
        .add_tag("table");
    let recipe_comp_csv = recipe_comp_csv.build();
    let recipe_comp_csv_id = recipe_comp_csv.rid.clone();

    let mut root = Container::new("Root");
    root.set_kind("project")
        .set_description("Testing different firework recipes.")
        .add_asset(recipe_comp_csv)
        .add_asset(recipe_comp_png);
    let root = root.build();
    let root_id = root.rid.clone();

    let mut ra_stats = Asset::with_path("recipe_stats.csv");
    ra_stats.set_kind("recipe-stats");
    let ra_stats = ra_stats.build();
    let ra_stats_id = ra_stats.rid.clone();

    let mut ra = Container::new("Recipe A");
    ra.set_kind("recipe")
        .set_description("Recipe A has 80% H and 20% C")
        .set_metadatum("h", 0.8)
        .set_metadatum("c", 0.2)
        .add_asset(ra_stats);
    let ra = ra.build();
    let ra_id = ra.rid.clone();

    let mut rb_stats = Asset::with_path("recipe_stats.csv");
    rb_stats.set_kind("recipe-stats");
    let rb_stats = rb_stats.build();
    let rb_stats_id = rb_stats.rid.clone();

    let mut rb = Container::new("Recipe B");
    rb.set_kind("recipe")
        .set_description("Recipe B has 50% H and 50% C")
        .set_metadatum("h", 0.5)
        .set_metadatum("c", 0.5)
        .add_asset(rb_stats);
    let rb = rb.build();
    let rb_id = rb.rid.clone();

    let mut graph = ContainerTree::new(root);
    graph.insert(root_id.clone(), ra).unwrap();
    graph.insert(root_id.clone(), rb).unwrap();

    let (tx, rx) = mpsc::unbounded_channel();
    let mut db = Datastore::new(rx);
    thread::spawn(move || db.run());

    let (ctx, crx) = oneshot::channel();
    tx.send(Command::Project(ProjectCommand::Create {
        tx: ctx,
        id: pid.clone(),
        project,
    }))
    .unwrap();
    crx.blocking_recv().unwrap().unwrap();

    let (ctx, crx) = oneshot::channel();
    tx.send(Command::Graph(GraphCommand::Create {
        tx: ctx,
        graph,
        project: pid.clone(),
    }))
    .unwrap();
    crx.blocking_recv().unwrap().unwrap();

    // ************
    // *** test ***
    // ************

    let (ctx, crx) = oneshot::channel();
    tx.send(Command::Query {
        tx: ctx,
        query: "SELECT * FROM container".to_string(),
    })
    .unwrap();
    let mut results = crx.blocking_recv().unwrap().unwrap();
    let results = results.take::<Vec<super::container::Record>>(0).unwrap();
    assert_eq!(results.len(), 3);

    let (ctx, crx) = oneshot::channel();
    tx.send(Command::Query {
        tx: ctx,
        query: "SELECT * FROM asset".to_string(),
    })
    .unwrap();
    let mut results = crx.blocking_recv().unwrap().unwrap();
    let results = results.take::<Vec<super::asset::Record>>(0).unwrap();
    assert_eq!(results.len(), 4);

    let (ctx, crx) = oneshot::channel();
    tx.send(Command::Search {
        tx: ctx,
        query: "recipe".to_string(),
    })
    .unwrap();
    let results = crx.blocking_recv().unwrap().unwrap();
    assert!(results.contains(&root_id));
    assert!(results.contains(&recipe_comp_csv_id));
    assert!(results.contains(&recipe_comp_png_id));
    assert!(results.contains(&ra_id));
    assert!(results.contains(&rb_id));
    assert!(results.contains(&ra_stats_id));
    assert!(results.contains(&rb_stats_id));
    assert_eq!(results.len(), 7);

    let (ctx, crx) = oneshot::channel();
    tx.send(Command::Search {
        tx: ctx,
        query: "recipe stats".to_string(),
    })
    .unwrap();
    let results = crx.blocking_recv().unwrap().unwrap();
    assert!(results.contains(&ra_stats_id));
    assert!(results.contains(&rb_stats_id));
    assert_eq!(results.len(), 2);

    let (ctx, crx) = oneshot::channel();
    tx.send(Command::Search {
        tx: ctx,
        query: "recipe_stats.csv".to_string(),
    })
    .unwrap();
    let results = crx.blocking_recv().unwrap().unwrap();
    assert!(results.contains(&ra_stats_id));
    assert!(results.contains(&rb_stats_id));
    assert_eq!(results.len(), 2);

    let (ctx, crx) = oneshot::channel();
    tx.send(Command::Search {
        tx: ctx,
        query: "image".to_string(),
    })
    .unwrap();
    let results = crx.blocking_recv().unwrap().unwrap();
    assert!(results.contains(&recipe_comp_png_id));
    assert_eq!(results.len(), 1);
}

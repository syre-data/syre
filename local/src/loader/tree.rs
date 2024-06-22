use crate::project::resources::Container;
use rayon::prelude::*;
use std::{
    collections::HashMap,
    fs, io,
    path::Path,
    sync::{Arc, Mutex},
};
use syre_core::{
    graph::{ResourceNode, ResourceTree},
    types::ResourceId,
};

type ContainerTree = ResourceTree<Container>;

pub struct Loader {}
impl Loader {
    /// Load a `Container` tree into a [`ResourceTree`].
    pub fn load(path: impl AsRef<Path>) -> Result<ContainerTree, error::Error> {
        let path = path.as_ref();
        if !path.exists() {
            return Err(error::Error::Root(io::ErrorKind::NotFound));
        }
        if !path.is_dir() {
            return Err(error::Error::Root(io::ErrorKind::NotADirectory));
        }

        let state = rayon::ThreadPoolBuilder::new()
            .build()
            .unwrap()
            .install(move || Self::load_tree(path));

        if state.is_ok() {
            let (nodes, edges, _root) = state.to_parts();
            let edges = edges
                .into_iter()
                .map(|(parent, children)| {
                    let parent = parent.lock().unwrap();
                    let parent = parent.as_ref().unwrap().rid().clone();

                    let children = children
                        .into_iter()
                        .map(|child| {
                            let child = child.lock().unwrap();
                            child.as_ref().unwrap().rid().clone()
                        })
                        .collect();

                    (parent, children)
                })
                .collect();

            let nodes = nodes
                .into_iter()
                .map(|node| {
                    let container = Arc::into_inner(node)
                        .unwrap()
                        .into_inner()
                        .unwrap()
                        .unwrap();

                    let container = ResourceNode::new(container);
                    (container.rid().clone(), container)
                })
                .collect();

            Ok(ContainerTree::from_parts(nodes, edges).unwrap())
        } else {
            Err(error::Error::State(state))
        }
    }

    /// Recursive loader.
    fn load_tree(path: impl AsRef<Path>) -> state::Tree {
        let path = path.as_ref();
        let root = state::Container::load(path);
        let mut graph = state::Tree::new(root);
        let children = fs::read_dir(path)
            .unwrap()
            .into_iter()
            .collect::<Vec<_>>()
            .into_par_iter()
            .map(|entry| Self::load_tree(entry.unwrap().path()))
            .collect::<Vec<_>>();

        let root = graph.root().clone();
        for child in children {
            graph.insert(&root, child);
        }

        graph
    }
}

// pub struct AssetValidator {}
// impl AssetValidator {
//     pub fn validate(graph: &ContainerTree) -> Result<(), HashMap<ResourceId, Vec<AssetFileError>>> {
//         let thread_pool = rayon::ThreadPoolBuilder::new().build().unwrap();
//         return thread_pool.install(move || Self::validate_tree(graph));
//     }

//     fn validate_tree(
//         graph: &ContainerTree,
//     ) -> Result<(), HashMap<ResourceId, Vec<AssetFileError>>> {
//         let errors = graph
//             .nodes()
//             .par_iter()
//             .filter_map(|(rid, container)| {
//                 match ContainerAssetValidator::validate(container.data()) {
//                     Ok(_) => None,
//                     Err(errs) => Some((rid.clone(), errs)),
//                 }
//             })
//             .collect::<HashMap<ResourceId, Vec<AssetFileError>>>();

//         if errors.is_empty() {
//             Ok(())
//         } else {
//             Err(errors)
//         }
//     }
// }

mod state {
    use super::super::container;
    use crate::{
        error::IoSerde,
        project::resources::{container::StoredContainerProperties, Container as LocalContainer},
        types::ContainerSettings,
    };
    use std::{
        fs, io,
        path::PathBuf,
        sync::{Arc, Mutex},
    };
    use syre_core::project::Asset;

    pub type Node = Arc<Mutex<Result<LocalContainer, Container>>>;
    pub type EdgeMap = Vec<(Node, Vec<Node>)>;

    #[derive(Debug)]
    pub struct Container {
        path: PathBuf,

        /// Container data.
        /// If `Err` indicates an issue with the root folder.
        data: Result<ContainerData, io::ErrorKind>,
    }

    impl Container {
        pub fn new(path: impl Into<PathBuf>, data: Result<ContainerData, io::ErrorKind>) -> Self {
            Self {
                path: path.into(),
                data,
            }
        }

        /// # Notes
        /// + Canonicalizes path.
        pub fn load(path: impl Into<PathBuf>) -> Result<LocalContainer, Self> {
            let path = path.into();
            let path = match fs::canonicalize(&path) {
                Ok(path) => path,
                Err(err) => {
                    return Err(Self {
                        path,
                        data: Err(err.kind()),
                    })
                }
            };
            if !path.is_dir() {
                return Err(Self {
                    path,
                    data: Err(io::ErrorKind::NotADirectory),
                });
            }

            container::Loader::load(&path).map_err(|state| Self {
                path: path.into(),
                data: Ok(ContainerData {
                    properties: state.properties,
                    settings: state.settings,
                    assets: state.assets,
                }),
            })
        }

        pub fn path(&self) -> &PathBuf {
            &self.path
        }

        pub fn data(&self) -> &Result<ContainerData, io::ErrorKind> {
            &self.data
        }
    }

    #[derive(Debug)]
    pub struct ContainerData {
        properties: Result<StoredContainerProperties, IoSerde>,
        settings: Result<ContainerSettings, IoSerde>,
        assets: Result<Vec<Asset>, IoSerde>,
    }

    impl ContainerData {
        pub fn is_ok(&self) -> bool {
            self.properties.is_ok() && self.settings.is_ok() && self.assets.is_ok()
        }
    }

    #[derive(Debug)]
    pub struct Tree {
        nodes: Vec<Node>,

        root: Node,

        /// Parent-children relations.
        children: EdgeMap,

        /// Child-parent realtions.
        parents: Vec<(Node, Node)>,
    }

    impl Tree {
        pub fn new(root: Result<LocalContainer, Container>) -> Self {
            let root = Arc::new(Mutex::new(root));
            Self {
                nodes: vec![root.clone()],
                root: root.clone(),
                children: vec![(root, vec![])],
                parents: vec![],
            }
        }

        pub fn root(&self) -> &Node {
            &self.root
        }

        /// # Panics
        /// If the parent does not exist in the graph.
        pub fn insert(&mut self, parent: &Node, graph: Self) {
            let Self {
                nodes,
                root,
                children,
                parents,
            } = graph;

            let parent = self
                .nodes
                .iter()
                .find(|node| Arc::ptr_eq(node, parent))
                .unwrap()
                .clone();

            self.nodes.extend(nodes);
            self.children.extend(children);
            self.parents.extend(parents);
            self.parents.push((root.clone(), parent.clone()));
            self.children
                .iter_mut()
                .find_map(|(p, children)| {
                    if Arc::ptr_eq(p, &parent) {
                        Some(children)
                    } else {
                        None
                    }
                })
                .unwrap()
                .push(root);
        }

        /// Returns `true` if all states are `Ok`,
        /// else `false`.
        pub fn is_ok(&self) -> bool {
            self.nodes.iter().any(|node| node.lock().is_ok())
        }

        /// Breaks graph into (nodes, edges, root)
        pub fn to_parts(self) -> (Vec<Node>, EdgeMap, Node) {
            let Self {
                nodes,
                root,
                children,
                parents: _,
            } = self;

            (nodes, children, root)
        }
    }
}

pub mod error {
    use super::state::Tree;
    use std::io;

    #[derive(Debug)]
    pub enum Error {
        /// The tree's root resource could not be accessed.
        Root(io::ErrorKind),

        /// The tree could not be loaded normally.
        State(Tree),
    }
}

#[cfg(test)]
#[path = "./tree_test.rs"]
mod tree_test;

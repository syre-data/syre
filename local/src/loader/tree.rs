use super::container::{AssetValidator as ContainerAssetValidator, Loader as ContainerLoader};
use super::error::container::AssetFile as AssetFileError;
use super::error::tree::Error;
use crate::project::container;
use crate::project::resources::Container;
use has_id::HasId;
use rayon::prelude::*;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use syre_core::graph::ResourceTree;
use syre_core::types::ResourceId;

type ContainerTree = ResourceTree<Container>;

pub struct Loader {}
impl Loader {
    /// Load a `Container` tree into a [`ResourceTree`].
    pub fn load(path: impl AsRef<Path>) -> Result<ContainerTree, HashMap<PathBuf, Error>> {
        let thread_pool = rayon::ThreadPoolBuilder::new().build().unwrap();
        let path = path.as_ref();
        return thread_pool.install(move || Self::load_tree(path));
    }

    fn load_tree(path: impl AsRef<Path>) -> Result<ContainerTree, HashMap<PathBuf, Error>> {
        let path = path.as_ref();
        let root = match ContainerLoader::load(path) {
            Ok(root) => root,

            Err(err) => {
                let mut errs = HashMap::new();
                errs.insert(path.to_path_buf(), Error::Load(err));
                return Err(errs);
            }
        };

        let rid = root.id().clone();
        let mut graph = ContainerTree::new(root);

        let mut children = Vec::new();
        let entries = match fs::read_dir(path) {
            Ok(entries) => entries,
            Err(err) => {
                let mut errs = HashMap::new();
                errs.insert(path.to_path_buf(), Error::Dir(err.kind()));
                return Err(errs);
            }
        };

        for entry in entries {
            let dir = match entry {
                Ok(entry) => entry,
                Err(err) => {
                    tracing::debug!(?err);
                    panic!("{err:?}");
                }
            };

            if container::path_is_container(&dir.path()) {
                children.push(dir.path());
            }
        }

        let children = children
            .into_par_iter()
            .map(|path| Self::load_tree(path))
            .collect::<Vec<_>>();

        let mut errors = HashMap::new();
        for child in children {
            match child {
                Ok(child) => graph.insert_tree(&rid, child).unwrap(),
                Err(err) => errors.extend(err),
            }
        }

        if errors.is_empty() {
            Ok(graph)
        } else {
            Err(errors)
        }
    }
}

pub struct AssetValidator {}
impl AssetValidator {
    pub fn validate(graph: &ContainerTree) -> Result<(), HashMap<ResourceId, Vec<AssetFileError>>> {
        let thread_pool = rayon::ThreadPoolBuilder::new().build().unwrap();
        return thread_pool.install(move || Self::validate_tree(graph));
    }

    fn validate_tree(
        graph: &ContainerTree,
    ) -> Result<(), HashMap<ResourceId, Vec<AssetFileError>>> {
        let errors = graph
            .nodes()
            .par_iter()
            .filter_map(|(rid, container)| {
                match ContainerAssetValidator::validate(container.data()) {
                    Ok(_) => None,
                    Err(errs) => Some((rid.clone(), errs)),
                }
            })
            .collect::<HashMap<ResourceId, Vec<AssetFileError>>>();

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

pub mod incremental {
    use super::Error;
    use super::{container, ContainerLoader, ContainerTree};
    use has_id::HasId;
    use rayon::prelude::*;
    use std::collections::HashMap;
    use std::fs;
    use std::path::{Path, PathBuf};

    #[derive(Debug)]
    pub struct PartialLoad {
        pub errors: HashMap<PathBuf, Error>,
        pub graph: Option<ContainerTree>,
    }

    /// Loads a graph stopping exploration down a branch when an error is encountered.
    pub struct Loader {}
    impl Loader {
        /// Load a `Container` tree into a [`ResourceTree`].
        pub fn load(path: impl AsRef<Path>) -> Result<ContainerTree, PartialLoad> {
            let thread_pool = rayon::ThreadPoolBuilder::new().build().unwrap();
            let path = path.as_ref();
            thread_pool.install(move || Self::load_tree(path))
        }

        fn load_tree(path: impl AsRef<Path>) -> Result<ContainerTree, PartialLoad> {
            let path = path.as_ref();
            let root = match ContainerLoader::load(path) {
                Ok(root) => root,

                Err(err) => {
                    let mut errors = HashMap::new();
                    errors.insert(path.to_path_buf(), Error::Load(err));
                    return Err(PartialLoad {
                        errors,
                        graph: None,
                    });
                }
            };

            let rid = root.id().clone();
            let mut graph = ContainerTree::new(root);

            let mut children = Vec::new();
            let entries = match fs::read_dir(path) {
                Ok(entries) => entries,
                Err(err) => {
                    let mut errors = HashMap::new();
                    errors.insert(path.to_path_buf(), Error::Dir(err.kind()));
                    return Err(PartialLoad {
                        errors,
                        graph: Some(graph),
                    });
                }
            };

            for entry in entries {
                let dir = match entry {
                    Ok(entry) => entry,
                    Err(err) => {
                        tracing::debug!(?err);
                        panic!("{err:?}");
                    }
                };

                if container::path_is_container(&dir.path()) {
                    children.push(dir.path());
                }
            }

            let children = children
                .into_par_iter()
                .map(|path| Self::load_tree(path))
                .collect::<Vec<_>>();

            let mut errors = HashMap::new();
            for child in children {
                match child {
                    Ok(child) => graph.insert_tree(&rid, child).unwrap(),
                    Err(PartialLoad {
                        graph: child,
                        errors: child_errors,
                    }) => {
                        if let Some(child) = child {
                            graph.insert_tree(&rid, child).unwrap();
                        }

                        errors.extend(child_errors);
                    }
                }
            }

            if errors.is_empty() {
                Ok(graph)
            } else {
                Err(PartialLoad {
                    errors,
                    graph: Some(graph),
                })
            }
        }
    }
}

#[cfg(test)]
#[path = "./tree_test.rs"]
mod tree_test;

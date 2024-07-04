pub use container::{AnalysisAssociation, Asset, State as Container};
pub use graph::State as Graph;
pub use project::State as Project;
pub use workspace::State as Workspace;

pub mod workspace {
    use leptos::*;

    #[derive(Clone)]
    pub struct State {
        pub preview: RwSignal<Preview>,
    }

    impl State {
        pub fn new() -> Self {
            Self {
                preview: RwSignal::new(Preview::default()),
            }
        }
    }

    #[derive(Clone)]
    pub struct Preview {
        pub assets: bool,
        pub analyses: bool,
        pub kind: bool,
        pub description: bool,
        pub tags: bool,
        pub metadata: bool,
    }

    impl Default for Preview {
        fn default() -> Self {
            Self {
                assets: true,
                analyses: false,
                kind: false,
                description: false,
                tags: false,
                metadata: false,
            }
        }
    }
}

pub mod project {
    use chrono::{DateTime, Utc};
    use leptos::*;
    use std::path::PathBuf;
    use syre_core::{
        project::Project as CoreProject,
        types::{ResourceId, ResourceMap, UserId, UserPermissions},
    };
    use syre_local::types::{AnalysisKind, ProjectSettings};
    use syre_local_database as db;

    #[derive(Clone)]
    pub struct State {
        rid: RwSignal<ResourceId>,
        properties: Properties,
        settings: RwSignal<db::state::DataResource<Settings>>,
    }

    impl State {
        /// # Notes
        /// Assumes `properties` is `Ok`.
        pub fn new(data: db::state::ProjectData) -> Self {
            let db::state::DataResource::Ok(properties) = data.properties() else {
                panic!("expected `properties` to be `Ok`");
            };

            Self {
                rid: RwSignal::new(properties.rid().clone()),
                properties: Properties::new(properties.clone()),
                settings: RwSignal::new(
                    data.settings()
                        .map(|settings| Settings::new(settings.clone())),
                ),
            }
        }

        pub fn rid(&self) -> RwSignal<ResourceId> {
            self.rid.clone()
        }

        pub fn properties(&self) -> &Properties {
            &self.properties
        }
    }

    #[derive(Clone)]
    pub struct Properties {
        name: RwSignal<String>,
        description: RwSignal<Option<String>>,
        data_root: RwSignal<PathBuf>,
        analysis_root: RwSignal<Option<PathBuf>>,
        meta_level: RwSignal<u16>,
    }

    impl Properties {
        pub fn new(properties: CoreProject) -> Self {
            let CoreProject {
                name,
                description,
                data_root,
                analysis_root,
                meta_level,
                ..
            } = properties;

            Self {
                name: RwSignal::new(name),
                description: RwSignal::new(description),
                data_root: RwSignal::new(data_root),
                analysis_root: RwSignal::new(analysis_root),
                meta_level: RwSignal::new(meta_level),
            }
        }

        pub fn name(&self) -> RwSignal<String> {
            self.name.clone()
        }

        pub fn description(&self) -> RwSignal<Option<String>> {
            self.description.clone()
        }

        pub fn data_root(&self) -> RwSignal<PathBuf> {
            self.data_root.clone()
        }

        pub fn analysis_root(&self) -> RwSignal<Option<PathBuf>> {
            self.analysis_root.clone()
        }

        pub fn meta_level(&self) -> RwSignal<u16> {
            self.meta_level.clone()
        }
    }

    #[derive(Clone)]
    pub struct Settings {
        created: RwSignal<DateTime<Utc>>,
        creator: RwSignal<Option<UserId>>,
        permissions: RwSignal<ResourceMap<UserPermissions>>,
    }

    impl Settings {
        pub fn new(settings: ProjectSettings) -> Self {
            let ProjectSettings {
                created,
                creator,
                permissions,
                ..
            } = settings;

            Self {
                created: RwSignal::new(created),
                creator: RwSignal::new(creator),
                permissions: RwSignal::new(permissions),
            }
        }
    }

    #[derive(Clone)]
    pub struct Analysis {
        properties: AnalysisKind,
        fs_resource: db::state::FileResource,
    }
}

pub mod graph {
    use super::Container;
    use leptos::*;
    use std::{
        cell::RefCell,
        path::{Component, Path, PathBuf},
        rc::Rc,
    };
    use syre_local_database as db;

    pub type Node = Rc<Container>;

    #[derive(Clone)]
    pub struct State {
        nodes: RwSignal<Vec<Node>>,
        root: Node,
        children: RwSignal<Vec<(Node, RwSignal<Vec<Node>>)>>,
        parents: Rc<RefCell<Vec<(Node, Node)>>>,
    }

    impl State {
        pub fn new(graph: db::state::Graph) -> Self {
            let db::state::Graph { nodes, children } = graph;

            let nodes = nodes
                .into_iter()
                .map(|container| Rc::new(Container::new(container)))
                .collect::<Vec<_>>();

            let root = nodes[0].clone();
            let children = children
                .into_iter()
                .map(|(parent, children)| {
                    let children = children
                        .into_iter()
                        .map(|child| nodes[child].clone())
                        .collect::<Vec<_>>();

                    (nodes[parent].clone(), RwSignal::new(children))
                })
                .collect::<Vec<_>>();

            let parents = children
                .iter()
                .flat_map(|(parent, children)| {
                    children.with_untracked(|children| {
                        children
                            .iter()
                            .map(|child| (parent.clone(), child.clone()))
                            .collect::<Vec<_>>()
                    })
                })
                .collect();

            Self {
                nodes: RwSignal::new(nodes),
                root,
                children: RwSignal::new(children),
                parents: Rc::new(RefCell::new(parents)),
            }
        }

        pub fn nodes(&self) -> RwSignal<Vec<Node>> {
            self.nodes.clone()
        }

        pub fn root(&self) -> &Node {
            &self.root
        }

        pub fn children(&self, parent: &Node) -> Option<RwSignal<Vec<Node>>> {
            self.children.with_untracked(|children| {
                children.iter().find_map(|(p, children)| {
                    if Rc::ptr_eq(p, parent) {
                        Some(children.clone())
                    } else {
                        None
                    }
                })
            })
        }

        /// # Returns
        /// The child's parent if it exists in the map, otherwise `None`.
        ///
        /// # Notes
        /// + `None` is returned in two cases:
        /// 1. The child node does not exist in the graph.
        /// 2. The child node is the graph root.
        /// It is left for the caller to distinguish between tese cases if needed.
        pub fn parent(&self, child: &Node) -> Option<Node> {
            self.parents.borrow().iter().find_map(|(c, parent)| {
                if Rc::ptr_eq(c, child) {
                    Some(parent.clone())
                } else {
                    None
                }
            })
        }

        /// # Returns
        /// List of ancestors, in order, starting with the given node until the root.
        /// If the given node is not in the graph, an empty `Vec` is returned.
        pub fn ancestors(&self, root: &Node) -> Vec<Node> {
            if Rc::ptr_eq(&self.root, root) {
                return vec![root.clone()];
            }

            let Some(parent) = self.parent(root) else {
                return vec![];
            };

            let mut ancestors = self.ancestors(&parent);
            ancestors.insert(0, root.clone());
            ancestors
        }

        /// Get the absolute path to the container from the root node.
        /// i.e. The root node has path `/`.
        pub fn path(&self, target: &Node) -> Option<PathBuf> {
            const SEPARATOR: &str = "/";

            let ancestors = self.ancestors(target);
            if ancestors.is_empty() {
                return None;
            }

            let path = ancestors
                .iter()
                .rev()
                .skip(1)
                .map(|ancestor| ancestor.name().get().to_string_lossy().to_string())
                .collect::<Vec<_>>()
                .join(SEPARATOR);

            Some(PathBuf::from(SEPARATOR).join(path))
        }

        /// # Returns
        /// If the graph contains the given node.
        pub fn contains(&self, node: &Node) -> bool {
            self.nodes
                .with_untracked(|nodes| nodes.iter().any(|existing| Node::ptr_eq(existing, node)))
        }

        /// Finds a node by its path.
        /// Path should be absolute from the graph root.
        /// i.e. The root path refers to the root node.
        pub fn find(&self, path: impl AsRef<Path>) -> Result<Option<Node>, error::InvalidPath> {
            let mut components = path.as_ref().components();
            let Some(Component::RootDir) = components.next() else {
                return Err(error::InvalidPath);
            };

            let mut node = self.root.clone();
            for component in components {
                match component {
                    Component::Prefix(_)
                    | Component::RootDir
                    | Component::CurDir
                    | Component::ParentDir => return Err(error::InvalidPath),
                    Component::Normal(name) => {
                        let Some(child) = self.children(&node).unwrap().with(|children| {
                            children.iter().find_map(|child| {
                                child.name().with(|child_name| {
                                    if child_name == name {
                                        Some(child.clone())
                                    } else {
                                        None
                                    }
                                })
                            })
                        }) else {
                            return Ok(None);
                        };

                        node = child;
                    }
                }
            }

            Ok(Some(node))
        }
    }

    impl State {
        /// Inserts a subgraph at the indicated path.
        pub fn insert(&self, parent: impl AsRef<Path>, graph: Self) -> Result<(), error::Insert> {
            let Self {
                nodes,
                root,
                children,
                parents,
            } = graph;

            if let Some(node) = nodes.with_untracked(|nodes| {
                nodes.iter().find_map(|node| {
                    if self.contains(node) {
                        Some(node.clone())
                    } else {
                        None
                    }
                })
            }) {
                return Err(error::Insert::NodeAlreadyExists(node.clone()));
            }

            let Some(parent) = self.find(parent)? else {
                return Err(error::Insert::ParentNotFound);
            };

            self.nodes
                .update(|current| current.extend(nodes.get_untracked()));

            // NB: Order of adding children is important for recursion
            // in graph view.
            // Can not combine two operations due to borrow error.
            self.children
                .update(|current| current.extend(children.get_untracked()));

            self.children.with_untracked(|current| {
                current
                    .iter()
                    .find_map(|(p, children)| {
                        if Node::ptr_eq(p, &parent) {
                            Some(children)
                        } else {
                            None
                        }
                    })
                    .unwrap()
                    .update(|children| {
                        children.push(root.clone());
                    });
            });

            self.parents
                .borrow_mut()
                .extend(Rc::into_inner(parents).unwrap().into_inner());

            self.parents.borrow_mut().push((root, parent));

            Ok(())
        }
    }

    pub mod error {
        use super::Node;

        #[derive(Debug)]
        pub struct InvalidPath;

        #[derive(Debug)]
        pub enum Insert {
            ParentNotFound,
            NodeAlreadyExists(Node),
            InvalidPath,
        }

        impl From<InvalidPath> for Insert {
            fn from(_: InvalidPath) -> Self {
                Self::InvalidPath
            }
        }
    }
}

pub mod container {
    use chrono::*;
    use leptos::*;
    use std::{ffi::OsString, path::PathBuf};
    use syre_core::{
        project::ContainerProperties,
        types::{Creator, ResourceId, ResourceMap, UserId, UserPermissions},
    };
    use syre_local_database as db;

    pub type PropertiesState = db::state::DataResource<Properties>;
    pub type AnalysesState = db::state::DataResource<RwSignal<Vec<AnalysisAssociation>>>;
    pub type AssetsState = db::state::DataResource<RwSignal<Vec<Asset>>>;
    pub type SettingsState = db::state::DataResource<Settings>;
    pub type Metadata = Vec<(String, RwSignal<serde_json::Value>)>;

    #[derive(Clone, Debug)]
    pub struct State {
        /// Folder name.
        name: RwSignal<OsString>,
        properties: RwSignal<PropertiesState>,
        analyses: RwSignal<AnalysesState>,
        assets: RwSignal<AssetsState>,
        settings: RwSignal<SettingsState>,
    }

    impl State {
        pub fn new(container: db::state::Container) -> Self {
            let properties = container.properties().map(|properties| {
                let rid = container.rid().cloned().unwrap();
                Properties::new(rid, properties.clone())
            });

            let analyses = container.analyses().map(|associations| {
                RwSignal::new(
                    associations
                        .iter()
                        .map(|association| AnalysisAssociation::new(association.clone()))
                        .collect(),
                )
            });

            let assets = container.assets().map(|assets| {
                let assets = assets
                    .iter()
                    .map(|asset| Asset::new(asset.clone()))
                    .collect();

                RwSignal::new(assets)
            });

            let settings = container
                .settings()
                .map(|settings| Settings::new(settings.clone()));

            Self {
                name: RwSignal::new(container.name().clone()),
                properties: RwSignal::new(properties),
                analyses: RwSignal::new(analyses),
                assets: RwSignal::new(assets),
                settings: RwSignal::new(settings),
            }
        }

        pub fn name(&self) -> RwSignal<OsString> {
            self.name.clone()
        }

        pub fn properties(&self) -> RwSignal<PropertiesState> {
            self.properties.clone()
        }

        pub fn analyses(&self) -> RwSignal<AnalysesState> {
            self.analyses.clone()
        }

        pub fn assets(&self) -> RwSignal<AssetsState> {
            self.assets.clone()
        }

        pub fn settings(&self) -> RwSignal<SettingsState> {
            self.settings.clone()
        }
    }

    #[derive(Clone)]
    pub struct Properties {
        rid: RwSignal<ResourceId>,
        name: RwSignal<String>,
        kind: RwSignal<Option<String>>,
        description: RwSignal<Option<String>>,
        tags: RwSignal<Vec<String>>,
        metadata: RwSignal<Metadata>,
    }

    impl Properties {
        pub fn new(rid: ResourceId, properties: ContainerProperties) -> Self {
            let ContainerProperties {
                name,
                kind,
                description,
                tags,
                metadata,
            } = properties;

            let metadata = metadata
                .into_iter()
                .map(|(key, value)| (key, RwSignal::new(value)))
                .collect();

            Self {
                rid: RwSignal::new(rid),
                name: RwSignal::new(name),
                kind: RwSignal::new(kind),
                description: RwSignal::new(description),
                tags: RwSignal::new(tags),
                metadata: RwSignal::new(metadata),
            }
        }
        pub fn rid(&self) -> RwSignal<ResourceId> {
            self.rid.clone()
        }

        pub fn name(&self) -> RwSignal<String> {
            self.name.clone()
        }

        pub fn kind(&self) -> RwSignal<Option<String>> {
            self.kind.clone()
        }

        pub fn description(&self) -> RwSignal<Option<String>> {
            self.description.clone()
        }

        pub fn tags(&self) -> RwSignal<Vec<String>> {
            self.tags.clone()
        }

        pub fn metadata(&self) -> RwSignal<Metadata> {
            self.metadata.clone()
        }
    }

    #[derive(Clone)]
    pub struct AnalysisAssociation {
        analysis: ResourceId,
        autorun: RwSignal<bool>,
        priority: RwSignal<i32>,
    }

    impl AnalysisAssociation {
        pub fn new(association: syre_core::project::AnalysisAssociation) -> Self {
            let analysis = association.analysis().clone();
            let syre_core::project::AnalysisAssociation {
                autorun, priority, ..
            } = association;

            Self {
                analysis,
                autorun: RwSignal::new(autorun),
                priority: RwSignal::new(priority),
            }
        }

        pub fn analysis(&self) -> &ResourceId {
            &self.analysis
        }

        pub fn autorun(&self) -> RwSignal<bool> {
            self.autorun.clone()
        }

        pub fn priority(&self) -> RwSignal<i32> {
            self.priority.clone()
        }
    }

    #[derive(Clone)]
    pub struct Asset {
        rid: RwSignal<ResourceId>,
        name: RwSignal<Option<String>>,
        kind: RwSignal<Option<String>>,
        description: RwSignal<Option<String>>,
        tags: RwSignal<Vec<String>>,
        metadata: RwSignal<Metadata>,
        path: RwSignal<PathBuf>,
        fs_resource: RwSignal<db::state::FileResource>,
        created: RwSignal<DateTime<Utc>>,
        creator: RwSignal<Creator>,
    }

    impl Asset {
        pub fn new(asset: db::state::Asset) -> Self {
            let fs_resource = if asset.is_present() {
                db::state::FileResource::Present
            } else {
                db::state::FileResource::Absent
            };

            let metadata = (*asset)
                .properties
                .metadata
                .iter()
                .map(|(key, value)| (key.clone(), RwSignal::new(value.clone())))
                .collect();

            Self {
                rid: RwSignal::new(asset.rid().clone()),
                name: RwSignal::new((*asset).properties.name.clone()),
                kind: RwSignal::new((*asset).properties.kind.clone()),
                description: RwSignal::new((*asset).properties.description.clone()),
                tags: RwSignal::new((*asset).properties.tags.clone()),
                metadata: RwSignal::new(metadata),
                path: RwSignal::new((*asset).path.clone()),
                fs_resource: RwSignal::new(fs_resource),
                created: RwSignal::new((*asset).properties.created().clone()),
                creator: RwSignal::new((*asset).properties.creator.clone()),
            }
        }

        pub fn rid(&self) -> RwSignal<ResourceId> {
            self.rid.clone()
        }

        pub fn name(&self) -> RwSignal<Option<String>> {
            self.name.clone()
        }

        pub fn kind(&self) -> RwSignal<Option<String>> {
            self.kind.clone()
        }

        pub fn description(&self) -> RwSignal<Option<String>> {
            self.description.clone()
        }

        pub fn tags(&self) -> RwSignal<Vec<String>> {
            self.tags.clone()
        }

        pub fn metadata(&self) -> RwSignal<Metadata> {
            self.metadata.clone()
        }

        pub fn path(&self) -> RwSignal<PathBuf> {
            self.path.clone()
        }

        pub fn fs_resource(&self) -> RwSignal<db::state::FileResource> {
            self.fs_resource.clone()
        }

        pub fn created(&self) -> RwSignal<DateTime<Utc>> {
            self.created.clone()
        }

        pub fn creator(&self) -> RwSignal<Creator> {
            self.creator.clone()
        }
    }

    #[derive(Clone)]
    pub struct Settings {
        creator: RwSignal<Option<UserId>>,
        created: RwSignal<DateTime<Utc>>,
        permissions: RwSignal<ResourceMap<UserPermissions>>,
    }

    impl Settings {
        pub fn new(settings: syre_local::types::ContainerSettings) -> Self {
            let syre_local::types::ContainerSettings {
                creator,
                created,
                permissions,
            } = settings;

            Self {
                creator: RwSignal::new(creator),
                created: RwSignal::new(created),
                permissions: RwSignal::new(permissions),
            }
        }
    }
}

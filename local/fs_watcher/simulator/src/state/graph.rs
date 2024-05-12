use super::HasPath;
use has_id::HasId;
use std::{
    cell::RefCell,
    path::PathBuf,
    rc::{Rc, Weak},
};

pub type Node<T> = Rc<RefCell<T>>;
pub type NodeRef<T> = Weak<RefCell<T>>;

#[derive(Debug)]
pub struct Tree<D> {
    root: NodeRef<D>,
    nodes: Vec<Node<D>>,
    children: Vec<(NodeRef<D>, Vec<NodeRef<D>>)>,
    parents: Vec<(NodeRef<D>, NodeRef<D>)>,
}

impl<D> Tree<D> {
    pub fn new(root: D) -> Self {
        let root = Rc::new(RefCell::new(root));
        let children = vec![(Rc::downgrade(&root), vec![])];

        Self {
            root: Rc::downgrade(&root),
            nodes: vec![root],
            children,
            parents: vec![],
        }
    }

    pub fn root(&self) -> Node<D> {
        self.root.upgrade().unwrap()
    }

    pub fn nodes(&self) -> &Vec<Node<D>> {
        &self.nodes
    }

    pub fn insert(&mut self, node: D, parent: &Node<D>) -> Result<(), error::Insert<D>> {
        let parent = self
            .nodes
            .iter()
            .find(|node| Rc::ptr_eq(&node, &parent))
            .ok_or(error::Insert::InvalidParent)?;

        let (_, children) = self
            .children
            .iter_mut()
            .find(|(p, _)| {
                if let Some(p) = p.upgrade() {
                    Rc::ptr_eq(&p, parent)
                } else {
                    false
                }
            })
            .unwrap();

        let node = Rc::new(RefCell::new(node));
        children.push(Rc::downgrade(&node));
        self.children.push((Rc::downgrade(&node), vec![]));
        self.parents
            .push((Rc::downgrade(&node), Rc::downgrade(parent)));

        self.nodes.push(node);
        Ok(())
    }

    /// Inserts a tree at the given node.
    pub fn insert_tree(&mut self, tree: Self, parent: &Node<D>) -> Result<(), error::Insert<D>> {
        if !self.contains(parent) {
            return Err(error::Insert::InvalidParent);
        }

        for node in tree.nodes() {
            if self.nodes.iter().any(|n| Rc::ptr_eq(n, node)) {
                return Err(error::Insert::AlreadyContains(Rc::downgrade(node)));
            }
        }

        let Self {
            root,
            nodes,
            children,
            parents,
        } = tree;

        let parent = Rc::downgrade(parent);
        self.nodes.extend(nodes);
        self.children.extend(children);
        self.parents.extend(parents);
        self.parents.push((root, parent.clone()));
        let (_, children) = self
            .children
            .iter_mut()
            .find(|(p, _)| parent.ptr_eq(p))
            .unwrap();

        children.push(parent);
        Ok(())
    }

    /// Removes a sub tree.
    pub fn remove(&mut self, root: &Node<D>) -> Option<Self> {
        let descendants = self
            .descendants(root)
            .into_iter()
            .map(|descendant| Rc::downgrade(&descendant))
            .collect::<Vec<_>>();

        if descendants.is_empty() {
            return None;
        }

        let mut nodes = Vec::with_capacity(descendants.len());
        let mut children = Vec::with_capacity(descendants.len());
        let mut parents = Vec::with_capacity(descendants.len());
        for descendant in descendants {
            let index = self
                .parents
                .iter()
                .position(|(child, _)| descendant.ptr_eq(child))
                .unwrap();

            parents.push(self.parents.swap_remove(index));

            let index = self
                .children
                .iter()
                .position(|(parent, _)| descendant.ptr_eq(parent))
                .unwrap();

            children.push(self.children.swap_remove(index));

            let index = self
                .nodes
                .iter()
                .position(|n| descendant.ptr_eq(&Rc::downgrade(n)))
                .unwrap();

            nodes.push(self.nodes.swap_remove(index));
        }

        // remove root's parent
        let root = Rc::downgrade(root);
        let index = parents
            .iter()
            .position(|(child, _)| root.ptr_eq(child))
            .unwrap();

        parents.swap_remove(index);

        Some(Self {
            root,
            nodes,
            children,
            parents,
        })
    }

    pub fn parent(&self, child: &Node<D>) -> Option<Node<D>> {
        let child = Rc::downgrade(child);
        let (_, parent) = self.parents.iter().find(|(c, _)| child.ptr_eq(c))?;
        Some(parent.upgrade().unwrap())
    }

    pub fn children(&self, parent: &Node<D>) -> Option<Vec<Node<D>>> {
        let parent = Rc::downgrade(parent);
        let (_, children) = self.children.iter().find(|(p, _)| parent.ptr_eq(p))?;
        let children = children
            .into_iter()
            .map(|child| child.upgrade().unwrap())
            .collect();

        Some(children)
    }

    /// Gets descendants of the given root node.
    ///
    /// # Returns
    /// + List of all descendants, including self.
    /// + Empty if root is not in the graph.
    pub fn descendants(&self, root: &Node<D>) -> Vec<Node<D>> {
        if !self.contains(root) {
            return vec![];
        }

        let mut descendants = self
            .children(root)
            .unwrap()
            .clone()
            .iter()
            .flat_map(|child| self.descendants(child))
            .collect::<Vec<_>>();

        descendants.push(root.clone());
        descendants
    }

    /// Get the ancestors or the given node.
    ///
    /// # Returns
    /// + Ordered list of ancestors, beginning with the given node.
    /// + Empty if node does not exist in the graph.
    pub fn ancestors(&self, child: &Node<D>) -> Vec<&Node<D>> {
        let mut child = Rc::downgrade(child);
        let this = child.upgrade().unwrap();
        let Some(this) = self.nodes.iter().find(|n| Rc::ptr_eq(&this, n)) else {
            return vec![];
        };

        let mut ancestors = vec![];
        ancestors.push(this);
        while let Some((c, parent)) = self.parents.iter().find(|(c, _)| child.ptr_eq(c)) {
            child = parent.clone();
            let parent = parent.upgrade().unwrap();
            let parent = self.nodes.iter().find(|n| Rc::ptr_eq(&parent, n)).unwrap();
            ancestors.push(parent);
        }

        ancestors
    }

    pub fn contains(&self, node: &Node<D>) -> bool {
        self.nodes.iter().find(|n| Rc::ptr_eq(&node, n)).is_some()
    }
}

impl<D> Tree<D>
where
    D: Clone,
{
    /// Duplicates the tree.
    pub fn duplicate(&self) -> Self {
        let mut node_map = vec![];
        let nodes = self
            .nodes
            .iter()
            .map(|node| {
                let node_clone = Rc::new(RefCell::new(node.borrow().clone()));
                node_map.push((node.clone(), node_clone.clone()));
                node_clone
            })
            .collect();

        let root = node_map
            .iter()
            .find_map(|(from, to)| {
                if Rc::ptr_eq(from, &self.root.upgrade().unwrap()) {
                    return Some(Rc::downgrade(to));
                }

                None
            })
            .unwrap();

        let children = self
            .children
            .iter()
            .map(|(parent, children)| {
                let parent = parent.upgrade().unwrap();
                let children = children
                    .iter()
                    .map(|child| {
                        let child = child.upgrade().unwrap();

                        node_map
                            .iter()
                            .find_map(|(from, to)| {
                                if Rc::ptr_eq(from, &child) {
                                    return Some(Rc::downgrade(to));
                                }

                                None
                            })
                            .unwrap()
                    })
                    .collect();

                let parent = node_map
                    .iter()
                    .find_map(|(from, to)| {
                        if Rc::ptr_eq(from, &parent) {
                            return Some(Rc::downgrade(to));
                        }

                        None
                    })
                    .unwrap();

                (parent, children)
            })
            .collect();

        let parents = self
            .parents
            .iter()
            .map(|(child, parent)| {
                let child = child.upgrade().unwrap();
                let parent = parent.upgrade().unwrap();

                let child = node_map
                    .iter()
                    .find_map(|(from, to)| {
                        if Rc::ptr_eq(from, &child) {
                            return Some(Rc::downgrade(to));
                        }

                        None
                    })
                    .unwrap();

                let parent = node_map
                    .iter()
                    .find_map(|(from, to)| {
                        if Rc::ptr_eq(from, &parent) {
                            return Some(Rc::downgrade(to));
                        }

                        None
                    })
                    .unwrap();

                (child, parent)
            })
            .collect();

        Self {
            nodes,
            root,
            children,
            parents,
        }
    }
}

impl<D> Tree<D>
where
    D: HasId,
{
    pub fn find(&self, id: &<D as HasId>::Id) -> Option<&Node<D>> {
        self.nodes
            .iter()
            .find(|node| <D as HasId>::id(&node.borrow()) == id)
    }
}

impl<D> Tree<D>
where
    D: HasPath,
{
    /// Get the path from the root of the graph to the given node.
    pub fn path(&self, node: &Node<D>) -> Option<PathBuf> {
        let mut ancestors = self.ancestors(node);
        if ancestors.is_empty() {
            return None;
        }

        ancestors.reverse();
        Some(
            ancestors
                .iter()
                .fold(PathBuf::new(), |path, node| path.join(node.borrow().path())),
        )
    }

    pub fn paths(&self, root: &Node<D>) -> Option<Vec<PathBuf>> {
        fn inner<D>(graph: &Tree<D>, root: &Node<D>) -> Vec<PathBuf>
        where
            D: HasPath,
        {
            let root_path = root.borrow().path().clone();
            let mut paths = graph
                .children(root)
                .unwrap()
                .iter()
                .flat_map(|child| {
                    inner(graph, child)
                        .into_iter()
                        .map(|path| root_path.join(path))
                })
                .collect::<Vec<_>>();

            paths.push(root_path);
            paths
        }

        let root_path = self.path(root)?;
        let root_path = root_path.parent().unwrap();
        Some(
            inner(self, root)
                .into_iter()
                .map(|path| root_path.join(path))
                .collect(),
        )
    }

    pub fn all_paths(&self) -> Vec<PathBuf> {
        self.paths(&self.root()).unwrap()
    }
}

pub mod error {
    use super::*;

    #[derive(Debug)]
    pub enum Insert<D> {
        /// The given parent does not exist in the tree.
        InvalidParent,

        /// The given node is already in the tree.
        AlreadyContains(NodeRef<D>),
    }
}

#[cfg(test)]
#[path = "graph_test.rs"]
mod graph_test;

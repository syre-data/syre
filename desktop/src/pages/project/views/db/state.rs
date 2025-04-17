pub mod data {
    use crate::pages::project::state;
    use leptos::prelude::*;
    use std::{assert_matches::assert_matches, path::PathBuf};
    use syre_project_watcher as db;

    #[derive(Clone, Copy, Default, Debug)]
    pub enum SortDirection {
        /// Ascending.
        #[default]
        Asc,
        /// Descending.
        Des,
    }

    #[derive(Clone, Copy, Default, PartialEq, Debug)]
    pub enum SortField {
        #[default]
        Path,
        File,
        Name,
        Kind,
    }

    #[derive(Clone)]
    pub struct Sort {
        field: SortField,
        direction: SortDirection,
    }

    impl Sort {
        pub fn field(&self) -> SortField {
            self.field
        }

        pub fn direction(&self) -> SortDirection {
            self.direction
        }
    }

    impl Default for Sort {
        fn default() -> Self {
            Self {
                field: Default::default(),
                direction: Default::default(),
            }
        }
    }

    #[derive(Clone)]
    pub struct Datum {
        container: RwSignal<state::graph::Node>,
        path: RwSignal<PathBuf>,
        asset: state::Asset,
    }

    impl Datum {
        /// Container path.
        pub fn path(&self) -> ReadSignal<PathBuf> {
            self.path.read_only()
        }

        pub fn asset(&self) -> &state::Asset {
            &self.asset
        }
    }

    #[derive(Clone)]
    pub struct State {
        /// Graph state.
        graph: state::Graph,

        /// Containers' assets state.
        node_states: RwSignal<Vec<ReadSignal<state::container::AssetsState>>>,

        /// Containers' assets' states.
        node_assets: RwSignal<Vec<ReadSignal<Vec<state::Asset>>>>,

        /// Individual assets.
        data: RwSignal<Vec<Datum>>,

        sort: RwSignal<Sort>,
    }

    impl State {
        pub fn from(graph: state::Graph) -> Self {
            let node_states = graph
                .nodes()
                .read_untracked()
                .iter()
                .map(|node| node.assets().read_only())
                .collect::<Vec<_>>();

            let node_assets = node_states
                .iter()
                .filter_map(|state| {
                    if let db::state::DataResource::Ok(assets) = state.get_untracked() {
                        Some(assets.read_only())
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>();

            let data = graph
                .nodes()
                .read_untracked()
                .iter()
                .map(|node| (node, node.assets().read_only()))
                .filter_map(|(node, assets)| {
                    if let db::state::DataResource::Ok(assets) = assets.get_untracked() {
                        Some((node, assets.read_only()))
                    } else {
                        None
                    }
                })
                .flat_map(|(node, assets)| {
                    let path = graph.path(node).expect(&format!(
                        "container path of `{:?}` should exist",
                        node.name().get_untracked(),
                    ));

                    assets
                        .get_untracked()
                        .into_iter()
                        .map(move |asset| (node.clone(), path.clone(), asset))
                })
                .map(|(node, path, asset)| Datum {
                    container: RwSignal::new(node),
                    path: RwSignal::new(path),
                    asset,
                })
                .collect::<Vec<_>>();

            let node_states = RwSignal::new(node_states);
            let node_assets = RwSignal::new(node_assets);
            let data = RwSignal::new(data);
            let sort = RwSignal::new(Sort::default());

            let _ = Effect::watch(
                graph.nodes(),
                move |graph, _, _| {
                    let update = graph
                        .iter()
                        .map(|node| node.assets().read_only())
                        .collect::<Vec<_>>();

                    node_states.set(update);
                },
                false,
            );

            let _ = Effect::watch(
                node_states,
                move |node_states, _, _| {
                    let update = node_states
                        .iter()
                        .filter_map(|node| {
                            if let db::state::DataResource::Ok(assets) = node.get_untracked() {
                                Some(assets.read_only())
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<_>>();

                    node_assets.set(update);
                },
                false,
            );

            let _ = Effect::watch(
                node_assets,
                move |node_assets, _, _| {
                    let update = node_assets
                        .iter()
                        .flat_map(|assets| assets.get_untracked())
                        .collect::<Vec<_>>();

                    todo!();
                    // data.set(update);
                },
                false,
            );

            let _ = Effect::watch(
                sort,
                move |sort, prev, _| {
                    let field = sort.field();
                    let needs_sorting = if let Some(prev) = prev {
                        field != prev.field()
                    } else {
                        true
                    };

                    if needs_sorting {
                        assert_matches!(sort.direction(), SortDirection::Asc);
                        match field {
                            SortField::Path => data.write().sort_by_key(|datum| {
                                datum
                                    .path()
                                    .get_untracked()
                                    .to_string_lossy()
                                    .to_lowercase()
                            }),
                            SortField::File => data.write().sort_by_key(|datum| {
                                datum
                                    .asset()
                                    .path()
                                    .get_untracked()
                                    .to_string_lossy()
                                    .to_lowercase()
                            }),
                            SortField::Name => {
                                data.write().sort_by_key(|datum| {
                                    datum
                                        .asset()
                                        .name()
                                        .get_untracked()
                                        .map(|value| value.to_lowercase())
                                });
                            }
                            SortField::Kind => {
                                data.write().sort_by_key(|datum| {
                                    datum
                                        .asset()
                                        .kind()
                                        .get_untracked()
                                        .map(|value| value.to_lowercase())
                                });
                            }
                        }
                    } else {
                        data.write().reverse();
                    }
                },
                false,
            );

            Self {
                graph,
                node_states,
                node_assets,
                data,
                sort,
            }
        }

        pub fn data(&self) -> ReadSignal<Vec<Datum>> {
            self.data.read_only()
        }

        pub fn sort(&self) -> ReadSignal<Sort> {
            self.sort.read_only()
        }

        pub fn sort_by(&self, field: SortField) {
            if field != self.sort.read_untracked().field {
                self.sort.set(Sort {
                    field,
                    direction: SortDirection::default(),
                });
            }
        }

        pub fn toggle_sort_direction(&self) {
            let direction = match self.sort.read_untracked().direction() {
                SortDirection::Asc => SortDirection::Des,
                SortDirection::Des => SortDirection::Asc,
            };

            let mut sort = self.sort().get_untracked();
            sort.direction = direction;
            self.sort.set(sort);
        }
    }
}

pub mod display {
    use leptos::{html, prelude::*};

    #[derive(Clone)]
    pub struct Column {
        node_ref: NodeRef<html::Col>,
        visible: RwSignal<bool>,
    }

    impl Column {
        pub fn new() -> Self {
            Self {
                node_ref: NodeRef::<html::Col>::new(),
                visible: RwSignal::new(true),
            }
        }

        pub fn node_ref(&self) -> NodeRef<html::Col> {
            self.node_ref
        }

        pub fn visible(&self) -> RwSignal<bool> {
            self.visible
        }
    }

    #[derive(Clone)]
    pub struct Columns {
        path: Column,
        file: Column,
        name: Column,
        kind: Column,
        description: Column,
        tags: Column,
        metadata: Vec<(String, Column)>,
    }

    impl Columns {
        pub fn new() -> Self {
            Self {
                path: Column::new(),
                file: Column::new(),
                name: Column::new(),
                kind: Column::new(),
                description: Column::new(),
                tags: Column::new(),
                metadata: vec![],
            }
        }

        pub fn path(&self) -> &Column {
            &self.path
        }

        pub fn file(&self) -> &Column {
            &self.file
        }

        pub fn name(&self) -> &Column {
            &self.name
        }

        pub fn kind(&self) -> &Column {
            &self.kind
        }

        pub fn description(&self) -> &Column {
            &self.description
        }

        pub fn tags(&self) -> &Column {
            &self.tags
        }

        /// Set all visibilities to `true`.
        pub fn clear(&self) {
            self.path.visible().set(true);
            self.file.visible().set(true);
            self.name.visible().set(true);
            self.kind.visible().set(true);
            self.description.visible().set(true);
            self.tags.visible().set(true);
            for (_, col) in &self.metadata {
                col.visible().set(true);
            }
        }
    }

    #[derive(Clone)]
    pub struct State {
        columns: Columns,
    }

    impl State {
        pub fn new() -> Self {
            Self {
                columns: Columns::new(),
            }
        }
    }

    impl State {
        pub fn columns(&self) -> &Columns {
            &self.columns
        }
    }
}

pub mod data {
    use leptos::prelude::*;
    use std::path;
    use std::{assert_matches::assert_matches, path::PathBuf};
    use syre_core as core;
    use syre_desktop_ui_lib as ui_lib;
    use syre_project_watcher as db;

    #[derive(Clone)]
    pub struct Datum {
        ancestors: RwSignal<Vec<ui_lib::state::graph::Node>>,
        asset: ui_lib::state::Asset,
    }

    impl Datum {
        pub fn asset(&self) -> &ui_lib::state::Asset {
            &self.asset
        }

        /// Container path.
        pub fn path(&self) -> Signal<PathBuf> {
            Signal::derive({
                let ancestors = self.ancestors.read_only();
                move || {
                    assert!(!ancestors.read().is_empty());

                    let path = ancestors
                        .read()
                        .iter()
                        .rev()
                        .skip(1)
                        .map(|ancestor| ancestor.name().get())
                        .collect::<path::PathBuf>();

                    std::iter::once(path::Component::RootDir)
                        .chain(path.components())
                        .collect::<path::PathBuf>()
                }
            })
        }

        /// Metadata values including inheritance.
        pub fn metadata(&self) -> Signal<Vec<(String, ReadSignal<core::types::Value>)>> {
            Signal::derive({
                let ancestors = self.ancestors.read_only();
                let asset_md = self.asset.metadata().read_only();
                move || {
                    let mut metadata = vec![];
                    ancestors
                        .read()
                        .iter()
                        .map(|ancestor| ancestor.properties().read_only())
                        .filter_map(|properties| {
                            properties.with(|properties| {
                                if let db::state::DataResource::Ok(properties) = properties {
                                    Some(properties.metadata().read_only())
                                } else {
                                    None
                                }
                            })
                        })
                        .rev()
                        .for_each(|md| {
                            for (md_key, md_value) in md.get() {
                                if let Some((_, value)) =
                                    metadata.iter_mut().find(|(key, _)| *key == md_key)
                                {
                                    *value = md_value.read_only();
                                } else {
                                    metadata.push((md_key.clone(), md_value.read_only()))
                                }
                            }
                        });

                    for (md_key, md_value) in asset_md.read().iter() {
                        if let Some((_, value)) = metadata.iter_mut().find(|(key, _)| key == md_key)
                        {
                            *value = md_value.read_only();
                        } else {
                            metadata.push((md_key.clone(), md_value.read_only()))
                        }
                    }

                    metadata
                }
            })
        }
    }

    #[derive(Clone)]
    pub struct State {
        /// Graph state.
        graph: ui_lib::state::Graph,

        /// Containers' assets state.
        node_states: RwSignal<Vec<ReadSignal<ui_lib::state::container::AssetsState>>>,

        /// Containers' assets' states.
        node_assets: RwSignal<Vec<ReadSignal<Vec<ui_lib::state::Asset>>>>,

        /// Individual assets.
        data: RwSignal<Vec<Datum>>,
    }

    impl State {
        pub fn from(graph: ui_lib::state::Graph) -> Self {
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
                    assets
                        .get_untracked()
                        .into_iter()
                        .map(move |asset| (node.clone(), asset))
                })
                .map(|(node, asset)| {
                    let ancestors = graph.ancestors(&node);
                    assert!(!ancestors.is_empty());
                    Datum {
                        ancestors: RwSignal::new(ancestors),
                        asset,
                    }
                })
                .collect::<Vec<_>>();

            let node_states = RwSignal::new(node_states);
            let node_assets = RwSignal::new(node_assets);
            let data = RwSignal::new(data);

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

            Self {
                graph,
                node_states,
                node_assets,
                data,
            }
        }

        pub fn data(&self) -> ReadSignal<Vec<Datum>> {
            self.data.read_only()
        }

        pub fn metadata_keys(&self) -> Signal<Vec<String>> {
            let nodes = self.graph.nodes().read_only();
            Signal::derive(move || {
                // TODO: Memoize.
                let mut keys = vec![];
                nodes
                    .read()
                    .iter()
                    .filter_map(|node| {
                        node.properties().with(|properties| {
                            if let db::state::DataResource::Ok(properties) = properties {
                                Some(properties.metadata().read_only())
                            } else {
                                None
                            }
                        })
                    })
                    .for_each(|metadata| {
                        metadata.read().iter().for_each(|(md_key, _)| {
                            if !keys.iter().any(|key| md_key == key) {
                                keys.push(md_key.clone())
                            }
                        })
                    });

                nodes
                    .read()
                    .iter()
                    .filter_map(|node| {
                        node.assets().with(|assets| {
                            if let db::state::DataResource::Ok(assets) = assets {
                                Some(assets.read_only())
                            } else {
                                None
                            }
                        })
                    })
                    .flat_map(|assets| {
                        assets
                            .read()
                            .iter()
                            .map(|asset| asset.metadata().read_only())
                            .collect::<Vec<_>>()
                    })
                    .for_each(|metadata| {
                        metadata.read().iter().for_each(|(md_key, _)| {
                            if !keys.iter().any(|key| md_key == key) {
                                keys.push(md_key.clone())
                            }
                        })
                    });

                keys.sort();
                keys
            })
        }
    }
}

pub mod display {
    use leptos::{html, prelude::*};

    #[derive(Clone)]
    pub struct State {
        data_source: ReadSignal<Vec<super::data::Datum>>,
        data: RwSignal<Vec<super::data::Datum>>,
        columns: Columns,
        sort: RwSignal<Sort>,
    }

    impl State {
        pub fn new(data: ReadSignal<Vec<super::data::Datum>>) -> Self {
            let data_source = data;
            let data = RwSignal::new(data_source.get_untracked());
            let sort = RwSignal::new(Sort::default());

            Effect::watch(
                sort.read_only(),
                {
                    let data = data.write_only();
                    move |sort, prev, _| {
                        Self::sort_effect(sort, prev, data);
                    }
                },
                false,
            );

            Self {
                data_source,
                data,
                columns: Columns::new(),
                sort,
            }
        }
    }

    impl State {
        fn sort_effect(
            sort: &Sort,
            prev: Option<&Sort>,
            data: WriteSignal<Vec<super::data::Datum>>,
        ) {
            tracing::debug!("0");
            let rev_only = prev
                .map(|prev| sort.field() == prev.field())
                .unwrap_or(false);

            if rev_only {
                tracing::debug!("1");
                data.write().reverse();
                tracing::debug!("2");
            } else {
                tracing::debug!("3");

                match sort.field() {
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
                    SortField::Metadata(key) => {
                        data.write().sort_by_key(|datum| {
                            datum.metadata().read_untracked().iter().find_map(
                                |(md_key, md_value)| {
                                    (md_key == key).then_some(md_value.get_untracked().to_string())
                                },
                            )
                        })
                    }
                }
                tracing::debug!("4");

                if matches!(sort.direction(), SortDirection::Des) {
                    tracing::debug!("5");

                    data.write().reverse();
                }
                tracing::debug!("6");
            }
        }
    }

    impl State {
        pub fn data(&self) -> ReadSignal<Vec<super::data::Datum>> {
            self.data.read_only()
        }

        pub fn columns(&self) -> &Columns {
            &self.columns
        }

        pub fn sort(&self) -> RwSignal<Sort> {
            self.sort
        }
    }

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
    pub struct ColumnPinnable {
        node_ref: NodeRef<html::Col>,
        visible: RwSignal<bool>,
        pinned: RwSignal<bool>,
        width: RwSignal<usize>,
    }

    impl ColumnPinnable {
        pub fn new() -> Self {
            Self {
                node_ref: NodeRef::<html::Col>::new(),
                visible: RwSignal::new(true),
                pinned: RwSignal::new(false),
                width: RwSignal::new(0),
            }
        }

        pub fn node_ref(&self) -> NodeRef<html::Col> {
            self.node_ref
        }

        pub fn visible(&self) -> RwSignal<bool> {
            self.visible
        }

        pub fn pinned(&self) -> RwSignal<bool> {
            self.pinned
        }

        pub fn width(&self) -> RwSignal<usize> {
            self.width
        }
    }

    #[derive(Clone)]
    pub struct Columns {
        path: ColumnPinnable,
        file: ColumnPinnable,
        name: Column,
        kind: Column,
        description: Column,
        tags: Column,
        metadata: RwSignal<Vec<(String, Column)>>,
    }

    impl Columns {
        pub fn new() -> Self {
            Self {
                path: ColumnPinnable::new(),
                file: ColumnPinnable::new(),
                name: Column::new(),
                kind: Column::new(),
                description: Column::new(),
                tags: Column::new(),
                metadata: RwSignal::new(vec![]),
            }
        }

        pub fn path(&self) -> &ColumnPinnable {
            &self.path
        }

        pub fn file(&self) -> &ColumnPinnable {
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

        pub fn metadata(&self, key: &String) -> Option<Column> {
            self.metadata
                .read()
                .iter()
                .find_map(|(col_key, col)| (key == col_key).then_some(col.clone()))
        }

        /// Create a new column for a metadata key.
        ///
        /// # Returns
        /// Whether the column was successfully created.
        /// This fails if the key is already present.
        pub fn new_metadata(&self, key: String) -> bool {
            if self
                .metadata
                .read_untracked()
                .iter()
                .any(|(col_key, _)| &key == col_key)
            {
                return false;
            }

            self.metadata.write().push((key, Column::new()));
            true
        }

        /// Set all visibilities to `true`.
        pub fn all_visible(&self) {
            self.path.visible().set(true);
            self.file.visible().set(true);
            self.name.visible().set(true);
            self.kind.visible().set(true);
            self.description.visible().set(true);
            self.tags.visible().set(true);

            for (_, col) in self.metadata.read().iter() {
                col.visible().set(true);
            }
        }
    }

    #[derive(PartialEq, Clone, Copy, Default, Debug)]
    pub enum SortDirection {
        /// Ascending.
        #[default]
        Asc,

        /// Descending.
        Des,
    }

    #[derive(Clone, Default, PartialEq, Debug)]
    pub enum SortField {
        #[default]
        Path,
        File,
        Name,
        Kind,
        Metadata(String),
    }

    #[derive(Clone)]
    pub struct Sort {
        field: SortField,
        direction: SortDirection,
    }

    impl Sort {
        pub fn ascending(field: SortField) -> Self {
            Self {
                field,
                direction: SortDirection::Asc,
            }
        }

        pub fn descending(field: SortField) -> Self {
            Self {
                field,
                direction: SortDirection::Des,
            }
        }

        pub fn field(&self) -> &SortField {
            &self.field
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
}

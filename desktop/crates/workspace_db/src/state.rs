pub mod data {
    use leptos::prelude::*;
    use std::{
        path::{self, PathBuf},
        sync::Arc,
    };
    use syre_core as core;
    use syre_desktop_ui_lib as ui_lib;
    use syre_project_watcher as db;

    #[derive(Clone)]
    pub struct Datum {
        ancestors: RwSignal<Vec<ui_lib::state::graph::Node>>,
        asset: ui_lib::state::Asset,
        path: ReadSignal<PathBuf>,
        metadata: ReadSignal<Vec<(String, MetadatumValue)>>,
    }

    impl Datum {
        #[cfg_attr(feature = "tracing", tracing::instrument(level = "trace", skip_all))]
        pub fn new(
            ancestors: RwSignal<Vec<ui_lib::state::graph::Node>>,
            asset: ui_lib::state::Asset,
        ) -> Self {
            let (path, set_path) = signal({
                let ancestors = ancestors.read_only();
                assert!(!ancestors.read_untracked().is_empty());

                let path = ancestors
                    .read_untracked()
                    .iter()
                    .rev()
                    .skip(1)
                    .map(|ancestor| ancestor.name().get_untracked())
                    .collect::<PathBuf>();

                std::iter::once(path::Component::RootDir)
                    .chain(path.components())
                    .collect::<PathBuf>()
            });
            Effect::new({
                let ancestors = ancestors.read_only();
                move |_| {
                    assert!(!ancestors.read().is_empty());

                    let path = ancestors
                        .read()
                        .iter()
                        .rev()
                        .skip(1)
                        .map(|ancestor| ancestor.name().get())
                        .collect::<PathBuf>();

                    let path = std::iter::once(path::Component::RootDir)
                        .chain(path.components())
                        .collect::<PathBuf>();

                    set_path(path);
                }
            });

            let (metadata, set_metadata) = signal({
                let ancestors = ancestors.read_only();
                let asset_md = asset.metadata().read_only();
                let mut metadata = vec![];
                ancestors
                    .read_untracked()
                    .iter()
                    .filter_map(|ancestor| {
                        let properties = ancestor.properties().read_only();
                        properties.with_untracked(|properties| {
                            if let db::state::DataResource::Ok(properties) = properties {
                                Some((ancestor.clone(), properties.metadata().read_only()))
                            } else {
                                None
                            }
                        })
                    })
                    .rev()
                    .for_each(|(ancestor, md)| {
                        for (md_key, md_value) in md.read_untracked().iter() {
                            if let Some((_, value)) =
                                metadata.iter_mut().find(|(key, _)| key == md_key)
                            {
                                *value = MetadatumValue::inherited(
                                    md_value.read_only(),
                                    ancestor.clone(),
                                );
                            } else {
                                metadata.push((
                                    md_key.clone(),
                                    MetadatumValue::inherited(
                                        md_value.read_only(),
                                        ancestor.clone(),
                                    ),
                                ))
                            }
                        }
                    });

                for (md_key, md_value) in asset_md.read_untracked().iter() {
                    if let Some((_, value)) = metadata.iter_mut().find(|(key, _)| key == md_key) {
                        *value = MetadatumValue::owned(md_value.read_only());
                    } else {
                        metadata.push((md_key.clone(), MetadatumValue::owned(md_value.read_only())))
                    }
                }

                metadata
            });

            Effect::new({
                let ancestors = ancestors.read_only();
                let asset_md = asset.metadata().read_only();
                move |_| {
                    let mut metadata = vec![];
                    ancestors
                        .read()
                        .iter()
                        .filter_map(|ancestor| {
                            let properties = ancestor.properties().read_only();
                            properties.with(|properties| {
                                if let db::state::DataResource::Ok(properties) = properties {
                                    Some((ancestor.clone(), properties.metadata().read_only()))
                                } else {
                                    None
                                }
                            })
                        })
                        .rev()
                        .for_each(|(ancestor, md)| {
                            for (md_key, md_value) in md.read().iter() {
                                if let Some((_, value)) =
                                    metadata.iter_mut().find(|(key, _)| key == md_key)
                                {
                                    *value = MetadatumValue::inherited(
                                        md_value.read_only(),
                                        ancestor.clone(),
                                    );
                                } else {
                                    metadata.push((
                                        md_key.clone(),
                                        MetadatumValue::inherited(
                                            md_value.read_only(),
                                            ancestor.clone(),
                                        ),
                                    ))
                                }
                            }
                        });

                    for (md_key, md_value) in asset_md.read().iter() {
                        if let Some((_, value)) = metadata.iter_mut().find(|(key, _)| key == md_key)
                        {
                            *value = MetadatumValue::owned(md_value.read_only());
                        } else {
                            metadata
                                .push((md_key.clone(), MetadatumValue::owned(md_value.read_only())))
                        }
                    }

                    set_metadata(metadata);
                }
            });

            Self {
                ancestors,
                asset,
                path,
                metadata,
            }
        }

        pub fn asset(&self) -> &ui_lib::state::Asset {
            &self.asset
        }

        /// Container path.
        pub fn path(&self) -> ReadSignal<PathBuf> {
            self.path
        }

        /// Metadata values including inheritance.
        pub fn metadata(&self) -> ReadSignal<Vec<(String, MetadatumValue)>> {
            self.metadata
        }
    }

    #[derive(Clone)]
    pub enum MetadatumSource {
        Owned,
        Inherited(ui_lib::state::graph::Node),
    }

    #[derive(derive_more::Deref, Clone)]
    pub struct MetadatumValue {
        #[deref]
        value: ReadSignal<core::types::Value>,
        source: MetadatumSource,
    }

    impl MetadatumValue {
        pub fn owned(value: ReadSignal<core::types::Value>) -> Self {
            Self {
                value,
                source: MetadatumSource::Owned,
            }
        }

        pub fn inherited(
            value: ReadSignal<core::types::Value>,
            owner: ui_lib::state::graph::Node,
        ) -> Self {
            Self {
                value,
                source: MetadatumSource::Inherited(owner),
            }
        }

        pub fn value(&self) -> ReadSignal<core::types::Value> {
            self.value
        }

        pub fn source(&self) -> &MetadatumSource {
            &self.source
        }

        pub fn is_owned(&self) -> bool {
            matches!(self.source, MetadatumSource::Owned)
        }
    }

    #[derive(Clone)]
    pub struct State {
        /// Graph state.
        graph: ui_lib::state::Graph,

        /// Individual assets.
        data: RwSignal<Vec<Datum>>,

        /// All metdata keys.
        metadata_keys: ReadSignal<Vec<String>>,
    }

    impl State {
        pub fn from(graph: ui_lib::state::Graph) -> Self {
            let node_states = graph
                .nodes()
                .read_untracked()
                .iter()
                .map(|node| (node.clone(), node.assets().read_only()))
                .collect::<Vec<_>>();

            let node_assets = node_states
                .iter()
                .filter_map(|(node, state)| {
                    if let db::state::DataResource::Ok(assets) = state.get_untracked() {
                        Some((node.clone(), assets.read_only()))
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>();

            let data = node_assets
                .iter()
                .flat_map(|(node, assets)| {
                    assets
                        .get_untracked()
                        .into_iter()
                        .map(move |asset| (node.clone(), asset))
                })
                .map(|(node, asset)| {
                    let ancestors = graph.ancestors(&node);
                    assert!(!ancestors.is_empty());
                    Datum::new(RwSignal::new(ancestors), asset)
                })
                .collect::<Vec<_>>();

            let node_states = RwSignal::new(node_states);
            let node_assets = RwSignal::new(node_assets);
            let data = RwSignal::new(data);

            let _ = Effect::watch(
                graph.nodes().read_only(),
                move |nodes, _, _| {
                    if nodes.len() == node_states.read_untracked().len() {
                        return;
                    }

                    node_states.update(|node_states| {
                        node_states.retain(|(node_state, _)| {
                            nodes.iter().any(|node| Arc::ptr_eq(node_state, node))
                        });

                        let added = nodes
                            .iter()
                            .filter(|node| {
                                !node_states
                                    .iter()
                                    .any(|(node_state, _)| Arc::ptr_eq(node, node_state))
                            })
                            .map(|node| node.clone())
                            .collect::<Vec<_>>();
                        for node in added.into_iter() {
                            let assets = node.assets().read_only();
                            node_states.push((node, assets));
                        }
                    });
                },
                false,
            );

            let _ = Effect::watch(
                {
                    let node_states = node_states.read_only();
                    move || {
                        node_states
                            .read()
                            .iter()
                            .for_each(|(_, state)| state.track());
                    }
                },
                move |_, _, _| {
                    let removed = node_assets
                        .read_untracked()
                        .iter()
                        .filter(|(assets_node, _)| {
                            node_states
                                .read_untracked()
                                .iter()
                                .find(|(node, _)| Arc::ptr_eq(node, assets_node))
                                .map(|(_, state)| state.read_untracked().is_err())
                                .unwrap_or(true)
                        })
                        .map(|(node, _)| node.clone())
                        .collect::<Vec<_>>();

                    let added = node_states
                        .read_untracked()
                        .iter()
                        .filter(|(node, _)| {
                            !node_assets
                                .read_untracked()
                                .iter()
                                .any(|(asset_node, _)| Arc::ptr_eq(asset_node, node))
                        })
                        .filter_map(|(node, state)| {
                            if let db::state::DataResource::Ok(assets) = state.get_untracked() {
                                Some((node.clone(), assets.read_only()))
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<_>>();

                    if !added.is_empty() || !removed.is_empty() {
                        node_assets.update(|node_assets| {
                            node_assets.retain(|(assets_node, _)| {
                                removed
                                    .iter()
                                    .find(|removed| Arc::ptr_eq(removed, assets_node))
                                    .is_none()
                            });

                            for asset in added {
                                node_assets.push(asset);
                            }
                        });
                    }
                },
                false,
            );

            let _ = Effect::watch(
                {
                    let node_assets = node_assets.read_only();
                    move || {
                        node_assets.read().iter().for_each(|(_, assets)| {
                            assets.track();
                        })
                    }
                },
                {
                    let graph = graph.clone();
                    move |_, _, _| {
                        let assets = node_assets
                            .read_untracked()
                            .iter()
                            .flat_map(|(node, assets)| {
                                assets
                                    .get_untracked()
                                    .into_iter()
                                    .map(|asset| (node.clone(), asset))
                                    .collect::<Vec<_>>()
                            })
                            .collect::<Vec<_>>();

                        let mut removed = data
                            .read_untracked()
                            .iter()
                            .enumerate()
                            .filter_map(|(idx, datum)| {
                                let rid = datum.asset().rid().get_untracked();
                                (!assets
                                    .iter()
                                    .any(|(_, asset)| asset.rid().get_untracked() == rid))
                                .then_some(idx)
                            })
                            .collect::<Vec<_>>();
                        removed.sort();

                        let added = assets
                            .iter()
                            .filter(|(_, asset)| {
                                let rid = asset.rid().get_untracked();
                                !data
                                    .read_untracked()
                                    .iter()
                                    .any(|datum| datum.asset().rid().get_untracked() == rid)
                            })
                            .collect::<Vec<_>>();

                        if !removed.is_empty() || !added.is_empty() {
                            data.update(|data| {
                                for idx in removed.into_iter().rev() {
                                    data.swap_remove(idx);
                                }

                                for (node, asset) in added {
                                    let ancestors = graph.ancestors(node);
                                    assert!(!ancestors.is_empty());
                                    data.push(Datum::new(RwSignal::new(ancestors), asset.clone()));
                                }
                            });
                        }
                    }
                },
                false,
            );

            let (metadata_keys, set_metadata_keys) = signal(vec![]);
            Effect::new({
                let nodes = graph.nodes().read_only();
                move |_| {
                    let mut keys = std::collections::HashSet::new();
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
                                if !keys.contains(md_key) {
                                    keys.insert(md_key.clone());
                                }
                            });
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
                                if !keys.contains(md_key) {
                                    keys.insert(md_key.clone());
                                }
                            });
                        });

                    let mut keys = keys.into_iter().collect::<Vec<_>>();
                    keys.sort();

                    if metadata_keys.with_untracked(|metadata_keys| *metadata_keys != keys) {
                        set_metadata_keys(keys);
                    }
                }
            });

            Self {
                graph,
                data,
                metadata_keys,
            }
        }

        pub fn data(&self) -> ReadSignal<Vec<Datum>> {
            self.data.read_only()
        }

        pub fn metadata_keys(&self) -> ReadSignal<Vec<String>> {
            self.metadata_keys
        }
    }
}

pub mod display {
    use leptos::{html, prelude::*};
    use syre_core::types::ResourceId;

    #[derive(Clone)]
    pub struct State {
        data_source: ReadSignal<Vec<super::data::Datum>>,
        data_sorted: RwSignal<Vec<super::data::Datum>>,
        data: RwSignal<Vec<super::data::Datum>>,
        columns: Columns,
        filter_bar: RwSignal<bool>,
        sort: RwSignal<Sort>,
        filter: RwSignal<Option<Vec<ResourceId>>>,
    }

    impl State {
        #[cfg_attr(feature = "tracing", tracing::instrument(level = "trace", skip_all))]
        pub fn new(source: &super::data::State) -> Self {
            let data_source = source.data();
            let data_sorted = RwSignal::new(data_source.get_untracked());
            let data = RwSignal::new(data_source.get_untracked());
            let sort = RwSignal::new(Sort::default());
            let filter = RwSignal::new(None);
            let columns = Columns::new();

            source
                .metadata_keys()
                .read_untracked()
                .iter()
                .for_each(|key| {
                    assert!(columns.new_metadata(key.clone()));
                });

            Effect::watch(
                data_source,
                {
                    let sort = sort.read_only();
                    move |data_source, _, _| {
                        let mut removed = data_sorted
                            .read_untracked()
                            .iter()
                            .enumerate()
                            .filter_map(|(idx, datum)| {
                                let rid = datum.asset().rid().read_only();
                                (!data_source.iter().any(|source_datum| {
                                    *source_datum.asset().rid().read_untracked()
                                        == *rid.read_untracked()
                                }))
                                .then_some(idx)
                            })
                            .collect::<Vec<_>>();
                        removed.sort();

                        let added = data_source
                            .iter()
                            .filter(|source_datum| {
                                let source_rid = source_datum.asset().rid().read_only();
                                !data_sorted.read_untracked().iter().any(|datum| {
                                    *datum.asset().rid().read_untracked()
                                        == *source_rid.read_untracked()
                                })
                            })
                            .map(|datum| datum.clone())
                            .collect::<Vec<_>>();

                        assert!(!removed.is_empty() || !added.is_empty());
                        data_sorted.update(|data_sorted| {
                            for idx in removed.into_iter().rev() {
                                data_sorted.swap_remove(idx);
                            }

                            for datum in added {
                                data_sorted.push(datum);
                            }
                        });

                        sort.with_untracked(|sort| Self::sort_data(sort, data_sorted.write_only()));
                    }
                },
                false,
            );

            Effect::watch(
                source.metadata_keys(),
                {
                    let columns = columns.metadata;
                    move |keys, _, _| {
                        let removed = columns
                            .read_untracked()
                            .iter()
                            .filter_map(|(md_key, _)| {
                                (!keys.iter().any(|key| md_key == key)).then_some(md_key.clone())
                            })
                            .collect::<Vec<_>>();

                        let added = keys
                            .iter()
                            .filter(|key| {
                                !columns
                                    .read_untracked()
                                    .iter()
                                    .any(|(md_key, _)| key == &md_key)
                            })
                            .collect::<Vec<_>>();

                        columns.update(|columns| {
                            columns.retain(|(key, _)| !removed.contains(key));
                            added
                                .into_iter()
                                .for_each(|key| columns.push((key.clone(), Column::new())));
                        });
                    }
                },
                true,
            );

            Effect::watch(
                sort.read_only(),
                {
                    let data_sorted = data_sorted.write_only();
                    let data = data.write_only();
                    move |sort, prev, _| {
                        Self::maybe_sort(sort, prev, data_sorted, data);
                    }
                },
                false,
            );

            Effect::watch(
                move || (filter.get(), data_sorted.get()),
                {
                    let data = data.write_only();
                    move |(filter, data_sorted), _, _| {
                        Self::filter_effect(filter, data_sorted, data);
                    }
                },
                false,
            );

            Self {
                data_source,
                data_sorted,
                data,
                columns,
                filter_bar: RwSignal::new(false),
                sort,
                filter,
            }
        }
    }

    impl State {
        fn sort_data(sort: &Sort, data_sorted: WriteSignal<Vec<super::data::Datum>>) {
            match sort.field() {
                SortField::Path => data_sorted.write().sort_by_key(|datum| {
                    datum
                        .path()
                        .get_untracked()
                        .to_string_lossy()
                        .to_lowercase()
                }),
                SortField::File => data_sorted.write().sort_by_key(|datum| {
                    datum
                        .asset()
                        .path()
                        .get_untracked()
                        .to_string_lossy()
                        .to_lowercase()
                }),
                SortField::Name => {
                    data_sorted.write().sort_by_key(|datum| {
                        datum
                            .asset()
                            .name()
                            .get_untracked()
                            .map(|value| value.to_lowercase())
                    });
                }
                SortField::Kind => {
                    data_sorted.write().sort_by_key(|datum| {
                        datum
                            .asset()
                            .kind()
                            .get_untracked()
                            .map(|value| value.to_lowercase())
                    });
                }
                SortField::Metadata(key) => data_sorted.write().sort_by_key(|datum| {
                    datum
                        .metadata()
                        .read_untracked()
                        .iter()
                        .find_map(|(md_key, md_value)| {
                            (md_key == key).then_some(md_value.get_untracked().to_string())
                        })
                }),
            }

            if matches!(sort.direction(), SortDirection::Des) {
                data_sorted.write().reverse();
            }
        }

        fn maybe_sort(
            sort: &Sort,
            prev: Option<&Sort>,
            data_sorted: WriteSignal<Vec<super::data::Datum>>,
            data: WriteSignal<Vec<super::data::Datum>>,
        ) {
            enum Action {
                None,
                Reverse,
                Sort,
            }

            let action = prev
                .map(|prev| {
                    if sort.field() == prev.field() {
                        if sort.direction() == prev.direction() {
                            Action::None
                        } else {
                            Action::Reverse
                        }
                    } else {
                        Action::Sort
                    }
                })
                .unwrap_or(Action::Sort);

            match action {
                Action::None => {}
                Action::Reverse => {
                    data_sorted.write().reverse();
                    data.write().reverse();
                }
                Action::Sort => {
                    Self::sort_data(sort, data_sorted);
                }
            }
        }

        fn filter_effect(
            filter: &Option<Vec<ResourceId>>,
            data_source: &Vec<super::data::Datum>,
            data: WriteSignal<Vec<super::data::Datum>>,
        ) {
            if let Some(filter) = filter {
                let filtered = data_source
                    .iter()
                    .filter(|datum| {
                        datum
                            .asset()
                            .rid()
                            .with_untracked(|rid| filter.contains(rid))
                    })
                    .collect::<Vec<_>>();

                data.set(filtered.into_iter().cloned().collect::<Vec<_>>());
            } else {
                data.set(data_source.clone())
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

        pub fn filter_bar(&self) -> RwSignal<bool> {
            self.filter_bar
        }

        pub fn sort(&self) -> RwSignal<Sort> {
            self.sort
        }

        pub fn filter(&self) -> RwSignal<Option<Vec<ResourceId>>> {
            self.filter
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

        pub fn metadata(&self) -> ReadSignal<Vec<(String, Column)>> {
            self.metadata.read_only()
        }

        pub fn get_metadata(&self, key: &String) -> Option<Column> {
            self.metadata
                .read_untracked()
                .iter()
                .find_map(|(col_key, col)| (key == col_key).then_some(col.clone()))
        }

        /// Create a new column for a metadata key.
        ///
        /// # Returns
        /// Whether the column was successfully created.
        /// This fails if the key is already present.
        #[cfg_attr(feature = "tracing", tracing::instrument(level = "trace", skip_all))]
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
        pub fn set_visibility_for_all(&self, visible: bool) {
            self.path.visible().set(visible);
            self.file.visible().set(visible);
            self.name.visible().set(visible);
            self.kind.visible().set(visible);
            self.description.visible().set(visible);
            self.tags.visible().set(visible);

            for (_, col) in self.metadata.read().iter() {
                col.visible().set(visible);
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

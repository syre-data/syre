use crate::{
    query::{AssetQuery, ContainerQuery},
    state,
};
use syre_core::{db::SearchFilter, types::Value};

impl SearchFilter<state::Container> for ContainerQuery {
    fn matches(&self, obj: &state::Container) -> bool {
        let state::DataResource::Ok(props) = &obj.properties() else {
            return false;
        };

        if let Some(query_name) = self.name.as_ref() {
            if query_name != &props.name {
                return false;
            }
        }

        if let Some(query_kind) = self.kind.as_ref() {
            if query_kind != &props.kind {
                return false;
            }
        }

        if !self.tags.iter().all(|tag| props.tags.contains(tag)) {
            return false;
        }

        for crate::query::Metadatum {
            key: query_key,
            value: query_val,
        } in self.metadata.iter()
        {
            let Some(container_val) = props.metadata.get(query_key) else {
                return false;
            };

            if container_val.is_number() && query_val.is_number() {
                // only compare number values, not types
                if container_val.as_f64() != query_val.as_f64() {
                    return false;
                }
            } else if container_val.is_quantity() && query_val.is_string() {
                // allow quantities to be represented as a string `"<magnitude> <unit>"`.
                let query_str = query_val.as_string().expect("checked as string");
                let query_parts = query_str.split_whitespace().collect::<Vec<_>>();
                let [magnitude, unit] = query_parts[..] else {
                    return false;
                };
                let Ok(magnitude) = magnitude.trim().parse::<f64>() else {
                    return false;
                };
                let query_quantity = Value::Quantity {
                    magnitude,
                    unit: unit.trim().to_string(),
                };

                if query_quantity != *container_val {
                    return false;
                }
            } else {
                if container_val != query_val {
                    return false;
                }
            }
        }

        // all search criteria matched
        true
    }
}

impl SearchFilter<state::Asset> for AssetQuery {
    fn matches(&self, obj: &state::Asset) -> bool {
        let asset = &obj.inner;
        let props = &asset.properties;

        if let Some(query_path) = self.path.as_ref() {
            if *query_path != asset.path {
                return false;
            }
        }

        if let Some(query_name) = self.name.as_ref() {
            if *query_name != props.name {
                return false;
            }
        }

        if let Some(query_kind) = self.kind.as_ref() {
            if *query_kind != props.kind {
                return false;
            }
        }

        if !self.tags.iter().all(|tag| props.tags.contains(tag)) {
            return false;
        }

        for crate::query::Metadatum {
            key: query_key,
            value: query_val,
        } in self.metadata.iter()
        {
            let Some(asset_val) = props.metadata.get(query_key) else {
                return false;
            };

            if asset_val.is_number() && query_val.is_number() {
                // only compare number values, not types
                if asset_val.as_f64() != query_val.as_f64() {
                    return false;
                }
            } else if asset_val.is_quantity() && query_val.is_string() {
                // allow quantities to be represented as a string `"<magnitude> <unit>"`.
                let query_str = query_val.as_string().expect("checked as string");
                let query_parts = query_str.split_whitespace().collect::<Vec<_>>();
                let [magnitude, unit] = query_parts[..] else {
                    return false;
                };
                let Ok(magnitude) = magnitude.trim().parse::<f64>() else {
                    return false;
                };
                let query_quantity = Value::Quantity {
                    magnitude,
                    unit: unit.trim().to_string(),
                };

                if query_quantity != *asset_val {
                    return false;
                }
            } else {
                if asset_val != query_val {
                    return false;
                }
            }
        }

        // all search criteria matched
        true
    }
}

use super::{AssetProperties, ContainerProperties};

#[derive(PartialEq, Eq, Clone)]
pub enum ResourceProperties {
    Container(ContainerProperties),
    Asset(AssetProperties),
}

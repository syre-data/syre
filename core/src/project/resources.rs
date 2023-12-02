use super::{AssetProperties, ContainerProperties};

#[derive(PartialEq, Eq, Clone)]
pub enum ResourceProperties {
    Container(ContainerProperties),
    Asset(AssetProperties),
}

impl From<ContainerProperties> for ResourceProperties {
    fn from(props: ContainerProperties) -> Self {
        Self::Container(props)
    }
}

impl From<AssetProperties> for ResourceProperties {
    fn from(props: AssetProperties) -> Self {
        Self::Asset(props)
    }
}

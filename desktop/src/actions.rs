pub mod container {
    use serde::{Deserialize, Serialize};
    use thot_core::types::ResourceId;

    #[derive(Serialize, Deserialize, Debug)]
    pub enum Action {
        /// Add a Script association to the Container.
        AddScriptAssociation(ResourceId),
    }
}

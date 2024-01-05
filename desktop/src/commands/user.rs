use crate::commands::common::{EmptyArgs, ResourceIdArgs};
use crate::common::invoke_result;
use thot_core::system::User;
use thot_core::types::ResourceId;

pub async fn get_active_user() -> thot_local::Result<Option<User>> {
    invoke_result("get_active_user", EmptyArgs {}).await
}

pub async fn set_active_user(user: ResourceId) -> thot_local::Result {
    invoke_result("set_active_user", ResourceIdArgs { rid: user }).await
}

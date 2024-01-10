use super::common::{EmptyArgs, ResourceIdArgs};
use crate::common::invoke_result;
use thot_core::system::User;
use thot_core::types::ResourceId;
use thot_local::Result;

pub async fn get_active_user() -> Result<Option<User>> {
    invoke_result("get_active_user", EmptyArgs {}).await
}

pub async fn set_active_user(user: ResourceId) -> Result {
    invoke_result("set_active_user", ResourceIdArgs { rid: user }).await
}

pub async fn unset_active_user() -> Result {
    invoke_result("unset_active_user", EmptyArgs {}).await
}
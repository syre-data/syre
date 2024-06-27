use crate::invoke::invoke_result;

/// # Returns
/// User count if user manifest is `Ok`,
/// otherwise `Err`.
pub async fn count() -> Result<usize, ()> {
    invoke_result("user_count", ()).await
}

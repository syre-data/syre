/// # Returns
/// User count if user manifest is `Ok`,
/// otherwise `Err`.
pub async fn count() -> Result<usize, ()> {
    tauri_sys::core::invoke_result("user_count", ()).await
}

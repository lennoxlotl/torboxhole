use crate::user::UserData;

/// Resolves the maximum concurrent downloads the given user is allowed to have.
pub fn resolve_download_limit(data: &UserData) -> i32 {
    let base_downloads = match data.plan {
        0 => 1,
        1 => 3,
        2 => 10,
        3 => 5,
        _ => 0,
    };
    // This will never exceed i32s bit limit
    (base_downloads + data.additional_concurrent_slots) as i32
}

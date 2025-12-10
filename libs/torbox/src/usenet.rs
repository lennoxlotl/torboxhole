use crate::TorBoxApiState;
use crate::request::{OptionalForm, Request, tb_url};
use chrono::{DateTime, Utc};
use reqwest::Method;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Data returned by `api/usenet/createusenetdownload`
#[derive(Deserialize, Debug, Clone)]
pub struct CreateDownloadData {
    pub hash: String,
    pub usenetdownload_id: i32,
    pub auth_id: String,
}

/// Data returned by `api/usenet/mylist`
///
/// Reference: https://www.postman.com/wamy-dev/torbox/example/29572726-9d09c4a0-a0b2-41d9-aff7-76ca43d09e7c
#[derive(Deserialize, Debug, Clone)]
pub struct ListData {
    pub id: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub auth_id: String,
    pub name: String,
    pub hash: String,
    pub download_state: String,
    pub download_speed: i32,
    pub original_url: Option<String>,
    pub eta: i32,
    pub progress: i32,
    pub size: i32,
    pub download_id: String,
    pub files: Vec<ListDataFile>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ListDataFile {
    pub id: i32,
    pub md5: Option<String>,
    pub hash: String,
    pub name: String,
    pub size: i32,
    pub zipped: bool,
    pub s3_path: String,
    pub infected: bool,
    pub mimetype: String,
    pub short_name: String,
    pub absolute_path: String,
    pub opensubtitles_hash: Option<String>,
}

/// Creates a usenet download under your account. Send **either** a link or an nzb file.
/// Once they have been checked, they will begin downloading assuming your account has available
/// active download slots, and they aren't too large.
///
/// ### Arguments
/// * `state` - TorBox API state
/// * `path` - Path to a .nzb file
/// * `link` - Link to a .nzb file (raw)
/// * `name` - The name you want the usenet download to be
/// * `password` - The password to send to the Usenet client. This is used for extracting the RAR at the end.
/// * `post_processing` - The post-processing option to use. See https://www.postman.com/wamy-dev/torbox/request/zczxfjg/create-usenet-download?tab=overview
/// * `as_queued` - Tells TorBox you want this usenet download instantly queued.
/// * `add_only_if_cached` - Only adds the download if it is cached on TorBox. If not cached, it won't be added.
pub async fn create_download<P: AsRef<Path>>(
    state: &TorBoxApiState,
    path: Option<P>,
    link: Option<String>,
    name: Option<String>,
    password: Option<String>,
    post_processing: i8,
    as_queued: bool,
    add_only_if_cached: bool,
) -> eyre::Result<CreateDownloadData> {
    Request::<()>::builder()
        .with_url(&tb_url(state, "api/usenet/createusenetdownload"))
        .with_method(Method::POST)
        .with_multipart(
            OptionalForm::new()
                .file("file", path)
                .await?
                .text("link", link)
                .text("name", name)
                .text("password", password)
                .text("post_processing", Some(post_processing.to_string()))
                .text("as_queued", Some(as_queued.to_string()))
                .text("add_only_if_cached", Some(add_only_if_cached.to_string()))
                .inner(),
        )
        .with_auth_key(&state.api_key)
        .build()
        .send_parse()
        .await
}

#[derive(Serialize, Debug, Default, Clone)]
struct RequestDlQuery<'a> {
    token: &'a str,
    usenet_id: i32,
    file_id: Option<i32>,
    zip_link: bool,
    user_ip: Option<String>,
}

/// Requests the download link from the server. Because downloads are metered, TorBox cannot
/// afford to allow free access to the links directly. This endpoint opens the link for 3 hours
/// for downloads. Once a download is started, the user has nearly unlimited time to download the file.
/// The 1-hour time limit is simply for starting downloads. This prevents long-term link sharing.
///
/// ### Arguments
/// * `state` - TorBox API state
/// * `usenet_id` - The usenet download's ID that you want to download
/// * `file_id` - The files' ID that you want to download. Optional if using "zip_link"
/// * `zip_link` - If you want a zip link. Required if no file_id. Takes precedence over file_id if both are given.
/// * `user_ip` - The user's IPv4 to determine the closest CDN.
pub async fn request_dl(
    state: &TorBoxApiState,
    usenet_id: i32,
    file_id: Option<i32>,
    zip_link: bool,
    user_ip: Option<String>,
) -> eyre::Result<String> {
    Request::builder()
        .with_url(&tb_url(state, "api/usenet/requestdl"))
        .with_method(Method::GET)
        .with_query(RequestDlQuery {
            token: &state.api_key,
            usenet_id,
            file_id,
            zip_link,
            user_ip,
        })
        .build()
        .send_parse()
        .await
}

#[derive(Serialize, Debug, Default, Clone)]
struct ListQuery {
    bypass_cache: bool,
    id: Option<i32>,
    offset: i32,
    limit: i32,
}

const MY_LIST_URL: &'static str = "api/usenet/mylist";

/// See [``list_all`] for overview.
///
/// ### Arguments
/// * `state` - TorBox API state
/// * `bypass_cache` - Allows you to bypass the cached data, and always get fresh information.
/// * `id` - Determines the usenet download requested.
pub async fn list(state: &TorBoxApiState, bypass_cache: bool, id: i32) -> eyre::Result<ListData> {
    Request::builder()
        .with_url(&tb_url(state, MY_LIST_URL))
        .with_method(Method::GET)
        .with_query(ListQuery {
            bypass_cache,
            id: Some(id),
            ..Default::default()
        })
        .with_auth_key(&state.api_key)
        .build()
        .send_parse()
        .await
}

/// Gets the user's usenet download list. This gives you the necessary information to perform other
/// usenet actions. Unlike Torrents, this information is updated on its own every 5 seconds for
/// live usenet downloads.
///
/// ### Arguments
/// * `state` - TorBox API state
/// * `bypass_cache` - Allows you to bypass the cached data, and always get fresh information.
/// * `offset` - Determines the offset of items to get from the database. Default is 0.
/// * `limit` - Determines the number of items to receive per request. Default is 1000.
pub async fn list_all(
    state: &TorBoxApiState,
    bypass_cache: bool,
    offset: i32,
    limit: i32,
) -> eyre::Result<Vec<ListData>> {
    Request::builder()
        .with_url(&tb_url(state, MY_LIST_URL))
        .with_method(Method::GET)
        .with_query(ListQuery {
            bypass_cache,
            id: None,
            offset,
            limit,
        })
        .with_auth_key(&state.api_key)
        .build()
        .send_parse()
        .await
}

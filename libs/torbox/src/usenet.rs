use crate::TorBoxApiConfig;
use crate::request::{OptionalForm, Request, tb_url};
use chrono::{DateTime, Utc};
use reqwest::Method;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Data returned by `api/usenet/createusenetdownload`
#[derive(Deserialize, Debug, Clone)]
pub struct CreateDownloadData {
    hash: String,
    usenetdownload_id: i32,
    auth_id: String,
}

/// Data returned by `api/usenet/mylist`
///
/// Reference: https://www.postman.com/wamy-dev/torbox/example/29572726-9d09c4a0-a0b2-41d9-aff7-76ca43d09e7c
#[derive(Deserialize, Debug, Clone)]
pub struct ListData {
    id: i32,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    auth_id: String,
    name: String,
    hash: String,
    download_state: String,
    download_speed: i32,
    original_url: Option<String>,
    eta: i32,
    progress: i32,
    size: i32,
    download_id: String,
    files: Vec<ListDataFile>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ListDataFile {
    id: i32,
    md5: Option<String>,
    hash: String,
    name: String,
    size: i32,
    zipped: bool,
    s3_path: String,
    infected: bool,
    mimetype: String,
    short_name: String,
    absolute_path: String,
    opensubtitles_hash: Option<String>,
}

/// Creates a usenet download under your account. Send **either** a link or an nzb file.
/// Once they have been checked, they will begin downloading assuming your account has available
/// active download slots, and they aren't too large.
///
/// ### Arguments
/// * `config` - TorBox API config
/// * `path` - Path to a .nzb file
/// * `link` - Link to a .nzb file (raw)
/// * `name` - The name you want the usenet download to be
/// * `password` - The password to send to the Usenet client. This is used for extracting the RAR at the end.
/// * `post_processing` - The post-processing option to use. See https://www.postman.com/wamy-dev/torbox/request/zczxfjg/create-usenet-download?tab=overview
/// * `as_queued` - Tells TorBox you want this usenet download instantly queued.
/// * `add_only_if_cached` - Only adds the download if it is cached on TorBox. If not cached, it won't be added.
pub async fn create_download<P: AsRef<Path>>(
    config: &TorBoxApiConfig,
    path: Option<P>,
    link: Option<String>,
    name: Option<String>,
    password: Option<String>,
    post_processing: i8,
    as_queued: bool,
    add_only_if_cached: bool,
) -> eyre::Result<CreateDownloadData> {
    Request::<()>::builder()
        .with_url(tb_url(config, "api/usenet/createusenetdownload"))
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
        .with_auth_key(config.api_key.to_owned())
        .build()
        .send_parse()
        .await
}

#[derive(Serialize, Debug, Default, Clone)]
struct RequestDlQuery {
    token: String,
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
/// * `config` - TorBox API config
/// * `usenet_id` - The usenet download's ID that you want to download
/// * `file_id` - The files' ID that you want to download. Optional if using "zip_link"
/// * `zip_link` - If you want a zip link. Required if no file_id. Takes precedence over file_id if both are given.
/// * `user_ip` - The user's IPv4 to determine the closest CDN.
pub async fn request_dl(
    config: &TorBoxApiConfig,
    usenet_id: i32,
    file_id: Option<i32>,
    zip_link: bool,
    user_ip: Option<String>,
) -> eyre::Result<String> {
    Request::builder()
        .with_url(tb_url(config, "api/usenet/requestdl"))
        .with_method(Method::GET)
        .with_query(RequestDlQuery {
            token: config.api_key.to_string(),
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
/// * `config` - TorBox API config
/// * `bypass_cache` - Allows you to bypass the cached data, and always get fresh information.
/// * `id` - Determines the usenet download requested.
pub async fn list(config: &TorBoxApiConfig, bypass_cache: bool, id: i32) -> eyre::Result<ListData> {
    Request::builder()
        .with_url(tb_url(config, MY_LIST_URL))
        .with_method(Method::GET)
        .with_query(ListQuery {
            bypass_cache,
            id: Some(id),
            ..Default::default()
        })
        .with_auth_key(config.api_key.to_owned())
        .build()
        .send_parse()
        .await
}

/// Gets the user's usenet download list. This gives you the necessary information to perform other
/// usenet actions. Unlike Torrents, this information is updated on its own every 5 seconds for
/// live usenet downloads.
///
/// ### Arguments
/// * `config` - TorBox API config
/// * `bypass_cache` - Allows you to bypass the cached data, and always get fresh information.
/// * `offset` - Determines the offset of items to get from the database. Default is 0.
/// * `limit` - Determines the number of items to receive per request. Default is 1000.
pub async fn list_all(
    config: &TorBoxApiConfig,
    bypass_cache: bool,
    offset: i32,
    limit: i32,
) -> eyre::Result<Vec<ListData>> {
    Request::builder()
        .with_url(tb_url(config, MY_LIST_URL))
        .with_method(Method::GET)
        .with_query(ListQuery {
            bypass_cache,
            id: None,
            offset,
            limit,
        })
        .with_auth_key(config.api_key.to_owned())
        .build()
        .send_parse()
        .await
}

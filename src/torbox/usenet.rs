use crate::torbox::request::{tb_url, OptionalForm, Request};
use crate::torbox::TorBoxApiConfig;
use std::path::Path;
use reqwest::Method;

pub async fn create_download<P: AsRef<Path>>(
    config: &TorBoxApiConfig,
    path: Option<P>,
    name: Option<String>,
    password: Option<String>,
    post_processing: i8,
    as_queued: bool,
    add_only_if_cached: bool,
) -> eyre::Result<()> {
    Request::builder()
        .with_url(tb_url(
            config,
            "api/usenet/createusenetdownload".to_string(),
        ))
        .with_method(Method::POST)
        .with_multipart(
            OptionalForm::new()
                .file("file", path)
                .await?
                .text("name", name)
                .text("password", password)
                .text("post_processing", Some(post_processing.to_string()))
                .text("as_queued", Some(as_queued.to_string()))
                .text("add_only_if_cached", Some(add_only_if_cached.to_string()))
                .inner(),
        )
        .with_auth_key(config.api_key.to_owned())
        .build()
        .send()
        .await
}

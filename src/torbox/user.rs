use crate::torbox::request::{tb_url, Request};
use crate::torbox::TorBoxConfig;
use chrono::{DateTime, Utc};
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;

/// User data returned by `api/user/me`
///
/// Reference: https://www.postman.com/wamy-dev/torbox/example/29572726-3aba169f-23fc-48c6-927a-b1e9289a8b10
#[derive(Deserialize, Debug, Clone)]
pub struct UserData {
    id: i64,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    email: String,
    plan: i8,
    total_downloaded: i64,
    customer: String,
    #[serde(default)]
    server: i64,
    is_subscribed: bool,
    premium_expires_at: Option<DateTime<Utc>>,
    cooldown_until: Option<DateTime<Utc>>,
    auth_id: String,
    user_referral: Option<String>,
    base_email: String,
    #[serde(default)]
    settings: HashMap<String, Value>,
}

/// Returns the user data belonging to the authenticated user (API key).
pub async fn me(config: &TorBoxConfig, settings: bool) -> eyre::Result<UserData> {
    Request::builder()
        .with_url(tb_url(config, format!("api/user/me?settings={}", settings)))
        .with_method(reqwest::Method::GET)
        .with_auth_key(config.api_key.to_owned())
        .build()
        .send_parse::<UserData>()
        .await
}

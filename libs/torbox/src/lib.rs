use serde::Deserialize;

pub mod user;
pub mod usenet;
mod request;
pub mod download_limit;

/// Stores configuration for sending requests to the TorBox API.
#[derive(Debug, Clone)]
pub struct TorBoxApiState {
    api_base: String,
    api_version: String,
    api_key: String,
}

/// Generic response received from the TorBox API.
#[derive(Debug, Clone, Deserialize)]
pub struct GenericTorBoxJson<T> {
    success: bool,
    error: Option<String>,
    data: Option<T>,
}

impl TorBoxApiState {
    pub fn new(api_base: String, api_version: String, api_key: String) -> Self {
        Self {
            api_base,
            api_version,
            api_key,
        }
    }
}

use serde::Deserialize;

pub mod user;
mod request;

/// Stores configuration for sending requests to the TorBox API.
#[derive(Debug, Clone)]
pub struct TorBoxConfig {
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

impl TorBoxConfig {
    pub fn new(api_base: String, api_version: String, api_key: String) -> Self {
        Self {
            api_base,
            api_version,
            api_key,
        }
    }
}

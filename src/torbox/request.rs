use crate::torbox::{GenericTorBoxJson, TorBoxConfig};
use eyre::eyre;
use reqwest::{Method, Response};
use serde::de::DeserializeOwned;

/// Wrapper for [`reqwest::Request`]s allowing easier access to the TorBox API.
pub struct Request {
    config: Config,
}

pub struct RequestBuilder {
    config: Config,
}

#[derive(Default)]
struct Config {
    url: String,
    method: Method,
    auth_key: String,
}

impl Request {
    pub fn builder() -> RequestBuilder {
        RequestBuilder {
            config: Config::default(),
        }
    }

    /// Sends the built request expecting a result defined as [`T`].
    pub async fn send_parse<T: DeserializeOwned>(self) -> eyre::Result<T> {
        let generic_response = self
            .inner_send()
            .await?
            .json::<GenericTorBoxJson<T>>()
            .await
            .map_err(|e| eyre::eyre!("failed to parse response: {}", e))?;

        // No data present if request is unsuccessful
        if !generic_response.success {
            return Err(eyre!(
                "torbox error: {}",
                generic_response.error.unwrap_or("unknown error".into())
            ));
        }

        generic_response.data.ok_or(eyre!("data = None"))
    }

    /// Sends the built request without expecting result data.
    pub async fn send(self) -> eyre::Result<()> {
        self.inner_send().await.map(|_| ())
    }

    async fn inner_send(self) -> eyre::Result<Response> {
        reqwest::Client::new()
            .request(self.config.method, self.config.url)
            .bearer_auth(self.config.auth_key)
            .send()
            .await
            .map_err(|e| eyre::eyre!("failed to send request: {}", e))
    }
}

impl RequestBuilder {
    pub fn with_url(mut self, url: String) -> Self {
        self.config.url = url;
        self
    }

    pub fn with_method(mut self, method: Method) -> Self {
        self.config.method = method;
        self
    }

    pub fn with_auth_key(mut self, auth_key: String) -> Self {
        self.config.auth_key = auth_key;
        self
    }

    pub fn build(self) -> Request {
        Request {
            config: self.config,
        }
    }
}

/// Formats a URL using the torbox configuration and a path including query parameters.
pub fn tb_url(config: &TorBoxConfig, path_incl_query: String) -> String {
    format!(
        "{}/{}/{}",
        config.api_base, config.api_version, path_incl_query
    )
}

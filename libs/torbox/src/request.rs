use crate::{GenericTorBoxJson, TorBoxApiState};
use eyre::eyre;
use log::debug;
use reqwest::multipart::{Form, Part};
use reqwest::{Body, Method, Response};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::borrow::Cow;
use std::fmt::Debug;
use std::io;
use std::ops::Deref;
use std::path::Path;

/// Wrapper for [`reqwest::Request`]s allowing easier access to the TorBox API.
pub struct Request<'a, Q>
where
    Q: Serialize + Default,
{
    config: Config<'a, Q>,
}

/// Wrapper for [`Form`]'s allowing easy request building with optional parameters.
pub struct OptionalForm {
    inner: Form,
}

pub struct RequestBuilder<'a, Q>
where
    Q: Serialize + Default,
{
    config: Config<'a, Q>,
}

#[derive(Default)]
struct Config<'a, Q>
where
    Q: Serialize + Default,
{
    url: &'a str,
    method: Method,
    auth_key: &'a str,
    body: Body,
    multipart: Form,
    query: Q,
}

impl<'a, Q: Serialize + Default> Request<'a, Q> {
    pub fn builder() -> RequestBuilder<'a, Q> {
        RequestBuilder {
            config: Config::default(),
        }
    }

    /// Sends the built request expecting a result defined as [`T`].
    pub async fn send_parse<T: DeserializeOwned + Debug>(self) -> eyre::Result<T> {
        let generic_response = self
            .inner_send()
            .await?
            .json::<GenericTorBoxJson<T>>()
            .await
            .map_err(|e| eyre::eyre!("failed to parse response: {}", e))?;

        debug!("response: {:#?}", generic_response);

        // No data present if request is unsuccessful
        if !generic_response.success {
            return Err(eyre!(
                "torbox error: {}",
                generic_response.error.unwrap_or("unknown error".into())
            ));
        }

        generic_response.data.ok_or_else(|| eyre!("data = None"))
    }

    /// Sends the built request without expecting result data.
    pub async fn send(self) -> eyre::Result<()> {
        let resp = self.inner_send().await?;
        debug!("response: {:#?}", resp);
        debug!(
            "response text: {:#?}",
            resp.text().await.unwrap_or_default()
        );
        Ok(())
    }

    async fn inner_send(self) -> eyre::Result<Response> {
        reqwest::Client::new()
            .request(self.config.method, self.config.url)
            .bearer_auth(self.config.auth_key)
            .body(self.config.body)
            .query(&self.config.query)
            .multipart(self.config.multipart)
            .send()
            .await
            .map_err(|e| eyre::eyre!("failed to send request: {}", e))
    }
}

impl<'a, Q: Serialize + Default> RequestBuilder<'a, Q> {
    pub fn with_url(mut self, url: &'a str) -> Self {
        self.config.url = url;
        self
    }

    pub fn with_method(mut self, method: Method) -> Self {
        self.config.method = method;
        self
    }

    pub fn with_auth_key(mut self, auth_key: &'a str) -> Self {
        self.config.auth_key = auth_key;
        self
    }

    pub fn with_body<T: Into<Body>>(mut self, body: T) -> Self {
        self.config.body = body.into();
        self
    }

    pub fn with_multipart(mut self, multipart: Form) -> Self {
        self.config.multipart = multipart;
        self
    }

    pub fn with_query(mut self, query: Q) -> Self {
        self.config.query = query;
        self
    }

    pub fn build(self) -> Request<'a, Q> {
        Request {
            config: self.config,
        }
    }
}

impl OptionalForm {
    pub fn new() -> Self {
        Self {
            inner: Default::default(),
        }
    }

    /// See [`reqwest::multipart::Form::text`]
    pub fn text<T, U>(mut self, name: T, value: Option<U>) -> Self
    where
        T: Into<Cow<'static, str>>,
        U: Into<Cow<'static, str>>,
    {
        if let Some(value) = value {
            self.inner = self.inner.text(name, value);
        }
        self
    }

    /// See [`reqwest::multipart::Form::file`]
    pub async fn file<T, U>(mut self, name: T, path: Option<U>) -> io::Result<Self>
    where
        T: Into<Cow<'static, str>>,
        U: AsRef<Path>,
    {
        if let Some(path) = path {
            self.inner = self.inner.file(name, path).await?;
        }
        Ok(self)
    }

    /// See [`reqwest::multipart::Form::part`]
    pub fn part<T>(mut self, name: T, part: Option<Part>) -> Self
    where
        T: Into<Cow<'static, str>>,
    {
        if let Some(part) = part {
            self.inner = self.inner.part(name, part)
        }
        self
    }

    pub fn inner(self) -> Form {
        self.inner
    }
}

impl Deref for OptionalForm {
    type Target = Form;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

/// Formats a URL using the torbox configuration and a path including query parameters.
pub fn tb_url(config: &TorBoxApiState, path_incl_query: &str) -> String {
    format!(
        "{}/{}/{}",
        config.api_base, config.api_version, path_incl_query
    )
}

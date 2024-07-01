//! A custom [`Client`](reqwest::Client) used by the application for making requests
//! to the user's Plex server
//!
//! The original source for this code is from: https://github.com/seanmonstar/reqwest/issues/988#issuecomment-1475364352

use std::collections::HashMap;

use anyhow::{anyhow, Context, Result};
use reqwest::{header, Url};
use serde::Deserialize;
use simplelog::{debug, error};

use crate::utils;

/// A custom [`Client`](reqwest::Client), with a base url and headers set during creation.
#[derive(Clone, Default, Debug)]
pub struct HttpClient {
    /// The plex server URL
    base_url: String,
    /// Default headers to use with the custom client
    headers: header::HeaderMap,
    /// The user's plex token
    plex_token: String,
    /// The resulting custom client
    client: reqwest::Client,
}

/// Shorthand for headers parameter type
type Params = Option<HashMap<String, String>>;

impl HttpClient {
    /// Creates a new custom ['Client'](reqwest::Client)
    ///
    /// Custom headers and a base url are set during creation
    pub fn new(base_url: &str, plex_token: &str) -> Result<Self> {
        debug!("Creating HTTP client...");

        let mut headers = header::HeaderMap::new();
        headers.append(
            header::ACCEPT,
            header::HeaderValue::from_static("application/json"),
        );

        let client = reqwest::Client::builder()
            .gzip(true)
            .brotli(true)
            .zstd(true)
            .build()?;

        Ok(Self {
            base_url: base_url.to_owned(),
            plex_token: plex_token.to_owned(),
            headers,
            client,
        })
    }

    /// Perform a `GET` request with the custom ['Client'](reqwest::Client)
    pub async fn get<T>(&self, path: &str, params: Params, max_results: Option<i32>) -> Result<T>
    where
        T: for<'de> Deserialize<'de> + Default,
    {
        let url = self.build_final_url(path, params)?;

        let req = self.client.get(url).headers(self.headers.clone());
        let req = if let Some(max_results) = max_results {
            req.header("X-Plex-Container-Size", max_results.to_string())
                .header("X-Plex-Container-Start", "0")
        } else {
            req
        };

        match req.send().await {
            Ok(resp) => {
                let contents = resp.text().await?;
                if contents.is_empty() {
                    return Ok(T::default());
                }

                serde_json::from_str(&contents).with_context(|| {
                    format!(
                        "Unable to deserialise response. Body was: \"{}\"",
                        utils::truncate_string(&contents, 2000)
                    )
                })
            }
            Err(err) => Err(anyhow!("An error occurred while attempting to GET: {err}")),
        }
    }

    /// Perform a `DELETE` request with the custom ['Client'](reqwest::Client)
    pub async fn delete(&self, path: &str, params: Params) -> Result<()> {
        let url = self.build_final_url(path, params)?;
        self.client.delete(url).send().await?;
        Ok(())
    }

    /// Perform a `POST` request with the custom ['Client'](reqwest::Client)
    pub async fn post<T>(&self, path: &str, params: Params) -> Result<T>
    where
        T: for<'de> Deserialize<'de> + Default,
    {
        let url = self.build_final_url(path, params)?;

        match self
            .client
            .post(url)
            .headers(self.headers.clone())
            .send()
            .await
        {
            Ok(resp) => {
                let contents = resp.text().await?;
                if contents.is_empty() {
                    return Ok(T::default());
                }

                serde_json::from_str(&contents).with_context(|| {
                    format!("Unable to deserialise response. Body was: \"{}\"", contents)
                })
            }
            Err(err) => Err(anyhow!("An error occurred while attempting to POST: {err}")),
        }
    }

    /// Perform a `PUT` request with the custom ['Client'](reqwest::Client)
    pub async fn put<T>(&self, path: &str, params: Params) -> Result<T>
    where
        T: for<'de> Deserialize<'de> + Default,
    {
        let url = self.build_final_url(path, params)?;
        match self
            .client
            .put(url)
            .headers(self.headers.clone())
            .send()
            .await
        {
            Ok(resp) => {
                let contents = resp.text().await?;
                if contents.is_empty() {
                    return Ok(T::default());
                }

                serde_json::from_str(&contents).with_context(|| {
                    format!("Unable to deserialise response. Body was: \"{}\"", contents)
                })
            }
            Err(err) => Err(anyhow!("An error occurred while attempting to PUT: {err}")),
        }
    }

    /// Constructs the final URL passed to the respective request
    ///
    /// Merges the base url, the path, and any parameters together
    fn build_final_url(&self, path: &str, params: Params) -> Result<Url> {
        let mut url = Url::parse(&self.base_url)?.join(path)?;

        url.query_pairs_mut()
            .append_pair("X-Plex-Token", &self.plex_token);

        if let Some(params) = params {
            for (k, v) in params {
                url.query_pairs_mut().append_pair(&k, &v);
            }
        }

        debug!("FINAL URL: {url}");

        Ok(url)
    }
}

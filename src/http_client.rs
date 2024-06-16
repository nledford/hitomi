// SOURCE: https://github.com/seanmonstar/reqwest/issues/988#issuecomment-1475364352

use std::collections::HashMap;

use anyhow::{anyhow, Context, Result};
use log::debug;
use reqwest::header;
use serde::Deserialize;

#[derive(Default, Debug)]
pub struct HttpClient {
    base_url: String,
    headers: header::HeaderMap,
    plex_token: String,
    pub client: reqwest::Client,
}

pub type Params = Option<HashMap<String, String>>;

impl HttpClient {
    pub fn new(base_url: &str, plex_token: &str) -> Result<Self> {
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

    pub async fn get<T>(&self, path: &str, params: Params, max_results: Option<i32>) -> Result<T>
    where
        T: for<'de> Deserialize<'de> + Default,
    {
        let url = self.build_final_url(path, params);

        let req = self.client.get(&url).headers(self.headers.clone());
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
                    format!("Unable to deserialise response. Body was: \"{}\"", contents)
                })
            }
            Err(err) => Err(anyhow!("An error occurred while attempting to GET: {err}")),
        }
    }

    pub async fn delete(&self, path: &str, params: Params) -> Result<()> {
        let url = self.build_final_url(path, params);
        self.client.delete(url).send().await?;
        Ok(())
    }

    pub async fn post<T>(&self, path: &str, params: Params) -> Result<T>
    where
        T: for<'de> Deserialize<'de> + Default,
    {
        let url = self.build_final_url(path, params);

        match self
            .client
            .post(&url)
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

    pub async fn put<T>(&self, path: &str, params: Params) -> Result<T>
    where
        T: for<'de> Deserialize<'de> + Default,
    {
        let url = self.build_final_url(path, params);
        match self
            .client
            .put(&url)
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

    fn build_final_url(&self, path: &str, params: Params) -> String {
        let url = format!(
            "{}/{}?X-Plex-Token={}",
            self.base_url, path, self.plex_token
        );

        let url = if let Some(params) = params {
            let params = params
                .iter()
                .map(|(k, v)| format!("{k}={v}"))
                .collect::<Vec<String>>()
                .join("&");
            format!("{url}&{params}")
        } else {
            url
        };

        debug!("{url}");

        url
    }

    /*pub async fn post<T>(&self, path: &str, body: &T) -> Result<Response, Error>
    where
        T: Serialize,
    {
        let url = format!("{}{}", self.base_url, path);
        let resp = self.client
            .post(&url)
            .headers(self.headers.clone())
            .json(body)
            .send()
            .await?;

        Ok(resp)
    }*/
}

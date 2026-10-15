use std::fmt;
use std::str::FromStr;

use reqwest::header::{AUTHORIZATION, HeaderMap, HeaderValue, InvalidHeaderValue, LINK};
use reqwest::{Client, StatusCode};
use serde::Deserialize;
use serde::de::DeserializeOwned;
use url::Url;

use crate::{APP_NAME, APP_VERSION};

#[derive(Clone)]
pub struct Token(HeaderValue);

impl FromStr for Token {
    type Err = InvalidHeaderValue;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut value = HeaderValue::from_str(&format!("Bearer {s}"))?;
        value.set_sensitive(true);
        Ok(Self(value))
    }
}

impl fmt::Debug for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Token(..)")
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Tls {
    Verify,
    Insecure,
}

#[derive(Debug, Clone)]
pub struct BaseUrl(Url);

#[derive(Debug, thiserror::Error)]
pub enum InvalidBaseUrl {
    #[error("{0}")]
    Parse(#[from] url::ParseError),
    #[error("base URL must be http or https: {0}")]
    Scheme(Url),
}

impl BaseUrl {
    pub fn endpoint<'a>(&self, segments: impl IntoIterator<Item = &'a str>) -> Url {
        let mut url = self.0.clone();
        if let Ok(mut path) = url.path_segments_mut() {
            path.pop_if_empty().extend(segments);
        }
        url
    }
}

impl FromStr for BaseUrl {
    type Err = InvalidBaseUrl;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let url = Url::parse(s)?;
        match url.scheme() {
            "http" | "https" => Ok(Self(url)),
            _ => Err(InvalidBaseUrl::Scheme(url)),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum HttpError {
    #[error("{0}")]
    Request(#[from] reqwest::Error),
    #[error("unexpected 404 for {url}: {message:?}")]
    NotFound { url: Url, message: Option<String> },
    #[error("invalid Link header URL {url:?}: {source}")]
    Link {
        url: String,
        source: url::ParseError,
    },
}

pub struct Http {
    client: Client,
}

impl Http {
    pub fn new(token: &Token, tls: Tls) -> Result<Self, HttpError> {
        let mut headers = HeaderMap::new();
        headers.insert(AUTHORIZATION, token.0.clone());
        let client = Client::builder()
            .user_agent(format!("{APP_NAME}/{APP_VERSION}"))
            .default_headers(headers)
            .tls_danger_accept_invalid_certs(matches!(tls, Tls::Insecure))
            .build()?;
        Ok(Self { client })
    }

    pub async fn get<T: DeserializeOwned>(&self, url: Url) -> Result<Option<T>, HttpError> {
        self.get_unless(url, None).await
    }

    pub async fn get_all<T: DeserializeOwned>(
        &self,
        url: Url,
    ) -> Result<Option<Vec<T>>, HttpError> {
        self.get_all_unless(url, None).await
    }

    pub async fn get_unless<T: DeserializeOwned>(
        &self,
        url: Url,
        not_found: Option<&str>,
    ) -> Result<Option<T>, HttpError> {
        match self.send(url, not_found).await? {
            Some(response) => Ok(Some(response.json().await?)),
            None => Ok(None),
        }
    }

    pub async fn get_all_unless<T: DeserializeOwned>(
        &self,
        url: Url,
        not_found: Option<&str>,
    ) -> Result<Option<Vec<T>>, HttpError> {
        let Some(mut response) = self.send(url, not_found).await? else {
            return Ok(None);
        };
        let mut items = Vec::new();
        loop {
            let next = next_link(response.headers())?;
            items.extend(response.json::<Vec<T>>().await?);
            let Some(next) = next else {
                return Ok(Some(items));
            };
            response = self.client.get(next).send().await?.error_for_status()?;
        }
    }

    async fn send(
        &self,
        url: Url,
        not_found: Option<&str>,
    ) -> Result<Option<reqwest::Response>, HttpError> {
        let response = self.client.get(url.clone()).send().await?;
        if response.status() != StatusCode::NOT_FOUND {
            return Ok(Some(response.error_for_status()?));
        }
        let Some(expected) = not_found else {
            return Ok(None);
        };
        let message = response
            .json::<NotFoundBody>()
            .await
            .ok()
            .map(|body| body.message);
        match message {
            Some(message) if message == expected => Ok(None),
            message => Err(HttpError::NotFound { url, message }),
        }
    }
}

#[derive(Deserialize)]
struct NotFoundBody {
    message: String,
}

fn next_link(headers: &HeaderMap) -> Result<Option<Url>, HttpError> {
    let next = headers
        .get_all(LINK)
        .iter()
        .filter_map(|value| value.to_str().ok())
        .flat_map(|value| value.split(','))
        .find_map(|link| {
            let (target, params) = link.split_once(';')?;
            let target = target.trim().strip_prefix('<')?.strip_suffix('>')?;
            params
                .split(';')
                .any(|param| param.trim() == r#"rel="next""#)
                .then_some(target)
        });
    next.map(|url| {
        Url::parse(url).map_err(|source| HttpError::Link {
            url: url.to_owned(),
            source,
        })
    })
    .transpose()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn headers(link: &str) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(LINK, HeaderValue::from_str(link).unwrap());
        headers
    }

    #[test]
    fn next_link_multi_rel() {
        let link = r#"<https://api.github.com/user/repos?page=1>; rel="prev", <https://api.github.com/user/repos?page=3>; rel="next", <https://api.github.com/user/repos?page=5>; rel="last""#;
        assert_eq!(
            next_link(&headers(link)).unwrap().unwrap().as_str(),
            "https://api.github.com/user/repos?page=3"
        );
    }

    #[test]
    fn next_link_without_next() {
        let link = r#"<https://api.github.com/user/repos?page=1>; rel="first", <https://api.github.com/user/repos?page=4>; rel="prev""#;
        assert!(next_link(&headers(link)).unwrap().is_none());
    }

    #[test]
    fn next_link_absent() {
        assert!(next_link(&HeaderMap::new()).unwrap().is_none());
    }
}

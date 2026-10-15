use crate::github::{self, GithubSource};
use crate::gitlab::{self, GitlabSource};
use crate::http::Http;
use crate::repo::{Filter, RemoteRepo};

#[derive(Debug)]
pub enum Source {
    Github(GithubSource),
    Gitlab(GitlabSource),
}

#[derive(Debug, thiserror::Error)]
pub enum SourceError {
    #[error(transparent)]
    Github(#[from] github::Error),
    #[error(transparent)]
    Gitlab(#[from] gitlab::Error),
}

impl Source {
    pub fn provider(&self) -> &'static str {
        match self {
            Self::Github(_) => "github",
            Self::Gitlab(_) => "gitlab",
        }
    }

    pub async fn list(&self, http: &Http, filter: Filter) -> Result<Vec<RemoteRepo>, SourceError> {
        match self {
            Self::Github(source) => Ok(github::list(http, source, filter).await?),
            Self::Gitlab(source) => Ok(gitlab::list(http, source, filter).await?),
        }
    }
}

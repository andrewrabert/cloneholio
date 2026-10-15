use std::str::FromStr;

use jiff::Timestamp;
use serde::Deserialize;
use tracing::warn;
use url::Url;

use crate::http::{BaseUrl, Http, HttpError};
use crate::repo::{BranchName, Filter, RemotePath, RemoteRepo, SshUrl};

pub const DEFAULT_API: &str = "https://api.github.com";

#[derive(Debug)]
pub struct GithubSource {
    pub api: BaseUrl,
    pub selection: Selection,
}

#[derive(Debug)]
pub enum Selection {
    All,
    Targets(Vec<Target>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Target {
    Owner(String),
    Repo { owner: String, name: String },
}

#[derive(Debug, thiserror::Error)]
#[error("invalid GitHub path {0:?}: expected OWNER or OWNER/REPO")]
pub struct InvalidTarget(String);

impl FromStr for Target {
    type Err = InvalidTarget;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let lower = s.to_lowercase();
        let mut parts = lower.split('/');
        match (parts.next(), parts.next(), parts.next()) {
            (Some(owner), None, None) if !owner.is_empty() => Ok(Self::Owner(owner.to_owned())),
            (Some(owner), Some(name), None) if !owner.is_empty() && !name.is_empty() => {
                Ok(Self::Repo {
                    owner: owner.to_owned(),
                    name: name.to_owned(),
                })
            }
            _ => Err(InvalidTarget(s.to_owned())),
        }
    }
}

impl Target {
    fn matches(&self, path: &RemotePath) -> bool {
        let lower = path.as_str().to_lowercase();
        let mut segments = lower.split('/');
        let (owner, name) = (segments.next(), segments.next());
        match self {
            Self::Owner(o) => owner == Some(o.as_str()),
            Self::Repo { owner: o, name: n } => {
                owner == Some(o.as_str()) && name == Some(n.as_str())
            }
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("GitHub API error: {0}")]
    Http(#[from] HttpError),
    #[error("GitHub API returned 404 for {0}")]
    NotFound(Url),
}

#[derive(Clone, Deserialize)]
struct Repo {
    full_name: RemotePath,
    ssh_url: SshUrl,
    pushed_at: Option<Timestamp>,
    default_branch: Option<BranchName>,
    fork: bool,
    archived: bool,
}

impl From<Repo> for RemoteRepo {
    fn from(repo: Repo) -> Self {
        Self {
            path: repo.full_name,
            url: repo.ssh_url,
            updated_at: repo.pushed_at,
            default_branch: repo.default_branch,
        }
    }
}

pub async fn list(
    http: &Http,
    source: &GithubSource,
    filter: Filter,
) -> Result<Vec<RemoteRepo>, Error> {
    let repos = match &source.selection {
        Selection::All => {
            let url = paged(source.api.endpoint(["user", "repos"]), &[]);
            http.get_all::<Repo>(url.clone())
                .await?
                .ok_or(Error::NotFound(url))?
        }
        Selection::Targets(targets) => list_targets(http, &source.api, targets).await?,
    };
    Ok(repos
        .into_iter()
        .filter(|repo| filter.admits(repo.archived, repo.fork))
        .map(RemoteRepo::from)
        .collect())
}

async fn list_targets(http: &Http, api: &BaseUrl, targets: &[Target]) -> Result<Vec<Repo>, Error> {
    let mut private: Option<Vec<Repo>> = None;
    let mut repos = Vec::new();
    for target in targets {
        let mut found = match target {
            Target::Owner(owner) => {
                let url = paged(api.endpoint(["users", owner.as_str(), "repos"]), &[]);
                let mut found = http.get_all::<Repo>(url).await?.unwrap_or_else(|| {
                    warn!("GitHub user not found: {owner}");
                    Vec::new()
                });
                if private.is_none() {
                    let url = paged(
                        api.endpoint(["user", "repos"]),
                        &[("visibility", "private")],
                    );
                    private = Some(http.get_all(url).await?.unwrap_or_default());
                }
                found.extend(
                    private
                        .iter()
                        .flatten()
                        .filter(|repo| target.matches(&repo.full_name))
                        .cloned(),
                );
                found
            }
            Target::Repo { owner, name } => {
                let url = api.endpoint(["repos", owner.as_str(), name.as_str()]);
                match http.get::<Repo>(url).await? {
                    Some(repo) => vec![repo],
                    None => {
                        warn!("GitHub repo not found: {owner}/{name}");
                        Vec::new()
                    }
                }
            }
        };
        found.retain(|repo| target.matches(&repo.full_name));
        repos.append(&mut found);
    }
    Ok(repos)
}

fn paged(mut url: Url, query: &[(&str, &str)]) -> Url {
    url.query_pairs_mut()
        .extend_pairs(query)
        .append_pair("per_page", "100");
    url
}

#[cfg(test)]
mod tests {
    use super::*;

    fn path(s: &str) -> RemotePath {
        s.parse().unwrap()
    }

    #[test]
    fn target_parses_owner() {
        assert_eq!(
            "Owner".parse::<Target>().unwrap(),
            Target::Owner("owner".into())
        );
    }

    #[test]
    fn target_parses_repo() {
        assert_eq!(
            "Owner/Repo".parse::<Target>().unwrap(),
            Target::Repo {
                owner: "owner".into(),
                name: "repo".into(),
            }
        );
    }

    #[test]
    fn target_rejects_invalid() {
        for s in ["", "a/", "a/b/c"] {
            assert!(s.parse::<Target>().is_err(), "{s:?}");
        }
    }

    #[test]
    fn target_matches() {
        let owner = Target::Owner("a".into());
        assert!(owner.matches(&path("A/x")));
        assert!(!owner.matches(&path("ab/x")));
        let repo = Target::Repo {
            owner: "a".into(),
            name: "b".into(),
        };
        assert!(repo.matches(&path("a/B")));
        assert!(!repo.matches(&path("a/c")));
        assert!(!repo.matches(&path("c/b")));
    }
}

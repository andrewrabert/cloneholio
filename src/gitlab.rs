use std::collections::BTreeSet;

use jiff::Timestamp;
use serde::Deserialize;
use serde::de::IgnoredAny;
use tracing::{debug, warn};
use url::Url;

use crate::http::{BaseUrl, Http, HttpError};
use crate::repo::{BranchName, Filter, InvalidRemotePath, RemotePath, RemoteRepo, SshUrl};

pub const DEFAULT_URL: &str = "https://gitlab.com";

#[derive(Debug)]
pub struct GitlabSource {
    pub base: BaseUrl,
    pub paths: Vec<RemotePath>,
    pub all_groups: bool,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("GitLab API error: {0}")]
    Http(#[from] HttpError),
    #[error("GitLab project, user, or group not found: {0}")]
    NotFound(RemotePath),
    #[error("GitLab returned 404 for {0}")]
    Missing(Url),
    #[error("GitLab returned {0}")]
    Path(#[from] InvalidRemotePath),
}

#[derive(Deserialize)]
struct Project {
    path_with_namespace: RemotePath,
    ssh_url_to_repo: SshUrl,
    last_activity_at: Option<Timestamp>,
    default_branch: Option<BranchName>,
    archived: bool,
    #[serde(default)]
    forked_from_project: Option<IgnoredAny>,
}

impl Project {
    fn is_under(&self, namespace: &RemotePath) -> bool {
        let path = self.path_with_namespace.as_str().to_lowercase();
        let prefix = format!("{}/", namespace.as_str().to_lowercase());
        path.starts_with(&prefix)
    }

    fn admitted_by(&self, filter: Filter) -> bool {
        filter.admits(self.archived, self.forked_from_project.is_some())
    }
}

impl From<Project> for RemoteRepo {
    fn from(project: Project) -> Self {
        Self {
            path: project.path_with_namespace,
            url: project.ssh_url_to_repo,
            updated_at: project.last_activity_at,
            default_branch: project.default_branch,
        }
    }
}

#[derive(Deserialize)]
struct Group {
    full_path: String,
}

pub async fn list(
    http: &Http,
    source: &GitlabSource,
    filter: Filter,
) -> Result<Vec<RemoteRepo>, Error> {
    let mut paths: BTreeSet<RemotePath> = source.paths.iter().cloned().collect();
    if source.all_groups {
        debug!("Getting all groups from GitLab");
        paths.extend(top_level_groups(http, &source.base).await?);
    }
    let mut repos = Vec::new();
    for path in &paths {
        repos.extend(projects(http, &source.base, path, filter).await?);
    }
    Ok(repos)
}

async fn top_level_groups(http: &Http, base: &BaseUrl) -> Result<BTreeSet<RemotePath>, Error> {
    let url = paged(
        base.endpoint(["api", "v4", "groups"]),
        &[("all_available", "true")],
    );
    let groups = http
        .get_all::<Group>(url.clone())
        .await?
        .ok_or(Error::Missing(url))?;
    groups
        .iter()
        .filter_map(|group| group.full_path.split('/').next())
        .map(|top| {
            top.to_lowercase()
                .parse::<RemotePath>()
                .map_err(Error::from)
        })
        .collect()
}

async fn projects(
    http: &Http,
    base: &BaseUrl,
    path: &RemotePath,
    filter: Filter,
) -> Result<Vec<RemoteRepo>, Error> {
    let url = base.endpoint(["api", "v4", "projects", path.as_str()]);
    if let Some(project) = http
        .get_unless::<Project>(url, Some("404 Project Not Found"))
        .await?
    {
        if project.path_with_namespace.as_str().to_lowercase() != path.as_str().to_lowercase() {
            warn!(
                "GitLab project {path} redirected to {}",
                project.path_with_namespace
            );
        }
        return Ok(project
            .admitted_by(filter)
            .then(|| project.into())
            .into_iter()
            .collect());
    }
    for (kind, not_found) in [
        ("users", "404 User Not Found"),
        ("groups", "404 Group Not Found"),
    ] {
        let mut query = vec![("include_subgroups", "true")];
        if filter.exclude_archived {
            query.push(("archived", "false"));
        }
        let url = paged(
            base.endpoint(["api", "v4", kind, path.as_str(), "projects"]),
            &query,
        );
        if let Some(found) = http.get_all_unless::<Project>(url, Some(not_found)).await? {
            return Ok(found
                .into_iter()
                .filter(|project| project.admitted_by(filter) && project.is_under(path))
                .map(RemoteRepo::from)
                .collect());
        }
    }
    Err(Error::NotFound(path.clone()))
}

fn paged(mut url: Url, query: &[(&str, &str)]) -> Url {
    url.query_pairs_mut()
        .extend_pairs(query)
        .append_pair("order_by", "id")
        .append_pair("pagination", "keyset")
        .append_pair("per_page", "100");
    url
}

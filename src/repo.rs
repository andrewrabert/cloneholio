use std::fmt;
use std::path::Path;
use std::str::FromStr;

use jiff::Timestamp;
use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize)]
#[serde(try_from = "String")]
pub struct RemotePath(String);

#[derive(Debug, thiserror::Error)]
#[error("invalid path {0:?}")]
pub struct InvalidRemotePath(String);

impl RemotePath {
    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn as_path(&self) -> &Path {
        Path::new(&self.0)
    }

    pub fn starts_with(&self, prefix: &RemotePath) -> bool {
        match self.0.strip_prefix(&prefix.0) {
            Some(rest) => rest.is_empty() || rest.starts_with('/'),
            None => false,
        }
    }
}

impl FromStr for RemotePath {
    type Err = InvalidRemotePath;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.split('/').all(|seg| !matches!(seg, "" | "." | "..")) {
            Ok(Self(s.to_owned()))
        } else {
            Err(InvalidRemotePath(s.to_owned()))
        }
    }
}

impl TryFrom<String> for RemotePath {
    type Error = InvalidRemotePath;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        s.parse()
    }
}

impl fmt::Display for RemotePath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(transparent)]
pub struct SshUrl(String);

impl SshUrl {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(transparent)]
pub struct BranchName(String);

impl BranchName {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for BranchName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Clone)]
pub struct RemoteRepo {
    pub path: RemotePath,
    pub url: SshUrl,
    pub updated_at: Option<Timestamp>,
    pub default_branch: Option<BranchName>,
}

#[derive(Debug, Clone, Copy)]
pub struct Filter {
    pub exclude_archived: bool,
    pub exclude_forks: bool,
}

impl Filter {
    pub fn admits(&self, archived: bool, fork: bool) -> bool {
        !(self.exclude_archived && archived) && !(self.exclude_forks && fork)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn path(s: &str) -> RemotePath {
        s.parse().unwrap()
    }

    #[test]
    fn remote_path_rejects_invalid() {
        for s in ["", "/a", "a//b", "a/./b", "a/../b"] {
            assert!(s.parse::<RemotePath>().is_err(), "{s:?}");
        }
    }

    #[test]
    fn remote_path_starts_with() {
        assert!(path("a/b").starts_with(&path("a")));
        assert!(path("a").starts_with(&path("a")));
        assert!(!path("ab").starts_with(&path("a")));
    }

    #[test]
    fn filter_admits() {
        for exclude_archived in [false, true] {
            for exclude_forks in [false, true] {
                let filter = Filter {
                    exclude_archived,
                    exclude_forks,
                };
                for archived in [false, true] {
                    for fork in [false, true] {
                        let expected = !(exclude_archived && archived) && !(exclude_forks && fork);
                        assert_eq!(filter.admits(archived, fork), expected);
                    }
                }
            }
        }
    }
}

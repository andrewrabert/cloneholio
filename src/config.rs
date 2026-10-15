use std::io;
use std::num::NonZeroUsize;
use std::path::PathBuf;

use crate::cli::{Args, Provider};
use crate::git::Layout;
use crate::github::{self, GithubSource, InvalidTarget, Selection, Target};
use crate::gitlab::{self, GitlabSource};
use crate::http::{BaseUrl, InvalidBaseUrl, Tls, Token};
use crate::log::Output;
use crate::orphan::OrphanPolicy;
use crate::repo::{Filter, InvalidRemotePath, RemotePath};
use crate::source::Source;

#[derive(Debug)]
pub struct Config {
    pub token: Token,
    pub tls: Tls,
    pub source: Source,
    pub filter: Filter,
    pub exclude: Vec<RemotePath>,
    pub output: Output,
    pub action: Action,
}

#[derive(Debug)]
pub enum Action {
    List,
    Sync(SyncPlan),
}

#[derive(Debug)]
pub struct SyncPlan {
    pub directory: PathBuf,
    pub jobs: NonZeroUsize,
    pub layout: Layout,
    pub orphans: OrphanPolicy,
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error(transparent)]
    GithubTarget(#[from] InvalidTarget),
    #[error(transparent)]
    GitlabPath(#[from] InvalidRemotePath),
    #[error("invalid default base URL: {0}")]
    BaseUrl(#[from] InvalidBaseUrl),
    #[error("cannot resolve directory: {0}")]
    Directory(#[from] io::Error),
}

impl Config {
    pub fn resolve(args: Args) -> Result<Self, ConfigError> {
        let source = match args.provider {
            Provider::Github => Source::Github(GithubSource {
                api: base_url(args.base_url, github::DEFAULT_API)?,
                selection: if args.all {
                    Selection::All
                } else {
                    Selection::Targets(
                        args.paths
                            .iter()
                            .map(|path| path.parse::<Target>())
                            .collect::<Result<_, _>>()?,
                    )
                },
            }),
            Provider::Gitlab => Source::Gitlab(GitlabSource {
                base: base_url(args.base_url, gitlab::DEFAULT_URL)?,
                paths: args
                    .paths
                    .iter()
                    .map(|path| path.parse::<RemotePath>())
                    .collect::<Result<_, _>>()?,
                all_groups: args.all,
            }),
        };
        let output = match (args.quiet, args.verbose, args.progress) {
            (true, _, _) => Output::Quiet,
            (_, true, _) => Output::Verbose,
            (_, _, true) => Output::Progress,
            _ => Output::Normal,
        };
        let action = if args.list {
            Action::List
        } else {
            Action::Sync(SyncPlan {
                directory: std::path::absolute(&args.directory)?,
                jobs: args.num_processes,
                layout: if args.mirror {
                    Layout::Mirror { prune: args.prune }
                } else {
                    Layout::Checkout { depth: args.depth }
                },
                orphans: if args.remove_orphans {
                    OrphanPolicy::Remove
                } else {
                    OrphanPolicy::Report
                },
            })
        };
        Ok(Self {
            token: args.token,
            tls: if args.insecure {
                Tls::Insecure
            } else {
                Tls::Verify
            },
            source,
            filter: Filter {
                exclude_archived: args.exclude_archived,
                exclude_forks: args.exclude_forks,
            },
            exclude: args.exclude,
            output,
            action,
        })
    }
}

fn base_url(given: Option<BaseUrl>, default: &str) -> Result<BaseUrl, InvalidBaseUrl> {
    match given {
        Some(url) => Ok(url),
        None => default.parse(),
    }
}

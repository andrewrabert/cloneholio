use std::collections::{BTreeMap, HashSet};
use std::io::{self, IsTerminal};
use std::process::ExitCode;

use futures::StreamExt;
use indicatif::{ProgressBar, ProgressDrawTarget};
use tracing::{debug, error, info, warn};

use crate::config::{Action, Config, SyncPlan};
use crate::git;
use crate::http::{Http, HttpError};
use crate::log::Output;
use crate::orphan::{self, OrphanPolicy};
use crate::repo::{RemotePath, RemoteRepo};
use crate::source::SourceError;

#[derive(Debug, thiserror::Error)]
pub enum RunError {
    #[error(transparent)]
    Http(#[from] HttpError),
    #[error(transparent)]
    Source(#[from] SourceError),
    #[error("orphan scan failed: {0}")]
    Orphan(#[from] io::Error),
}

#[derive(Debug, Clone, Copy)]
pub enum Outcome {
    Clean,
    Failed,
}

impl Outcome {
    pub fn exit_code(self) -> ExitCode {
        match self {
            Self::Clean => ExitCode::SUCCESS,
            Self::Failed => ExitCode::FAILURE,
        }
    }
}

pub async fn run(config: &Config, progress: ProgressBar) -> Result<Outcome, RunError> {
    let provider = config.source.provider();
    if let Action::Sync(plan) = &config.action {
        info!(
            "Begin \"{provider}\" processing using \"{}\"",
            plan.directory.display()
        );
    }
    let http = Http::new(&config.token, config.tls)?;
    let repos = select(config, config.source.list(&http, config.filter).await?);
    match &config.action {
        Action::List => {
            for path in repos.keys() {
                println!("{path}");
            }
            Ok(Outcome::Clean)
        }
        Action::Sync(plan) => sync(provider, plan, config.output, progress, repos).await,
    }
}

fn select(config: &Config, repos: Vec<RemoteRepo>) -> BTreeMap<RemotePath, RemoteRepo> {
    let mut selected = BTreeMap::new();
    for repo in repos {
        if config.exclude.iter().any(|e| repo.path.starts_with(e)) {
            debug!("Excluding repository: {}", repo.path);
        } else {
            selected.insert(repo.path.clone(), repo);
        }
    }
    selected
}

async fn sync(
    provider: &str,
    plan: &SyncPlan,
    output: Output,
    progress: ProgressBar,
    repos: BTreeMap<RemotePath, RemoteRepo>,
) -> Result<Outcome, RunError> {
    let total = repos.len();
    debug!("Syncing {total} repositories with {} jobs", plan.jobs);
    if let Output::Progress = output
        && io::stderr().is_terminal()
    {
        progress.set_length(total as u64);
        progress.set_draw_target(ProgressDrawTarget::stderr());
    }
    let results: Vec<_> = futures::stream::iter(repos.values().map(|repo| {
        let local = plan.directory.join(repo.path.as_path());
        async move {
            let result = git::sync(repo, &local, plan.layout).await;
            if let Err(e) = &result {
                error!("Sync error {}: {e}", repo.path);
            }
            (local, result.is_ok())
        }
    }))
    .buffer_unordered(plan.jobs.get())
    .inspect(|_| progress.inc(1))
    .collect()
    .await;
    progress.finish_and_clear();

    let mut failures = 0;
    let mut local_paths = HashSet::new();
    for (local, ok) in results {
        if !ok {
            failures += 1;
        }
        local_paths.insert(local);
    }

    let orphans = orphan::find(&plan.directory, &local_paths)?;
    let remove = match plan.orphans {
        OrphanPolicy::Remove if repos.is_empty() && !orphans.is_empty() => {
            warn!("Skipping orphan removal because no repositories were selected");
            false
        }
        OrphanPolicy::Remove => true,
        OrphanPolicy::Report => false,
    };
    for path in &orphans {
        let shown = path.strip_prefix(&plan.directory).unwrap_or(path).display();
        if remove {
            warn!("Removing orphan {shown}");
            orphan::remove(path)?;
        } else {
            warn!("Orphan {shown}");
        }
    }

    info!(
        "Finished \"{provider}\" processing {total} repos with {failures} failures and {} orphans",
        orphans.len()
    );
    Ok(match failures {
        0 => Outcome::Clean,
        _ => Outcome::Failed,
    })
}

use std::ffi::{OsStr, OsString};
use std::io;
use std::num::NonZeroU32;
use std::path::Path;
use std::process::{ExitStatus, Stdio};

use filetime::FileTime;
use jiff::Timestamp;
use tokio::process::Command;
use tracing::{debug, info, warn};

use crate::repo::RemoteRepo;

#[derive(Debug, Clone, Copy)]
pub enum Layout {
    Checkout { depth: Option<NonZeroU32> },
    Mirror { prune: bool },
}

#[derive(Debug, thiserror::Error)]
pub enum SyncError {
    #[error(transparent)]
    Git(#[from] GitError),
    #[error(transparent)]
    Io(#[from] io::Error),
}

#[derive(Debug, thiserror::Error)]
#[error("`git {args}` failed ({status}): {stderr}")]
pub struct GitError {
    args: String,
    status: ExitStatus,
    stderr: String,
}

pub async fn sync(repo: &RemoteRepo, local: &Path, layout: Layout) -> Result<(), SyncError> {
    debug!(
        "Processing {} (URL: {}, default branch: {:?}, last activity: {:?})",
        repo.path,
        repo.url.as_str(),
        repo.default_branch,
        repo.updated_at
    );
    let updated_at = repo.updated_at.map(file_time);
    if local.exists() {
        let stamp = FileTime::from_last_modification_time(&local.metadata()?);
        let stamp_current = updated_at == Some(stamp);
        match layout {
            Layout::Mirror { prune } => {
                if stamp_current {
                    debug!("{} is up to date", repo.path);
                } else {
                    update_mirror(repo, local, prune).await?;
                }
            }
            Layout::Checkout { .. } => {
                let head = git(Some(local), ["symbolic-ref", "--short", "HEAD"]).await?;
                let on_default = repo.default_branch.as_ref().map(|b| b.as_str()) == Some(&*head);
                if stamp_current && on_default {
                    debug!("{} is up to date", repo.path);
                } else {
                    update_checkout(repo, local, &head).await?;
                }
            }
        }
    } else {
        clone(repo, local, layout).await?;
    }
    if let Some(updated_at) = updated_at {
        filetime::set_file_mtime(local, updated_at)?;
    }
    Ok(())
}

async fn clone(repo: &RemoteRepo, local: &Path, layout: Layout) -> Result<(), SyncError> {
    info!("Cloning {}", repo.path);
    let mut args: Vec<OsString> = vec!["clone".into()];
    match layout {
        Layout::Checkout { depth: Some(depth) } => {
            args.extend(["--depth".into(), depth.to_string().into()]);
        }
        Layout::Checkout { depth: None } => {}
        Layout::Mirror { .. } => args.push("--mirror".into()),
    }
    args.extend([repo.url.as_str().into(), local.into()]);
    git(None, args).await?;
    if let Layout::Mirror { .. } = layout {
        git(Some(local), ["config", "remote.origin.mirror", "false"]).await?;
    }
    Ok(())
}

async fn update_mirror(repo: &RemoteRepo, local: &Path, prune: bool) -> Result<(), SyncError> {
    set_remote_urls(repo, local).await?;
    info!("Fetching {}", repo.path);
    let mut args = vec!["fetch"];
    if prune {
        args.push("--prune");
    }
    args.extend(["origin", "+refs/*:refs/*"]);
    git(Some(local), args).await?;
    git(Some(local), ["gc", "--auto"]).await?;
    Ok(())
}

async fn update_checkout(repo: &RemoteRepo, local: &Path, head: &str) -> Result<(), SyncError> {
    for remote in set_remote_urls(repo, local).await? {
        git(Some(local), ["fetch", remote.as_str()]).await?;
    }
    let branches = git(
        Some(local),
        ["for-each-ref", "--format=%(refname)", "refs/heads"],
    )
    .await?;
    if !branches.is_empty() {
        if let Some(default) = &repo.default_branch
            && default.as_str() != head
        {
            warn!("Switching from {head} to {default}");
            git(Some(local), ["checkout", default.as_str()]).await?;
        }
        info!("Pulling {}", repo.path);
        git(Some(local), ["pull", "origin"]).await?;
    }
    git(Some(local), ["gc", "--auto"]).await?;
    Ok(())
}

async fn set_remote_urls(repo: &RemoteRepo, local: &Path) -> Result<Vec<String>, SyncError> {
    let remotes: Vec<String> = git(Some(local), ["remote"])
        .await?
        .lines()
        .map(str::to_owned)
        .collect();
    for remote in &remotes {
        debug!("Updating remote {remote} of {}", repo.path);
        git(
            Some(local),
            ["remote", "set-url", remote.as_str(), repo.url.as_str()],
        )
        .await?;
    }
    Ok(remotes)
}

async fn git<I, S>(dir: Option<&Path>, args: I) -> Result<String, SyncError>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let args: Vec<S> = args.into_iter().collect();
    let mut command = Command::new("git");
    command.env("GIT_TERMINAL_PROMPT", "0");
    if std::env::var_os("GIT_SSH_COMMAND").is_none() {
        command.env("GIT_SSH_COMMAND", "ssh -o BatchMode=yes");
    }
    if let Some(dir) = dir {
        command.arg("-C").arg(dir);
        if let Some(parent) = dir.parent() {
            command.env("GIT_CEILING_DIRECTORIES", parent);
        }
    }
    let output = command.args(&args).stdin(Stdio::null()).output().await?;
    if !output.status.success() {
        return Err(GitError {
            args: args
                .iter()
                .map(|arg| arg.as_ref().to_string_lossy())
                .collect::<Vec<_>>()
                .join(" "),
            status: output.status,
            stderr: String::from_utf8_lossy(&output.stderr).trim().to_owned(),
        }
        .into());
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_owned())
}

fn file_time(timestamp: Timestamp) -> FileTime {
    let nanos = timestamp.as_nanosecond();
    FileTime::from_unix_time(
        nanos.div_euclid(1_000_000_000) as i64,
        nanos.rem_euclid(1_000_000_000) as u32,
    )
}

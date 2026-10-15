use std::num::{NonZeroU32, NonZeroUsize};
use std::path::PathBuf;

use clap::{Parser, ValueEnum};

use crate::http::{BaseUrl, Token};
use crate::repo::RemotePath;
use crate::{APP_NAME, APP_VERSION};

const TOKEN_HELP: &str = "\
Token creation:
  - GitLab
    Permissions:  api
    URL:  https://gitlab.com/profile/personal_access_tokens

  - GitHub
    Permissions:  repo:status
    URL:  https://github.com/settings/tokens/new";

#[derive(Debug, Parser)]
#[command(
    name = APP_NAME,
    version = APP_VERSION,
    about = "Maintain local backups of all Git repositories belonging to a user or group.",
    after_help = TOKEN_HELP
)]
pub struct Args {
    /// Number of repositories to sync concurrently
    #[arg(short = 'n', long = "num-processes", default_value = "1")]
    pub num_processes: NonZeroUsize,

    #[arg(short, long, help_heading = "Remote configuration")]
    pub token: Token,

    #[arg(short, long, value_enum, help_heading = "Remote configuration")]
    pub provider: Provider,

    /// Ignore SSL errors
    #[arg(long, help_heading = "Remote configuration")]
    pub insecure: bool,

    #[arg(short = 'u', long, help_heading = "Remote configuration")]
    pub base_url: Option<BaseUrl>,

    /// Paths to exclude from backup
    #[arg(short, long, help_heading = "Remote configuration")]
    pub exclude: Vec<RemotePath>,

    /// Exclude archived repositories
    #[arg(long, help_heading = "Remote configuration")]
    pub exclude_archived: bool,

    /// Exclude repositories that are forks
    #[arg(long, help_heading = "Remote configuration")]
    pub exclude_forks: bool,

    #[arg(short, long, default_value = ".", help_heading = "Local configuration")]
    pub directory: PathBuf,

    /// Remove orphaned directories
    #[arg(long, help_heading = "Local configuration")]
    pub remove_orphans: bool,

    /// Corresponds to the git clone --depth option
    #[arg(long, conflicts_with = "mirror")]
    pub depth: Option<NonZeroU32>,

    /// Maintain bare mirrors (git clone --mirror). Captures all refs; no working tree.
    #[arg(long)]
    pub mirror: bool,

    /// With --mirror, prune local refs deleted upstream. Default keeps them.
    #[arg(long, requires = "mirror")]
    pub prune: bool,

    /// Suppress informational output
    #[arg(short, long, conflicts_with_all = ["verbose", "progress"])]
    pub quiet: bool,

    /// Enable debug logs
    #[arg(short, long, conflicts_with = "progress")]
    pub verbose: bool,

    /// Show progress bar
    #[arg(long)]
    pub progress: bool,

    /// List remote repositories then exit
    #[arg(long)]
    pub list: bool,

    /// Get all groups (GitLab) or organizations (GitHub)
    #[arg(long)]
    pub all: bool,

    #[arg(required_unless_present = "all")]
    pub paths: Vec<String>,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum Provider {
    Github,
    Gitlab,
}

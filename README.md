# cloneholio

[![AUR Version](https://img.shields.io/aur/version/cloneholio.svg)](https://aur.archlinux.org/packages/cloneholio)

Maintain local backups of *all Git repositories* belonging to a user or group.

**Features:**

- Supports both GitHub and GitLab.
- Back up *all repositories* owned by users, groups, and subgroups.
- Back up individual repositories.
- Sync a configurable number of repositories concurrently.

## Installation

* [Arch Linux](https://aur.archlinux.org/packages/cloneholio/)
* From source: `cargo install --git https://github.com/andrewrabert/cloneholio`

## Token Setup

**GitHub**

Create a [personal access token (Tokens (classic))](https://github.com/settings/tokens)) with the following permissions:
- `repo:status`

**GitLab**

Create a [personal access token](https://gitlab.com/profile/personal_access_tokens) with the following permissions:
- `api` (Access the authenticated user's API)

## Example

This will back up all repositories owned by the [python](https://github.com/python) organization on GitHub.

```
$ cloneholio -t TOKEN -p github python
 INFO Begin "github" processing using "/home/draje/Code/GitLab/nvllsvm/cloneholio"
 INFO Cloning python/asyncio
...
 INFO Pulling python/typing
 INFO Finished "github" processing 62 repos with 0 failures and 0 orphans
```

## Help

```
$ cloneholio -h
Maintain local backups of all Git repositories belonging to a user or group.

Usage: cloneholio [OPTIONS] --token <TOKEN> --provider <PROVIDER> [PATHS]...

Arguments:
  [PATHS]...

Options:
  -n, --num-processes <NUM_PROCESSES>  Number of repositories to sync concurrently [default: 1]
      --depth <DEPTH>                  Corresponds to the git clone --depth option
      --mirror                         Maintain bare mirrors (git clone --mirror). Captures all refs; no working tree
      --prune                          With --mirror, prune local refs deleted upstream. Default keeps them
  -q, --quiet                          Suppress informational output
  -v, --verbose                        Enable debug logs
      --progress                       Show progress bar
      --list                           List remote repositories then exit
      --all                            Get all groups (GitLab) or organizations (GitHub)
  -h, --help                           Print help
  -V, --version                        Print version

Remote configuration:
  -t, --token <TOKEN>
  -p, --provider <PROVIDER>  [possible values: github, gitlab]
      --insecure             Ignore SSL errors
  -u, --base-url <BASE_URL>
  -e, --exclude <EXCLUDE>    Paths to exclude from backup
      --exclude-archived     Exclude archived repositories
      --exclude-forks        Exclude repositories that are forks

Local configuration:
  -d, --directory <DIRECTORY>  [default: .]
      --remove-orphans         Remove orphaned directories

Token creation:
  - GitLab
    Permissions:  api
    URL:  https://gitlab.com/profile/personal_access_tokens

  - GitHub
    Permissions:  repo:status
    URL:  https://github.com/settings/tokens/new
```

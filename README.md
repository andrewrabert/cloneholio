# cloneholio

[![PyPI Version](https://img.shields.io/pypi/v/cloneholio.svg)](https://pypi.org/pypi/cloneholio) [![AUR Version](https://img.shields.io/aur/version/cloneholio.svg)](https://aur.archlinux.org/packages/cloneholio)

Maintain local backups of *all Git repositories* belonging to a user or group.

**Features:**

- Supports both GitHub and GitLab.
- Back up *all repositories* owned by users, groups, and subgroups.
- Back up individual repositories.
- Scale to a configurable number of processes.

## Installation

* [Arch Linux](https://aur.archlinux.org/packages/cloneholio/)
* [PyPI](https://pypi.org/pypi/cloneholio)

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
INFO Processing python/asyncio
INFO Processing python/bpo-builder
...
INFO Processing python/typing
INFO Finished "github" processing 62 repos with 0 failures
```

## Help

```
$ cloneholio -h
usage: cloneholio [-h] [-n NUM_PROCESSES] -t TOKEN [-p {github,gitlab}]
                  [--insecure] [-u BASE_URL] [-e EXCLUDE] [--exclude-archived]
                  [--exclude-forks] [-d DIRECTORY] [--remove-orphans]
                  [--depth DEPTH | --mirror] [--prune] [-q | -v | --progress]
                  [--list] [--version] [--all]
                  [paths ...]

Maintain local backups of all Git repositories belonging to a user or group.

Token creation:
  - GitLab
    Permissions:  api
    URL:  https://gitlab.com/profile/personal_access_tokens

  - GitHub
    Permissions:  repo:status
    URL:  https://github.com/settings/tokens/new

positional arguments:
  paths

options:
  -h, --help            show this help message and exit
  -n, --num-processes NUM_PROCESSES
                        Number of processes to use
  --depth DEPTH         Corresponds to the git clone --depth option
  --mirror              Maintain bare mirrors (git clone --mirror).
                        Captures all refs; no working tree.
  --prune               With --mirror, prune local refs deleted upstream.
                        Default keeps them.
  -q, --quiet           Suppress informational output
  -v, --verbose         Enable debug logs
  --progress            Show progress bar
  --list                List remote repositories then exit.
  --version             show program's version number and exit
  --all                 Get all groups (GitLab) or organizations (GitHub)

remote configuration:
  -t, --token TOKEN
  -p, --provider {github,gitlab}
  --insecure            Ignore SSL errors
  -u, --base-url BASE_URL
  -e, --exclude EXCLUDE
                        Paths to exclude from backup
  --exclude-archived    exclude archived repositories
  --exclude-forks       exclude repositories that are forks

local configuration:
  -d, --directory DIRECTORY
  --remove-orphans      Remove orphaned directories
```

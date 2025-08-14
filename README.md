# cloneholio

[![PyPI Version](https://img.shields.io/pypi/v/cloneholio.svg)](https://pypi.org/pypi/cloneholio) [![AUR Version](https://img.shields.io/aur/version/cloneholio.svg)](https://aur.archlinux.org/packages/cloneholio)

Maintain local backups of *all Git repositories* belonging to a user or group.

**Features:**

- Supports both GitHub and GitLab.
- Backup *all repositories* owned by users, groups, and subgroups.
- Backup individual repositories.
- Scale to a configurable number of processes.

## Installation

* [Arch Linux](https://aur.archlinux.org/packages/cloneholio/)
* [PyPI](https://pypi.org/pypi/cloneholio)

## Example

This will backup all repositories owned by the [python](https://github.com/python) organization on GitHub.

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
usage: cloneholio [-h] [-n NUM_PROCESSES] [-d DIRECTORY] -t TOKEN
                  [-p {github,gitlab}] [--depth DEPTH] [--insecure]
                  [-u BASE_URL] [--version]
                  paths [paths ...]

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

optional arguments:
  -h, --help            show this help message and exit
  -n NUM_PROCESSES      Number of processes to use
  -d DIRECTORY, --directory DIRECTORY
  -t TOKEN, --token TOKEN
  -p {github,gitlab}, --provider {github,gitlab}
  --depth DEPTH         Corresponds to the git clone --depth option
  --insecure            Ignore SSL errors
  -u BASE_URL, --base-url BASE_URL
  --version             show program's version number and exit
```
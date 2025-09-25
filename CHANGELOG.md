# Changelog

## 0.15.1 (2025-09-25)
- Fix logging of some errors

## 0.15.0 (2025-09-22)
- Add support for GitHub Enterprise
- Add `--all` flag
- Remove `--all-groups` (replaced with `--all`)

## 0.14.0 (2025-08-14)
- Require Python>=3.10
- Replace `pkg_resources` with `importlib.metadata`

## 0.13.4 (2024-03-12)
- Fix private GitHub repo retrieval

## 0.13.3 (2024-03-12)
- Fix Python package

## 0.13.2 (2024-03-12)
- Fix specific GitLab project retrieval

## 0.13.1 (2023-03-23)
- Fix when GitHub user is not found

## 0.13.0 (2022-04-09)
- Change to default branch when local mismatches remote
- Exit non-zero when failure(s)
- Fix errors being reported as orphans
- Fix for repos that have no branches
- Fix forks and archived missing from github
- Fix missing log message upon error
- Fix when most recent update time is null
- Prevent unhandled exceptions from breaking run
- Update remote url

## 0.12.0 (2021-07-15)
- Increase performance by only executing git when the remote reports a change. 
  The local repo's **mtime** is set to the time of the remote's last update.
- Fix GitLab pagination

## 0.11.0 (2019-12-05)
- Add `--all-groups` option to retrieve repos from all available groups (GitLab only)

## 0.10.0 (2019-10-29)
- Add progress bar

## 0.9.1 (2019-10-14)
- Fix Gitlab error when retrieving subgroup

## 0.9.0 (2019-10-10)
- Add `--exclude-forks` option to exclude repositories that are forks

## 0.8.1 (2019-09-13)
- Fix group fork filtering

## 0.8.0 (2019-08-28)
- Add `--exclude-archived` option to exclude archived repositories

## 0.7.2 (2019-08-12)
- Fix typo

## 0.7.1 (2019-08-12)
- Fix insecure HTTPS errors when using --insecure

## 0.7.0 (2019-08-08)
- Add option to list remote repos then exit
- Ignore error when argument GitHub repository is not found
- Refactor into package
- Remove `consumers` dependency

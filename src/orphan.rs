use std::collections::HashSet;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy)]
pub enum OrphanPolicy {
    Report,
    Remove,
}

pub fn find(root: &Path, repos: &HashSet<PathBuf>) -> io::Result<Vec<PathBuf>> {
    if !root.try_exists()? {
        return Ok(Vec::new());
    }
    let parents: HashSet<&Path> = repos
        .iter()
        .flat_map(|path| path.ancestors().skip(1))
        .collect();
    let mut orphans = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        for entry in fs::read_dir(&dir)? {
            let entry = entry?;
            let path = entry.path();
            if repos.contains(&path) {
                continue;
            }
            if !parents.contains(path.as_path()) {
                orphans.push(path);
            } else if entry.file_type()?.is_dir() {
                stack.push(path);
            }
        }
    }
    orphans.sort();
    Ok(orphans)
}

pub fn remove(path: &Path) -> io::Result<()> {
    if fs::symlink_metadata(path)?.is_dir() {
        fs::remove_dir_all(path)
    } else {
        fs::remove_file(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_root(name: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!(
            "cloneholio-test-{name}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let _ = fs::remove_dir_all(&root);
        root
    }

    #[test]
    fn find_orphans() {
        let root = temp_root("find");
        let repo = root.join("group/sub/repo");
        fs::create_dir_all(&repo).unwrap();
        fs::create_dir_all(root.join("group/stray")).unwrap();
        fs::create_dir_all(root.join("other")).unwrap();
        fs::write(root.join("group/file"), "").unwrap();
        let repos = HashSet::from([repo]);
        let found = find(&root, &repos).unwrap();
        fs::remove_dir_all(&root).unwrap();
        assert_eq!(
            found,
            vec![
                root.join("group/file"),
                root.join("group/stray"),
                root.join("other")
            ]
        );
    }

    #[test]
    fn find_missing_root() {
        let root = temp_root("missing");
        assert!(find(&root, &HashSet::new()).unwrap().is_empty());
    }
}

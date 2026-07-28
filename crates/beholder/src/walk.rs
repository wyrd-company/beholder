//! Sources of files to analyze.
//!
//! Two sources exist and both yield the same shape: repo-relative path plus
//! bytes, in a deterministic, stably sorted order.
//!
//! - [`walk_worktree`] reads the working tree, honoring `.gitignore` and the
//!   configured ignores.
//! - [`read_revision`] reads a git revision's tree, so delta mode never needs a
//!   checkout.

use std::path::Path;

use anyhow::{Context, Result};
use ignore::WalkBuilder;

use crate::config::Config;

/// One file offered to analysis.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceFile {
    /// Repo-relative, forward-slash separated. Never absolute.
    pub path: String,
    pub contents: String,
}

/// Walk a working tree, honoring `.gitignore` and the configured ignores.
///
/// Files that are not valid UTF-8 are skipped: no tier can read them.
pub fn walk_worktree(root: &Path, config: &Config) -> Result<Vec<SourceFile>> {
    let mut files = Vec::new();

    let walker = WalkBuilder::new(root)
        .hidden(false)
        .git_ignore(true)
        .git_global(true)
        .git_exclude(true)
        .parents(true)
        .build();

    for entry in walker {
        let entry = entry.context("walking the repository")?;

        if !entry.file_type().is_some_and(|t| t.is_file()) {
            continue;
        }

        let Ok(relative) = entry.path().strip_prefix(root) else {
            continue;
        };
        let path = to_repo_relative(relative);

        if path == ".gitignore" || path.starts_with(".git/") || !config.accepts(&path) {
            continue;
        }

        match std::fs::read(entry.path()) {
            Ok(bytes) => {
                if let Ok(contents) = String::from_utf8(bytes) {
                    files.push(SourceFile { path, contents });
                }
            }
            Err(err) => {
                return Err(err).with_context(|| format!("reading {}", entry.path().display()))
            }
        }
    }

    sort_stably(&mut files);
    Ok(files)
}

/// Read every blob in a git revision's tree, filtered by the same configuration.
pub fn read_revision(repo: &git2::Repository, revision: &str, config: &Config) -> Result<Vec<SourceFile>> {
    let object = repo
        .revparse_single(revision)
        .with_context(|| format!("resolving revision {revision}"))?;
    let commit = object
        .peel_to_commit()
        .with_context(|| format!("{revision} does not name a commit"))?;
    let tree = commit.tree().context("reading the commit tree")?;

    let mut files = Vec::new();
    let mut walk_error: Option<anyhow::Error> = None;

    tree.walk(git2::TreeWalkMode::PreOrder, |dir, entry| {
        if entry.kind() != Some(git2::ObjectType::Blob) {
            return git2::TreeWalkResult::Ok;
        }

        let Ok(name) = entry.name() else {
            return git2::TreeWalkResult::Ok;
        };
        let path = format!("{dir}{name}");

        if !config.accepts(&path) {
            return git2::TreeWalkResult::Ok;
        }

        match repo.find_blob(entry.id()) {
            Ok(blob) => {
                if let Ok(contents) = std::str::from_utf8(blob.content()) {
                    files.push(SourceFile {
                        path,
                        contents: contents.to_owned(),
                    });
                }
                git2::TreeWalkResult::Ok
            }
            Err(err) => {
                walk_error = Some(anyhow::Error::new(err).context(format!("reading blob {path}")));
                git2::TreeWalkResult::Abort
            }
        }
    })
    .context("walking the revision tree")?;

    if let Some(err) = walk_error {
        return Err(err);
    }

    sort_stably(&mut files);
    Ok(files)
}

/// Deterministic order: byte order of the repo-relative path.
fn sort_stably(files: &mut [SourceFile]) {
    files.sort_by(|a, b| a.path.cmp(&b.path));
}

fn to_repo_relative(path: &Path) -> String {
    path.components()
        .map(|c| c.as_os_str().to_string_lossy().into_owned())
        .collect::<Vec<_>>()
        .join("/")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write(root: &Path, path: &str, contents: &str) {
        let full = root.join(path);
        std::fs::create_dir_all(full.parent().unwrap()).unwrap();
        std::fs::write(full, contents).unwrap();
    }

    #[test]
    fn honors_gitignore_and_configured_ignores() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        git2::Repository::init(root).unwrap();

        write(root, ".gitignore", "target/\n");
        write(root, "src/main.rs", "fn main() {}\n");
        write(root, "target/debug/build.rs", "fn built() {}\n");
        write(root, "vendor/dep.rs", "fn dep() {}\n");
        write(root, "Cargo.lock", "# lock\n");

        let config = Config::from_toml(
            r#"
            ignored_extensions = ["lock"]
            ignored_paths = ["vendor/"]
        "#,
        )
        .unwrap();

        let paths: Vec<_> = walk_worktree(root, &config)
            .unwrap()
            .into_iter()
            .map(|f| f.path)
            .collect();

        assert_eq!(paths, vec!["src/main.rs".to_string()]);
    }

    #[test]
    fn output_is_stably_sorted() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        git2::Repository::init(root).unwrap();

        for name in ["z.rs", "a.rs", "m/n.rs", "m/a.rs"] {
            write(root, name, "fn f() {}\n");
        }

        let paths: Vec<_> = walk_worktree(root, &Config::default())
            .unwrap()
            .into_iter()
            .map(|f| f.path)
            .collect();

        assert_eq!(paths, vec!["a.rs", "m/a.rs", "m/n.rs", "z.rs"]);
    }
}

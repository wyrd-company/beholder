//! Sources of files to analyze.
//!
//! Two sources exist and both yield the same shape: repo-relative path plus
//! bytes, in a deterministic, stably sorted order.
//!
//! - [`walk_worktree`] reads the working tree.
//! - [`read_revision`] reads a git revision's tree, so delta mode never needs a
//!   checkout.
//!
//! Both apply the same [`PathRules`]. That matters more than it looks: `index`
//! reads the working tree and `delta` reads revisions, and if they selected
//! different files for the same content then a symbol could appear or vanish
//! purely because of which command asked.

use std::path::Path;

use anyhow::{Context, Result};
use ignore::gitignore::{Gitignore, GitignoreBuilder};
use ignore::WalkBuilder;

use crate::config::Config;

/// One file offered to analysis.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceFile {
    /// Repo-relative, forward-slash separated. Never absolute.
    pub path: String,
    pub contents: String,
}

/// Which files analysis is allowed to see.
///
/// Three layers, in order: files that describe the repository or that beholder
/// itself produced, `.gitignore`, and the configured ignores.
pub struct PathRules<'c> {
    config: &'c Config,
    /// One matcher per directory that holds a `.gitignore`, shallowest first.
    /// A single matcher cannot express per-directory scoping: a bare `*.rs` in
    /// `src/generated/.gitignore` applies to that subtree, not to the repository.
    ignores: Vec<(String, Gitignore)>,
}

impl<'c> PathRules<'c> {
    /// Rules for a working tree. The walker applies `.gitignore` itself, so no
    /// ignore matcher is built here.
    pub fn for_worktree(config: &'c Config) -> Self {
        Self {
            config,
            ignores: Vec::new(),
        }
    }

    /// Rules for a committed tree, using the `.gitignore` files in that tree.
    pub fn for_revision(config: &'c Config, ignores: Vec<(String, Gitignore)>) -> Self {
        Self { config, ignores }
    }

    /// Should this repo-relative path be analyzed?
    pub fn selects(&self, path: &str) -> bool {
        if is_infrastructure(path) || !self.config.accepts(path) {
            return false;
        }

        !self.ignored(path)
    }

    /// Deepest applicable `.gitignore` wins, as git resolves it, so a nested
    /// negation can re-include what a shallower rule excluded.
    fn ignored(&self, path: &str) -> bool {
        let mut ignored = false;

        for (directory, matcher) in &self.ignores {
            let Some(relative) = strip_directory(path, directory) else {
                continue;
            };
            match matcher.matched_path_or_any_parents(relative, false) {
                ignore::Match::Ignore(_) => ignored = true,
                ignore::Match::Whitelist(_) => ignored = false,
                ignore::Match::None => {}
            }
        }

        ignored
    }
}

/// A path relative to a directory, or `None` when it is not inside it.
fn strip_directory<'p>(path: &'p str, directory: &str) -> Option<&'p str> {
    if directory.is_empty() {
        return Some(path);
    }
    path.strip_prefix(directory)?.strip_prefix('/')
}

/// Files that describe the repository or that beholder itself produced.
///
/// Indexing beholder's own output would make a second run over an unchanged
/// working tree report new files, which is confusing and useless.
fn is_infrastructure(path: &str) -> bool {
    path == ".gitignore"
        || path.ends_with("/.gitignore")
        || path.starts_with(".git/")
        || path == "symbols.jsonl"
        || path == "files.jsonl"
}

/// Walk a working tree, honoring `.gitignore` and the configured ignores.
///
/// Files that are not valid UTF-8 are skipped: no tier can read them.
pub fn walk_worktree(root: &Path, config: &Config) -> Result<Vec<SourceFile>> {
    let rules = PathRules::for_worktree(config);
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

        if !rules.selects(&path) {
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

/// Read every analyzable blob in a git revision's tree.
pub fn read_revision(
    repo: &git2::Repository,
    revision: &str,
    config: &Config,
) -> Result<Vec<SourceFile>> {
    let tree = revision_tree(repo, revision)?;
    let rules = PathRules::for_revision(config, revision_ignores(repo, &tree)?);

    let mut files = Vec::new();

    for_each_blob(repo, &tree, &mut |path, blob| {
        if !rules.selects(path) {
            return Ok(());
        }
        if let Ok(contents) = std::str::from_utf8(blob) {
            files.push(SourceFile {
                path: path.to_owned(),
                contents: contents.to_owned(),
            });
        }
        Ok(())
    })?;

    sort_stably(&mut files);
    Ok(files)
}

/// Every analyzable path in a revision, with the content hash of its blob.
///
/// This is what a stored analysis has to agree with before it can be treated as
/// the analysis of that revision.
pub fn revision_manifest(
    repo: &git2::Repository,
    revision: &str,
    config: &Config,
) -> Result<std::collections::BTreeMap<String, String>> {
    Ok(read_revision(repo, revision, config)?
        .into_iter()
        .map(|file| {
            let hash = crate::hash::content_hash(file.contents.as_bytes());
            (file.path, hash)
        })
        .collect())
}

/// The same manifest, taken from a working tree.
pub fn worktree_manifest(
    root: &Path,
    config: &Config,
) -> Result<std::collections::BTreeMap<String, String>> {
    Ok(walk_worktree(root, config)?
        .into_iter()
        .map(|file| {
            let hash = crate::hash::content_hash(file.contents.as_bytes());
            (file.path, hash)
        })
        .collect())
}

/// The tree of a revision.
pub fn revision_tree<'r>(repo: &'r git2::Repository, revision: &str) -> Result<git2::Tree<'r>> {
    repo.revparse_single(revision)
        .with_context(|| format!("resolving revision {revision}"))?
        .peel_to_commit()
        .with_context(|| format!("{revision} does not name a commit"))?
        .tree()
        .context("reading the commit tree")
}

/// Build one `.gitignore` matcher per directory that holds one, from the
/// `.gitignore` files inside a tree.
///
/// The working-tree walk honors `.gitignore`; a committed tree has to be given
/// the same treatment explicitly or the two walkers disagree.
fn revision_ignores(
    repo: &git2::Repository,
    tree: &git2::Tree<'_>,
) -> Result<Vec<(String, Gitignore)>> {
    let mut sources = Vec::new();

    for_each_blob(repo, tree, &mut |path, blob| {
        if path == ".gitignore" || path.ends_with("/.gitignore") {
            if let Ok(text) = std::str::from_utf8(blob) {
                let directory = path.rsplit_once('/').map(|(dir, _)| dir).unwrap_or("");
                sources.push((directory.to_owned(), text.to_owned()));
            }
        }
        Ok(())
    })?;

    // Shallowest first, so a deeper file is evaluated last and wins.
    sources.sort_by_key(|(directory, _)| (directory.matches('/').count(), directory.clone()));

    let mut matchers = Vec::with_capacity(sources.len());

    for (directory, text) in &sources {
        let mut builder = GitignoreBuilder::new("");
        for line in text.lines() {
            builder
                .add_line(None, line)
                .with_context(|| format!("parsing .gitignore in {directory:?}"))?;
        }
        matchers.push((
            directory.clone(),
            builder.build().context("building an ignore matcher")?,
        ));
    }

    Ok(matchers)
}

/// Is this tree entry a regular file?
///
/// Git stores a symlink as a blob holding its target, and a submodule as a
/// gitlink. The working-tree walk sees neither as a file, so the revision walk
/// must not either — otherwise `delta` analyzes a symlink's target path as if it
/// were source and `index` does not.
fn is_regular_file(entry: &git2::TreeEntry<'_>) -> bool {
    entry.kind() == Some(git2::ObjectType::Blob)
        && matches!(
            entry.filemode(),
            0o100644 /* blob */ | 0o100755 /* executable blob */
        )
}

/// Visit every blob in a tree with its repo-relative path.
fn for_each_blob(
    repo: &git2::Repository,
    tree: &git2::Tree<'_>,
    visit: &mut dyn FnMut(&str, &[u8]) -> Result<()>,
) -> Result<()> {
    let mut error: Option<anyhow::Error> = None;

    tree.walk(git2::TreeWalkMode::PreOrder, |dir, entry| {
        if !is_regular_file(entry) {
            return git2::TreeWalkResult::Ok;
        }

        let Ok(name) = entry.name() else {
            return git2::TreeWalkResult::Ok;
        };
        let path = format!("{dir}{name}");

        match repo
            .find_blob(entry.id())
            .map_err(anyhow::Error::new)
            .and_then(|blob| visit(&path, blob.content()))
        {
            Ok(()) => git2::TreeWalkResult::Ok,
            Err(err) => {
                error = Some(err.context(format!("reading blob {path}")));
                git2::TreeWalkResult::Abort
            }
        }
    })
    .context("walking the revision tree")?;

    match error {
        Some(err) => Err(err),
        None => Ok(()),
    }
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

    /// A repository with the same files in the worktree and in a commit.
    fn fixture(files: &[(&str, &str)]) -> (tempfile::TempDir, git2::Repository) {
        let dir = tempfile::tempdir().unwrap();
        let repo = git2::Repository::init(dir.path()).unwrap();

        for (path, contents) in files {
            write(dir.path(), path, contents);
        }

        {
            let mut index = repo.index().unwrap();
            index
                .add_all(["*"], git2::IndexAddOption::DEFAULT, None)
                .unwrap();
            index.write().unwrap();
            let tree = repo.find_tree(index.write_tree().unwrap()).unwrap();
            let signature =
                git2::Signature::new("test", "test@example.invalid", &git2::Time::new(0, 0))
                    .unwrap();
            repo.commit(Some("HEAD"), &signature, &signature, "fixture", &tree, &[])
                .unwrap();
        }

        (dir, repo)
    }

    fn paths(files: Vec<SourceFile>) -> Vec<String> {
        files.into_iter().map(|f| f.path).collect()
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

        assert_eq!(
            paths(walk_worktree(root, &config).unwrap()),
            vec!["src/main.rs".to_string()]
        );
    }

    #[test]
    fn beholder_does_not_index_its_own_output() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        git2::Repository::init(root).unwrap();

        write(root, "src/main.rs", "fn main() {}\n");
        write(root, "symbols.jsonl", "{}\n");
        write(root, "files.jsonl", "{}\n");

        assert_eq!(
            paths(walk_worktree(root, &Config::default()).unwrap()),
            vec!["src/main.rs".to_string()]
        );
    }

    #[test]
    fn output_is_stably_sorted() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        git2::Repository::init(root).unwrap();

        for name in ["z.rs", "a.rs", "m/n.rs", "m/a.rs"] {
            write(root, name, "fn f() {}\n");
        }

        assert_eq!(
            paths(walk_worktree(root, &Config::default()).unwrap()),
            vec!["a.rs", "m/a.rs", "m/n.rs", "z.rs"]
        );
    }

    #[test]
    fn both_walkers_select_the_same_files() {
        // index reads the working tree and delta reads revisions. If they
        // disagree, the same content yields different symbols depending on
        // which command asked.
        let (dir, repo) = fixture(&[
            (".gitignore", "generated/\n*.tmp\n"),
            ("src/main.rs", "fn main() {}\n"),
            ("src/nested/lib.rs", "fn lib() {}\n"),
            ("docs/notes.md", "# notes\n"),
            ("vendor/dep.rs", "fn dep() {}\n"),
            ("Cargo.lock", "# lock\n"),
            ("symbols.jsonl", "{}\n"),
        ]);

        let config = Config::from_toml(
            r#"
            ignored_extensions = ["lock"]
            ignored_paths = ["vendor/"]
        "#,
        )
        .unwrap();

        let worktree = paths(walk_worktree(dir.path(), &config).unwrap());
        let revision = paths(read_revision(&repo, "HEAD", &config).unwrap());

        assert_eq!(worktree, revision);
        assert_eq!(
            worktree,
            vec!["docs/notes.md", "src/main.rs", "src/nested/lib.rs"]
        );
    }

    #[test]
    fn a_revision_honors_the_gitignore_committed_alongside_it() {
        // A tracked file that .gitignore also matches is invisible to the
        // working-tree walk, so it must be invisible to the revision walk too.
        let (dir, repo) = fixture(&[
            (".gitignore", "generated/\n"),
            ("src/main.rs", "fn main() {}\n"),
            ("generated/schema.rs", "fn generated() {}\n"),
        ]);

        let worktree = paths(walk_worktree(dir.path(), &Config::default()).unwrap());
        let revision = paths(read_revision(&repo, "HEAD", &Config::default()).unwrap());

        assert_eq!(worktree, vec!["src/main.rs".to_string()]);
        assert_eq!(worktree, revision);
    }

    #[test]
    fn neither_walker_treats_a_symlink_as_source() {
        // Git stores a symlink as a blob whose content is the target path. Read
        // as source it is neither, and only one of the two walkers would see it.
        let (dir, repo) = fixture(&[("src/main.rs", "fn main() {}\n")]);

        std::os::unix::fs::symlink("main.rs", dir.path().join("src/alias.rs")).unwrap();

        let mut index = repo.index().unwrap();
        index
            .add_all(["*"], git2::IndexAddOption::DEFAULT, None)
            .unwrap();
        index.write().unwrap();
        let tree_id = index.write_tree().unwrap();
        let signature =
            git2::Signature::new("test", "test@example.invalid", &git2::Time::new(0, 0)).unwrap();
        let parent = repo.head().unwrap().peel_to_commit().unwrap();
        let tree = repo.find_tree(tree_id).unwrap();
        repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            "add a symlink",
            &tree,
            &[&parent],
        )
        .unwrap();
        drop(tree);

        let worktree = paths(walk_worktree(dir.path(), &Config::default()).unwrap());
        let revision = paths(read_revision(&repo, "HEAD", &Config::default()).unwrap());

        assert_eq!(worktree, vec!["src/main.rs".to_string()]);
        assert_eq!(worktree, revision);
    }

    #[test]
    fn a_nested_gitignore_applies_to_its_own_directory() {
        let (dir, repo) = fixture(&[
            ("src/main.rs", "fn main() {}\n"),
            ("src/generated/.gitignore", "*.rs\n"),
            ("src/generated/schema.rs", "fn generated() {}\n"),
            ("other/keep.rs", "fn keep() {}\n"),
        ]);

        let worktree = paths(walk_worktree(dir.path(), &Config::default()).unwrap());
        let revision = paths(read_revision(&repo, "HEAD", &Config::default()).unwrap());

        assert_eq!(worktree, vec!["other/keep.rs", "src/main.rs"]);
        assert_eq!(worktree, revision);
    }
}

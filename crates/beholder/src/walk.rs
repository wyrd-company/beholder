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
/// itself produced, the repository's own `.gitignore` files, and the configured
/// ignores.
///
/// Only rules that live in the repository count. A global excludes file,
/// `.git/info/exclude`, and `.gitignore` files in directories above the
/// repository are all machine-local: they differ between a laptop and CI, and
/// they are invisible to a committed tree. Honoring them would make the file set
/// — and therefore the analysis, and therefore anything stored under a commit —
/// depend on which machine ran it.
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
        || path == "edges.jsonl"
}

/// Walk a working tree, honoring `.gitignore` and the configured ignores.
///
/// Files that are not valid UTF-8 are skipped: no tier can read them.
pub fn walk_worktree(root: &Path, config: &Config) -> Result<Vec<SourceFile>> {
    let rules = PathRules::for_worktree(config);
    let mut files = Vec::new();

    let walker = WalkBuilder::new(root)
        .hidden(false)
        // The repository's own rules, as checked out.
        .git_ignore(true)
        // Everything below is machine-local or outside the repository, and a
        // committed tree cannot see any of it.
        .ignore(false)
        .git_global(false)
        .git_exclude(false)
        .parents(false)
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
    repo: &gix::Repository,
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
    repo: &gix::Repository,
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

/// The commit a revision names.
///
/// A revision may name a commit through a chain of annotated tags, and a tag may
/// point at another tag. Every revision beholder accepts arrives here — delta,
/// report, audit and the stored-analysis lookup all resolve through this — so
/// peeling has to follow the chain to its end rather than requiring the caller
/// to hand over a commit already.
pub fn revision_commit<'r>(repo: &'r gix::Repository, revision: &str) -> Result<gix::Commit<'r>> {
    // gix peels an annotated tag to its commit before it applies `~n` or `^n`,
    // so `sometag~1` resolves the way git resolves it.
    let id = repo
        .rev_parse_single(revision)
        .with_context(|| format!("resolving revision {revision}"))?
        .detach();

    repo.find_object(id)
        .with_context(|| format!("reading the object {revision} names"))?
        .peel_to_commit()
        .with_context(|| format!("{revision} does not name a commit"))
}

/// The tree of a revision.
pub fn revision_tree<'r>(repo: &'r gix::Repository, revision: &str) -> Result<gix::Tree<'r>> {
    revision_commit(repo, revision)?
        .tree()
        .context("reading the commit tree")
}

/// Build one `.gitignore` matcher per directory that holds one, from the
/// `.gitignore` files inside a tree.
///
/// The working-tree walk honors `.gitignore`; a committed tree has to be given
/// the same treatment explicitly or the two walkers disagree.
fn revision_ignores(
    repo: &gix::Repository,
    tree: &gix::Tree<'_>,
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
///
/// The two accepted kinds are exactly filemodes 0o100644 and 0o100755.
fn is_regular_file(mode: gix::object::tree::EntryMode) -> bool {
    matches!(
        mode.kind(),
        gix::object::tree::EntryKind::Blob | gix::object::tree::EntryKind::BlobExecutable
    )
}

/// Visit every blob in a tree with its repo-relative path.
///
/// The traversal yields entries in git's own tree order, which is not path
/// order; every caller sorts what it collects, so the two are kept apart
/// deliberately rather than relying on the walk to arrive sorted.
fn for_each_blob(
    repo: &gix::Repository,
    tree: &gix::Tree<'_>,
    visit: &mut dyn FnMut(&str, &[u8]) -> Result<()>,
) -> Result<()> {
    let entries = tree
        .traverse()
        .breadthfirst
        .files()
        .context("walking the revision tree")?;

    for entry in entries {
        if !is_regular_file(entry.mode) {
            continue;
        }

        let Ok(path) = std::str::from_utf8(&entry.filepath) else {
            continue;
        };

        let blob = repo
            .find_object(entry.oid)
            .with_context(|| format!("reading blob {path}"))?;

        visit(path, &blob.data).with_context(|| format!("reading blob {path}"))?;
    }

    Ok(())
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

    /// Run one git command in `dir`, panicking on failure.
    ///
    /// Fixtures are built by running git rather than by calling a library, so a
    /// passing test is a statement about what git actually writes.
    fn git(dir: &Path, args: &[&str]) {
        let status = std::process::Command::new("git")
            .args(args)
            .current_dir(dir)
            .env("GIT_AUTHOR_NAME", "test")
            .env("GIT_AUTHOR_EMAIL", "test@example.invalid")
            .env("GIT_AUTHOR_DATE", "@0 +0000")
            .env("GIT_COMMITTER_NAME", "test")
            .env("GIT_COMMITTER_EMAIL", "test@example.invalid")
            .env("GIT_COMMITTER_DATE", "@0 +0000")
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .env("GIT_CONFIG_SYSTEM", "/dev/null")
            .output()
            .expect("running git");
        assert!(status.status.success(), "git {args:?} failed");
    }

    fn init(root: &Path) {
        git(root, &["init", "--quiet", "-b", "main", "."]);
    }

    fn commit_all(root: &Path, message: &str) {
        git(root, &["add", "-A"]);
        git(root, &["commit", "--quiet", "--allow-empty", "-m", message]);
    }

    /// The object id a revision resolves to, according to git.
    fn resolve(root: &Path, revision: &str) -> String {
        let output = std::process::Command::new("git")
            .args(["rev-parse", revision])
            .current_dir(root)
            .output()
            .expect("running git rev-parse");
        assert!(output.status.success(), "git rev-parse {revision} failed");
        String::from_utf8(output.stdout).unwrap().trim().to_owned()
    }

    /// A repository with the same files in the worktree and in a commit.
    fn fixture(files: &[(&str, &str)]) -> (tempfile::TempDir, gix::Repository) {
        let dir = tempfile::tempdir().unwrap();
        init(dir.path());

        for (path, contents) in files {
            write(dir.path(), path, contents);
        }

        commit_all(dir.path(), "fixture");

        let repo = gix::open(dir.path()).unwrap();
        (dir, repo)
    }

    fn paths(files: Vec<SourceFile>) -> Vec<String> {
        files.into_iter().map(|f| f.path).collect()
    }

    #[test]
    fn selected_paths_are_shared_boundaries_and_preserve_exclusions() {
        let (dir, repo) = fixture(&[
            ("src/lib/a.rs", "fn a() {}"),
            ("src/library/b.rs", "fn b() {}"),
            ("other.rs", "fn c() {}"),
            ("src/lib/skip.lock", "ignored"),
            (".gitignore", "src/lib/generated/\n"),
            ("src/lib/generated/a.rs", "ignored"),
        ]);
        let mut config = Config::from_toml("ignored_extensions = [\"lock\"]").unwrap();
        config
            .select_paths(&["./src/lib/".into(), "other.rs".into()])
            .unwrap();
        let expected = vec!["other.rs", "src/lib/a.rs"];
        assert_eq!(paths(walk_worktree(dir.path(), &config).unwrap()), expected);
        assert_eq!(
            paths(read_revision(&repo, "HEAD", &config).unwrap()),
            expected
        );
        assert_ne!(config.fingerprint(), Config::default().fingerprint());
        config.select_paths(&["missing".into()]).unwrap();
        assert!(walk_worktree(dir.path(), &config).unwrap().is_empty());
        for invalid in ["", ".", "../src", "/src", "C:/src"] {
            assert!(config.select_paths(&[invalid.into()]).is_err());
        }
    }

    #[test]
    fn honors_gitignore_and_configured_ignores() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        init(root);

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
        init(root);

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
        init(root);

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
    fn machine_local_git_rules_are_ignored_by_both_walkers() {
        // .git/info/exclude, a global excludes file and rules above the
        // repository are all invisible to a committed tree. Honoring them in the
        // working tree alone would make index and delta disagree, and would make
        // the analysis depend on the machine that ran it.
        let (dir, repo) = fixture(&[
            ("src/main.rs", "fn main() {}\n"),
            ("src/excluded_locally.rs", "fn excluded_locally() {}\n"),
        ]);

        std::fs::write(
            dir.path().join(".git/info/exclude"),
            "excluded_locally.rs\n",
        )
        .unwrap();

        // Git itself agrees the file is excluded on this machine.
        assert!(
            std::process::Command::new("git")
                .args([
                    "check-ignore",
                    "--no-index",
                    "-q",
                    "src/excluded_locally.rs"
                ])
                .current_dir(dir.path())
                .status()
                .expect("running git check-ignore")
                .success(),
            "the local exclude is not in force, so this test proves nothing"
        );

        let worktree = paths(walk_worktree(dir.path(), &Config::default()).unwrap());
        let revision = paths(read_revision(&repo, "HEAD", &Config::default()).unwrap());

        assert_eq!(
            worktree,
            vec!["src/excluded_locally.rs", "src/main.rs"],
            "a local exclude must not remove a committed file from analysis"
        );
        assert_eq!(worktree, revision);
    }

    #[test]
    fn a_dot_ignore_file_is_not_a_git_rule() {
        // The ignore crate reads .ignore files by default. Git does not, and a
        // revision walk has no notion of them, so neither walker may.
        let (dir, repo) = fixture(&[
            ("src/main.rs", "fn main() {}\n"),
            (".ignore", "src/hidden.rs\n"),
            ("src/hidden.rs", "fn hidden() {}\n"),
        ]);

        let worktree = paths(walk_worktree(dir.path(), &Config::default()).unwrap());
        let revision = paths(read_revision(&repo, "HEAD", &Config::default()).unwrap());

        assert!(worktree.contains(&"src/hidden.rs".to_string()));
        assert_eq!(worktree, revision);
    }

    #[cfg(unix)]
    #[test]
    fn neither_walker_treats_a_symlink_as_source() {
        // Git stores a symlink as a blob whose content is the target path. Read
        // as source it is neither, and only one of the two walkers would see it.
        let (dir, repo) = fixture(&[("src/main.rs", "fn main() {}\n")]);

        std::os::unix::fs::symlink("main.rs", dir.path().join("src/alias.rs")).unwrap();
        commit_all(dir.path(), "add a symlink");

        let worktree = paths(walk_worktree(dir.path(), &Config::default()).unwrap());
        let revision = paths(read_revision(&repo, "HEAD", &Config::default()).unwrap());

        assert_eq!(worktree, vec!["src/main.rs".to_string()]);
        assert_eq!(worktree, revision);
    }

    #[test]
    fn an_annotated_tag_names_the_commit_it_points_at() {
        // A tag is a revision like any other, and an annotated tag is its own
        // object rather than a pointer to the commit. Every command that takes a
        // revision resolves it here, so a tag that does not peel takes delta,
        // report, audit and the stored-analysis lookup with it.
        let (dir, repo) = fixture(&[("src/main.rs", "fn main() {}\n")]);
        let head = resolve(dir.path(), "HEAD");

        git(dir.path(), &["tag", "-a", "release", "-m", "a release"]);
        git(dir.path(), &["tag", "lightweight"]);

        assert_ne!(
            resolve(dir.path(), "release"),
            head,
            "an annotated tag must be its own object or this test proves nothing"
        );

        for revision in ["release", "lightweight"] {
            assert_eq!(
                revision_commit(&repo, revision).unwrap().id().to_string(),
                head,
                "{revision} should name the commit it points at"
            );
            assert_eq!(
                paths(read_revision(&repo, revision, &Config::default()).unwrap()),
                vec!["src/main.rs".to_string()]
            );
        }
    }

    #[test]
    fn a_tag_pointing_at_a_tag_still_names_a_commit() {
        // Peeling is a chain, not one step.
        let (dir, repo) = fixture(&[("src/main.rs", "fn main() {}\n")]);
        let head = resolve(dir.path(), "HEAD");

        git(dir.path(), &["tag", "-a", "inner", "-m", "inner"]);
        let inner = resolve(dir.path(), "inner");
        git(dir.path(), &["tag", "-a", "outer", "-m", "outer", &inner]);

        assert_ne!(
            resolve(dir.path(), "outer"),
            inner,
            "outer is its own object"
        );
        assert_ne!(inner, head, "inner is its own object");

        assert_eq!(
            revision_commit(&repo, "outer").unwrap().id().to_string(),
            head
        );
        assert_eq!(
            paths(read_revision(&repo, "outer", &Config::default()).unwrap()),
            vec!["src/main.rs".to_string()]
        );
    }

    #[test]
    fn a_revision_navigating_from_an_annotated_tag_names_a_commit() {
        // `~` and `^` count from a commit, so git peels the tag before
        // navigating. A revision spelled this way is ordinary in a release
        // workflow, and it resolved before the migration.
        let (dir, repo) = fixture(&[("src/main.rs", "fn first() {}\n")]);
        let first = resolve(dir.path(), "HEAD");

        write(dir.path(), "src/main.rs", "fn first() {}\nfn second() {}\n");
        commit_all(dir.path(), "second");
        let second = resolve(dir.path(), "HEAD");

        git(dir.path(), &["tag", "-a", "release", "-m", "a release"]);

        for (revision, expected) in [
            ("release", &second),
            ("release~1", &first),
            ("release^", &first),
            ("release^1", &first),
            ("release^{commit}", &second),
        ] {
            assert_eq!(
                &revision_commit(&repo, revision).unwrap().id().to_string(),
                expected,
                "{revision} resolved to the wrong commit"
            );
            assert_eq!(
                resolve(dir.path(), &format!("{revision}^{{commit}}")),
                *expected,
                "the fixture disagrees with git about {revision}"
            );
        }
    }

    #[test]
    fn a_revision_that_names_no_commit_is_refused() {
        // Peeling follows tags to a commit. It must not keep going and turn a
        // commit into its tree, or a revision naming a tree would be accepted as
        // if it were the revision itself.
        let (dir, repo) = fixture(&[("src/main.rs", "fn main() {}\n")]);
        let tree = resolve(dir.path(), "HEAD^{tree}");

        assert!(revision_commit(&repo, &tree).is_err());
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

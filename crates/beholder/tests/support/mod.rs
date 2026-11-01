//! Git fixtures for the integration tests.
//!
//! Fixtures are built by running `git`, not by calling into the library
//! beholder itself uses to read them. A fixture built with the same crate under
//! test can only ever prove that the crate agrees with itself; running real git
//! means a test that passes here is a test against what git actually writes.
//!
//! Identity and timestamps are fixed so a commit is a function of its content
//! and its parents, which several tests depend on directly.

#![allow(dead_code)]

use std::path::{Path, PathBuf};
use std::process::Command;

use gix::ObjectId;

/// Run one git command in `dir` and return its stdout, panicking on failure.
pub fn git(dir: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
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

    assert!(
        output.status.success(),
        "git {args:?} in {} failed: {}",
        dir.display(),
        String::from_utf8_lossy(&output.stderr)
    );

    String::from_utf8(output.stdout).expect("git emitted invalid UTF-8")
}

/// Run one git command, reporting whether it succeeded rather than panicking.
pub fn git_succeeds(dir: &Path, args: &[&str]) -> bool {
    Command::new("git")
        .args(args)
        .current_dir(dir)
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_SYSTEM", "/dev/null")
        .status()
        .expect("running git")
        .success()
}

/// A repository with a working tree, in a directory that cleans itself up.
pub struct Fixture {
    _dir: tempfile::TempDir,
    pub repo: gix::Repository,
}

impl Fixture {
    pub fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();
        init(dir.path());
        let repo = gix::open(dir.path()).unwrap();
        Self { _dir: dir, repo }
    }

    pub fn path(&self) -> &Path {
        self.repo.workdir().unwrap()
    }

    pub fn root(&self) -> PathBuf {
        self.path().to_owned()
    }

    pub fn write(&self, path: &str, contents: &str) {
        let full = self.path().join(path);
        std::fs::create_dir_all(full.parent().unwrap()).unwrap();
        std::fs::write(full, contents).unwrap();
    }

    /// Commit everything git can see, returning the new commit id.
    pub fn commit_all(&self, message: &str) -> ObjectId {
        commit_all(self.path(), message)
    }

    /// Write and commit one file, returning the new commit id.
    pub fn commit(&self, path: &str, contents: &str) -> ObjectId {
        self.write(path, contents);
        self.commit_all(&format!("commit {path}"))
    }

    pub fn add_remote(&self, name: &str, url: &str) {
        git(self.path(), &["remote", "add", name, url]);
    }
}

/// Initialize a repository with a working tree and a fixed branch name.
///
/// The branch is named explicitly so nothing depends on the machine's
/// `init.defaultBranch`.
pub fn init(path: &Path) {
    git(path, &["init", "--quiet", "-b", "main", "."]);
}

/// Initialize a bare repository at `path`.
pub fn init_bare(path: &Path) {
    std::fs::create_dir_all(path).unwrap();
    git(path, &["init", "--quiet", "--bare", "-b", "main", "."]);
}

pub fn commit_all(work_dir: &Path, message: &str) -> ObjectId {
    git(work_dir, &["add", "-A"]);
    git(
        work_dir,
        &["commit", "--quiet", "--allow-empty", "-m", message],
    );
    let id = git(work_dir, &["rev-parse", "HEAD"]);
    ObjectId::from_hex(id.trim().as_bytes()).unwrap()
}

/// Clone `url` into `path` and open the result.
pub fn clone(url: &str, path: &Path) -> gix::Repository {
    let parent = path.parent().unwrap();
    std::fs::create_dir_all(parent).unwrap();
    git(parent, &["clone", "--quiet", url, path.to_str().unwrap()]);
    gix::open(path).unwrap()
}

/// Push whatever the default branch is called, so the fixture works regardless
/// of the local `init.defaultBranch`.
pub fn push_default_branch(work_dir: &Path) {
    let head = git(work_dir, &["symbolic-ref", "HEAD"]);
    let head = head.trim();
    git(
        work_dir,
        &["push", "--quiet", "origin", &format!("{head}:{head}")],
    );
}

/// The tip of a ref in the repository at `path`, if it exists.
pub fn ref_target(repo: &gix::Repository, refname: &str) -> Option<ObjectId> {
    repo.try_find_reference(refname)
        .unwrap()
        .and_then(|reference| reference.target().try_id().map(gix::oid::to_owned))
}

/// Parent `n` of a commit.
pub fn parent(repo: &gix::Repository, commit: ObjectId, n: usize) -> Option<ObjectId> {
    repo.find_commit(commit)
        .unwrap()
        .parent_ids()
        .nth(n)
        .map(gix::Id::detach)
}

/// How many parents a commit has.
pub fn parent_count(repo: &gix::Repository, commit: ObjectId) -> usize {
    repo.find_commit(commit).unwrap().parent_ids().count()
}

/// Push the index the way `scripts/publish-index.sh` does: never forced.
///
/// Returns `true` when the remote took it and `false` when the remote refused it
/// because its ref moved, which is how a writer learns another writer won. Any
/// other failure panics.
pub fn push_index(work_dir: &Path) -> bool {
    let output = Command::new("git")
        .args([
            "push",
            "--porcelain",
            "origin",
            "refs/beholder/index:refs/beholder/index",
        ])
        .current_dir(work_dir)
        .env("LC_ALL", "C")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_SYSTEM", "/dev/null")
        .output()
        .expect("running git push");

    if output.status.success() {
        return true;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout
            .lines()
            .any(|line| line.starts_with('!') && line.contains("\t[rejected]")),
        "git push failed for a reason other than a lost race: {stdout}{}",
        String::from_utf8_lossy(&output.stderr)
    );
    false
}

/// Publish the index the way the action does, returning how many times it gave
/// way to another writer.
///
/// git moves the bytes; beholder reconciles. After a rejection the winner's tip
/// is fetched with git into `FETCH_HEAD`, the local index commits are rebuilt
/// onto it, and the push is tried again.
pub fn publish_index(repo: &gix::Repository) -> usize {
    let work_dir = repo.workdir().expect("a working tree").to_owned();

    for gave_way in 0..8 {
        if push_index(&work_dir) {
            return gave_way;
        }

        git(
            &work_dir,
            &[
                "fetch",
                "--quiet",
                "--no-tags",
                "origin",
                "refs/beholder/index",
            ],
        );
        let theirs = git(&work_dir, &["rev-parse", "FETCH_HEAD"]);
        let theirs = ObjectId::from_hex(theirs.trim().as_bytes()).unwrap();
        beholder::store::Store::open(repo)
            .rebuild_onto(theirs)
            .expect("rebuilding onto the winner's tip");
    }

    panic!("gave up publishing after 8 attempts");
}

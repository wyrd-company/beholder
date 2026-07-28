//! Generated results, stored in a custom git ref.
//!
//! # The ref
//!
//! Beholder writes to `refs/beholder/index`. It is an ordinary git ref outside
//! `refs/heads/*`, so it needs no branch, no worktree, no GitHub-specific API
//! and no artifact service. It travels through ordinary git with
//! [`REFSPEC`]:
//!
//! ```text
//! git push origin +refs/beholder/*:refs/beholder/*
//! git fetch origin +refs/beholder/*:refs/beholder/*
//! ```
//!
//! # The index commit
//!
//! Each index commit carries exactly two parents, in this order:
//!
//! 0. the **source commit** whose analysis it stores
//! 1. the **previous index commit**
//!
//! The first index commit for a repository has only parent 0, because there is
//! no previous index.
//!
//! Traversal follows from that order. Walking parent 1 from the ref tip visits
//! every index in reverse write order; reading parent 0 of any index names the
//! source it describes. Naming the source commit as a parent is also what makes
//! the ref self-contained: pushing it carries the analyzed objects with it, so a
//! fresh clone that fetches the ref can resolve the source it refers to.
//!
//! # Tree layout
//!
//! ```text
//! meta.json                 fingerprint plus the source commit id
//! phase1/<repo path>.json   one phase 1 result per analyzed file
//! phase2.json               the whole-set phase 2 result
//! ```
//!
//! Mirroring the source layout under `phase1/` means `git diff` between two
//! index commits reads as a diff of the analysis.
//!
//! # Derived cache, not source of truth
//!
//! Nothing here is authoritative. A stored result is reused only after
//! [`Meta::validate`] agrees on the source commit, schema version, tool version,
//! analysis configuration, language table and path rules — and, per file, only
//! when the content hash still matches.

use std::collections::BTreeMap;

use anyhow::{bail, Context, Result};
use git2::{Oid, Repository, Signature, Time};
use serde::{Deserialize, Serialize};

use crate::analysis::{is_repo_relative, Analysis, Fingerprint, Phase1Cache, Phase2};
use crate::phase1::FileAnalysis;

/// The ref beholder writes by default.
pub const DEFAULT_REF: &str = "refs/beholder/index";

/// Refspec that moves beholder's results through ordinary fetch and push.
pub const REFSPEC: &str = "+refs/beholder/*:refs/beholder/*";

const META_PATH: &str = "meta.json";
const PHASE2_PATH: &str = "phase2.json";
const PHASE1_DIR: &str = "phase1";
const DEFAULT_ATTEMPTS: usize = 8;
const DEFAULT_CACHE_DEPTH: usize = 64;

/// The header of an index commit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Meta {
    pub fingerprint: Fingerprint,
    /// Hex id of the analyzed source commit. Always equal to parent 0.
    pub source_commit: String,
}

/// Why a stored result could not be reused.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Invalid {
    SourceCommit {
        expected: String,
        found: String,
    },
    SchemaVersion {
        expected: u32,
        found: u32,
    },
    ToolVersion {
        expected: String,
        found: String,
    },
    Config,
    LanguageTable,
    PathRules {
        found: String,
    },
    Path {
        path: String,
    },
    /// The stored analysis covers a different set of files than the source.
    FileSet {
        missing: Vec<String>,
        extra: Vec<String>,
    },
    /// A stored file's content hash does not match the source blob.
    ContentHash {
        path: String,
    },
}

impl std::fmt::Display for Invalid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SourceCommit { expected, found } => {
                write!(f, "stored analysis is of {found}, not {expected}")
            }
            Self::SchemaVersion { expected, found } => {
                write!(f, "stored schema version {found}, expected {expected}")
            }
            Self::ToolVersion { expected, found } => {
                write!(f, "stored by beholder {found}, running {expected}")
            }
            Self::Config => write!(f, "stored under a different analysis configuration"),
            Self::LanguageTable => write!(f, "stored under a different language table"),
            Self::PathRules { found } => write!(f, "stored under path rules {found}"),
            Self::Path { path } => write!(f, "stored path {path} is not repo-relative"),
            Self::FileSet { missing, extra } => write!(
                f,
                "stored analysis covers a different file set: {} missing, {} unexpected",
                missing.len(),
                extra.len()
            ),
            Self::ContentHash { path } => {
                write!(
                    f,
                    "stored result for {path} was computed from other content"
                )
            }
        }
    }
}

impl Meta {
    /// Check every rule that must hold before a stored result may be reused.
    pub fn validate(&self, source_commit: Oid, expected: &Fingerprint) -> Result<(), Invalid> {
        if self.source_commit != source_commit.to_string() {
            return Err(Invalid::SourceCommit {
                expected: source_commit.to_string(),
                found: self.source_commit.clone(),
            });
        }
        self.validate_fingerprint(expected)
    }

    /// The subset of validation that does not depend on which source commit is
    /// being asked about. Used when reusing individual phase 1 results.
    pub fn validate_fingerprint(&self, expected: &Fingerprint) -> Result<(), Invalid> {
        if self.fingerprint.schema_version != expected.schema_version {
            return Err(Invalid::SchemaVersion {
                expected: expected.schema_version,
                found: self.fingerprint.schema_version,
            });
        }
        if self.fingerprint.tool_version != expected.tool_version {
            return Err(Invalid::ToolVersion {
                expected: expected.tool_version.clone(),
                found: self.fingerprint.tool_version.clone(),
            });
        }
        if self.fingerprint.config != expected.config {
            return Err(Invalid::Config);
        }
        if self.fingerprint.language_table != expected.language_table {
            return Err(Invalid::LanguageTable);
        }
        if self.fingerprint.path_rules != expected.path_rules {
            return Err(Invalid::PathRules {
                found: self.fingerprint.path_rules.clone(),
            });
        }
        Ok(())
    }
}

/// Check a stored analysis against the source it claims to describe.
///
/// Metadata agreeing is not enough. The payload itself has to be the analysis of
/// this tree: the same files, each analyzed from the content that is actually
/// there, and every path spelled the one way beholder stores paths. Anything
/// else is a derived cache pretending to be a source of truth.
pub fn validate_payload(
    analysis: &Analysis,
    manifest: &BTreeMap<String, String>,
) -> std::result::Result<(), Invalid> {
    for file in &analysis.files {
        if !is_repo_relative(&file.path) {
            return Err(Invalid::Path {
                path: file.path.clone(),
            });
        }
    }

    let stored: BTreeMap<&str, &str> = analysis
        .files
        .iter()
        .map(|f| (f.path.as_str(), f.content_hash.as_str()))
        .collect();

    let missing: Vec<String> = manifest
        .keys()
        .filter(|path| !stored.contains_key(path.as_str()))
        .cloned()
        .collect();
    let extra: Vec<String> = stored
        .keys()
        .filter(|path| !manifest.contains_key(**path))
        .map(|path| (*path).to_owned())
        .collect();

    if !missing.is_empty() || !extra.is_empty() {
        return Err(Invalid::FileSet { missing, extra });
    }

    for (path, hash) in manifest {
        if stored.get(path.as_str()) != Some(&hash.as_str()) {
            return Err(Invalid::ContentHash { path: path.clone() });
        }
    }

    Ok(())
}

/// One entry in the index history.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexCommit {
    pub id: Oid,
    pub source_commit: Oid,
    pub meta: Meta,
}

/// Reads and writes beholder's custom ref.
pub struct Store<'r> {
    repo: &'r Repository,
    refname: String,
    remote: Option<String>,
    attempts: usize,
}

impl<'r> Store<'r> {
    /// Open the default ref.
    pub fn open(repo: &'r Repository) -> Self {
        Self::with_ref(repo, DEFAULT_REF)
    }

    pub fn with_ref(repo: &'r Repository, refname: &str) -> Self {
        Self {
            repo,
            refname: refname.to_owned(),
            remote: None,
            attempts: DEFAULT_ATTEMPTS,
        }
    }

    /// Fetch this remote's copy of the ref before retrying a conflicted write.
    pub fn with_remote(mut self, remote: impl Into<String>) -> Self {
        self.remote = Some(remote.into());
        self
    }

    pub fn refname(&self) -> &str {
        &self.refname
    }

    /// Current tip of the index ref, if the ref exists.
    pub fn tip(&self) -> Result<Option<Oid>> {
        match self.repo.find_reference(&self.refname) {
            Ok(reference) => Ok(reference.target()),
            Err(e) if e.code() == git2::ErrorCode::NotFound => Ok(None),
            Err(e) => Err(e).context("reading the beholder ref"),
        }
    }

    /// Fetch the ref from a remote using the portable refspec.
    pub fn fetch(&self, remote: &str) -> Result<()> {
        self.repo
            .find_remote(remote)
            .with_context(|| format!("finding remote {remote}"))?
            .fetch(&[REFSPEC], None, None)
            .with_context(|| format!("fetching {REFSPEC} from {remote}"))
    }

    /// Push the ref to a remote.
    ///
    /// The wildcard [`REFSPEC`] is what `git push` on the command line takes;
    /// libgit2 does not expand wildcards on push, so this names the ref
    /// concretely. Both land the same objects under the same name.
    pub fn push(&self, remote: &str) -> Result<()> {
        let refspec = format!("+{0}:{0}", self.refname);
        self.repo
            .find_remote(remote)
            .with_context(|| format!("finding remote {remote}"))?
            .push(&[refspec.as_str()], None)
            .with_context(|| format!("pushing {refspec} to {remote}"))
    }

    /// Index commits from newest to oldest, following parent 1.
    pub fn history(&self, limit: usize) -> Result<Vec<IndexCommit>> {
        let mut out = Vec::new();
        let mut current = self.tip()?;

        while let Some(id) = current {
            if out.len() >= limit {
                break;
            }

            let commit = self
                .repo
                .find_commit(id)
                .context("reading an index commit")?;
            let tree = commit.tree().context("reading an index tree")?;
            let meta = read_json::<Meta>(self.repo, &tree, META_PATH)?
                .with_context(|| format!("index commit {id} has no {META_PATH}"))?;

            let source_commit = commit
                .parent_id(0)
                .with_context(|| format!("index commit {id} has no source parent"))?;

            out.push(IndexCommit {
                id,
                source_commit,
                meta,
            });

            current = commit.parent_id(1).ok();
        }

        Ok(out)
    }

    /// The stored analysis of `source_commit`, if one is valid for reuse.
    ///
    /// Validation runs in two stages because they cost different amounts. The
    /// metadata check is free and rules out most misses; only then is the source
    /// tree read so the payload can be checked against it.
    pub fn find(
        &self,
        source_commit: Oid,
        expected: &Fingerprint,
        config: &crate::Config,
    ) -> Result<Option<Analysis>> {
        let mut manifest = None;

        for entry in self.history(DEFAULT_CACHE_DEPTH)? {
            if entry.source_commit != source_commit {
                continue;
            }
            if entry.meta.validate(source_commit, expected).is_err() {
                continue;
            }

            let analysis = self.read_analysis(entry.id)?;

            let manifest = match &manifest {
                Some(manifest) => manifest,
                None => manifest.insert(crate::walk::revision_manifest(
                    self.repo,
                    &source_commit.to_string(),
                    config,
                )?),
            };

            if validate_payload(&analysis, manifest).is_err() {
                continue;
            }

            return Ok(Some(analysis));
        }

        Ok(None)
    }

    /// A phase 1 cache backed by the stored history.
    pub fn phase1_cache(&self, expected: &Fingerprint) -> Result<StoredPhase1Cache<'r>> {
        let mut trees = Vec::new();

        for entry in self.history(DEFAULT_CACHE_DEPTH)? {
            if entry.meta.validate_fingerprint(expected).is_err() {
                continue;
            }
            let commit = self
                .repo
                .find_commit(entry.id)
                .context("reading an index commit")?;
            trees.push(commit.tree_id());
        }

        Ok(StoredPhase1Cache {
            repo: self.repo,
            trees,
        })
    }

    /// Rebuild a full analysis from an index commit.
    pub fn read_analysis(&self, index_commit: Oid) -> Result<Analysis> {
        let commit = self.repo.find_commit(index_commit)?;
        let tree = commit.tree()?;

        let meta = read_json::<Meta>(self.repo, &tree, META_PATH)?
            .with_context(|| format!("index commit {index_commit} has no {META_PATH}"))?;
        let phase2 = read_json::<Phase2>(self.repo, &tree, PHASE2_PATH)?.unwrap_or_default();

        let mut files = Vec::new();
        if let Ok(entry) = tree.get_path(std::path::Path::new(PHASE1_DIR)) {
            let subtree = self.repo.find_tree(entry.id())?;
            let mut error = None;

            subtree.walk(git2::TreeWalkMode::PreOrder, |_, entry| {
                if entry.kind() != Some(git2::ObjectType::Blob) {
                    return git2::TreeWalkResult::Ok;
                }
                match self
                    .repo
                    .find_blob(entry.id())
                    .map_err(anyhow::Error::new)
                    .and_then(|b| Ok(serde_json::from_slice::<FileAnalysis>(b.content())?))
                {
                    Ok(analysis) => {
                        files.push(analysis);
                        git2::TreeWalkResult::Ok
                    }
                    Err(e) => {
                        error = Some(e);
                        git2::TreeWalkResult::Abort
                    }
                }
            })?;

            if let Some(e) = error {
                return Err(e).context("reading stored phase 1 results");
            }
        }

        files.sort_by(|a, b| a.path.cmp(&b.path));

        Ok(Analysis {
            fingerprint: meta.fingerprint,
            files,
            phase2,
        })
    }

    /// Append an index commit for `source_commit`, retrying on conflict.
    pub fn write(&self, source_commit: Oid, analysis: &Analysis) -> Result<Oid> {
        self.write_with_probe(source_commit, analysis, &mut |_| {})
    }

    /// [`Store::write`] with a hook that runs after the tip is read and before
    /// the ref is updated. Exists so a concurrent writer can be simulated.
    pub fn write_with_probe(
        &self,
        source_commit: Oid,
        analysis: &Analysis,
        probe: &mut dyn FnMut(usize),
    ) -> Result<Oid> {
        let tree_id = self.build_tree(source_commit, analysis)?;

        for attempt in 0..self.attempts {
            let expected_tip = self.tip()?;
            let index_commit = self.build_commit(source_commit, tree_id, expected_tip)?;

            probe(attempt);

            match self.update_ref(expected_tip, index_commit) {
                Ok(()) => return Ok(index_commit),
                Err(err) if is_conflict(&err) => {
                    // Someone else advanced the ref. Their results stay exactly
                    // where they are; this write is rebuilt on top of the tip
                    // they left behind.
                    if let Some(remote) = &self.remote {
                        self.fetch(remote)?;
                    }
                    continue;
                }
                Err(err) => {
                    return Err(anyhow::Error::new(err).context("updating the beholder ref"))
                }
            }
        }

        bail!(
            "gave up updating {} after {} attempts against concurrent writers",
            self.refname,
            self.attempts
        )
    }

    fn update_ref(
        &self,
        expected_tip: Option<Oid>,
        new_tip: Oid,
    ) -> std::result::Result<(), git2::Error> {
        let message = format!("beholder index {new_tip}");

        match expected_tip {
            // Compare-and-swap against the tip this commit was built on.
            Some(expected) => self
                .repo
                .reference_matching(&self.refname, new_tip, true, expected, &message)
                .map(|_| ()),
            // Create only. Fails if another writer created the ref first.
            None => self
                .repo
                .reference(&self.refname, new_tip, false, &message)
                .map(|_| ()),
        }
    }

    fn build_commit(
        &self,
        source_commit: Oid,
        tree_id: Oid,
        previous_index: Option<Oid>,
    ) -> Result<Oid> {
        let tree = self.repo.find_tree(tree_id)?;
        let source = self
            .repo
            .find_commit(source_commit)
            .with_context(|| format!("source commit {source_commit} is not in this repository"))?;

        // Parent 0 is the source. Parent 1, when it exists, is the previous
        // index. The first index for a repository has only parent 0.
        let mut parents = vec![source];
        if let Some(previous) = previous_index {
            parents.push(self.repo.find_commit(previous)?);
        }

        let parent_refs: Vec<&git2::Commit<'_>> = parents.iter().collect();

        // A fixed signature keeps an index commit a function of its content and
        // its parents. Two machines analyzing the same source produce the same
        // index commit id.
        let signature = Signature::new("beholder", "beholder@wyrd.invalid", &Time::new(0, 0))
            .context("building the index signature")?;

        Ok(self.repo.commit(
            None,
            &signature,
            &signature,
            &format!("beholder index for {source_commit}\n"),
            &tree,
            &parent_refs,
        )?)
    }

    fn build_tree(&self, source_commit: Oid, analysis: &Analysis) -> Result<Oid> {
        let mut entries: BTreeMap<String, Oid> = BTreeMap::new();

        let meta = Meta {
            fingerprint: analysis.fingerprint.clone(),
            source_commit: source_commit.to_string(),
        };
        entries.insert(META_PATH.to_owned(), self.blob_json(&meta)?);
        entries.insert(PHASE2_PATH.to_owned(), self.blob_json(&analysis.phase2)?);

        for file in &analysis.files {
            if !is_repo_relative(&file.path) {
                bail!("refusing to store non repo-relative path {}", file.path);
            }
            entries.insert(
                format!("{PHASE1_DIR}/{}.json", file.path),
                self.blob_json(file)?,
            );
        }

        build_tree(self.repo, &entries, "")
    }

    fn blob_json<T: Serialize>(&self, value: &T) -> Result<Oid> {
        let mut bytes = serde_json::to_vec_pretty(value)?;
        bytes.push(b'\n');
        Ok(self.repo.blob(&bytes)?)
    }
}

/// Phase 1 results read back out of the index history.
pub struct StoredPhase1Cache<'r> {
    repo: &'r Repository,
    trees: Vec<Oid>,
}

impl Phase1Cache for StoredPhase1Cache<'_> {
    fn get(&self, path: &str, content_hash: &str) -> Option<FileAnalysis> {
        if !is_repo_relative(path) {
            return None;
        }

        let stored_path = format!("{PHASE1_DIR}/{path}.json");

        for tree_id in &self.trees {
            let Ok(tree) = self.repo.find_tree(*tree_id) else {
                continue;
            };
            let Ok(Some(stored)) = read_json::<FileAnalysis>(self.repo, &tree, &stored_path) else {
                continue;
            };
            // Phase 1 is a pure function of content, so a stored result is only
            // this file's result when the content it was computed from matches.
            if stored.content_hash == content_hash && stored.path == path {
                return Some(stored);
            }
        }

        None
    }
}

// ---------------------------------------------------------------------------
// git plumbing
// ---------------------------------------------------------------------------

fn is_conflict(err: &git2::Error) -> bool {
    matches!(
        err.code(),
        git2::ErrorCode::Modified | git2::ErrorCode::Exists | git2::ErrorCode::NotFastForward
    )
}

fn read_json<T: for<'de> Deserialize<'de>>(
    repo: &Repository,
    tree: &git2::Tree<'_>,
    path: &str,
) -> Result<Option<T>> {
    let Ok(entry) = tree.get_path(std::path::Path::new(path)) else {
        return Ok(None);
    };
    let blob = repo.find_blob(entry.id())?;
    Ok(Some(
        serde_json::from_slice(blob.content()).with_context(|| format!("parsing stored {path}"))?,
    ))
}

/// Build a git tree from a flat map of slash-separated paths to blob ids.
fn build_tree(repo: &Repository, entries: &BTreeMap<String, Oid>, prefix: &str) -> Result<Oid> {
    let mut builder = repo.treebuilder(None)?;
    let mut subdirectories: BTreeMap<String, BTreeMap<String, Oid>> = BTreeMap::new();

    for (path, oid) in entries {
        match path.split_once('/') {
            None => builder.insert(path.as_str(), *oid, git2::FileMode::Blob.into())?,
            Some((head, rest)) => {
                subdirectories
                    .entry(head.to_owned())
                    .or_default()
                    .insert(rest.to_owned(), *oid);
                continue;
            }
        };
    }

    for (name, children) in &subdirectories {
        let child_prefix = format!("{prefix}{name}/");
        let subtree = build_tree(repo, children, &child_prefix)?;
        builder.insert(name.as_str(), subtree, git2::FileMode::Tree.into())?;
    }

    Ok(builder.write()?)
}

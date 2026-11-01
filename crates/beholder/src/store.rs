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

use std::collections::{BTreeMap, HashSet};

use anyhow::{bail, Context, Result};
use gix::refs::transaction::{Change, LogChange, PreviousValue, RefEdit, RefLog};
use gix::ObjectId as Oid;
use gix::Repository;
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
        // Compared as the stored text, so only the exact lowercase hex git
        // writes is accepted.
        let expected_commit = source_commit.to_string();
        if self.source_commit != expected_commit {
            return Err(Invalid::SourceCommit {
                expected: expected_commit,
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

/// Where a fetch or a rebuild left the local ref.
///
/// `rebuilt` is how many local index commits the remote tip did not have. They
/// are on top of it now, and a push of `tip` is a fast-forward.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reconciled {
    /// The remote's tip, or `None` when the remote has no index yet.
    pub remote: Option<Oid>,
    /// The local tip afterwards.
    pub tip: Option<Oid>,
    /// Local index commits rebuilt on top of `remote`.
    pub rebuilt: usize,
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
        let Some(reference) = self
            .repo
            .try_find_reference(self.refname.as_str())
            .context("reading the beholder ref")?
        else {
            return Ok(None);
        };

        Ok(reference.target().try_id().map(gix::oid::to_owned))
    }

    /// Fetch the remote's index and keep every local index commit on top of it.
    ///
    /// Only this ref is asked for, with no local destination, so the fetch
    /// itself moves no local ref and brings down no branches. The local ref then
    /// moves once, through [`Store::rebuild_onto`]: index commits the remote
    /// does not have are rebuilt on its tip, so a fetch never drops work that
    /// was only written here.
    ///
    /// The transport is gix's own. It reads the repository's git configuration,
    /// but not URL-scoped `http.<url>.*` keys, so a credential stored as
    /// `http.<url>.extraHeader` (the way `actions/checkout` stores its token) is
    /// not sent. The action therefore fetches with git and hands the tip to
    /// [`Store::rebuild_onto`]; this method is for every other caller.
    pub fn fetch(&self, remote: &str) -> Result<Reconciled> {
        match self.fetch_remote_tip(remote)? {
            Some(theirs) => self.rebuild_onto(theirs),
            None => Ok(Reconciled {
                remote: None,
                tip: self.tip()?,
                rebuilt: 0,
            }),
        }
    }

    /// The remote's tip of this ref, with its objects now in the local
    /// database, or `None` when the remote has no index yet.
    fn fetch_remote_tip(&self, remote: &str) -> Result<Option<Oid>> {
        transport::ensure_tls_provider();

        // A source without a destination: the objects arrive and no local ref
        // moves, so the only ref update is the compare-and-swap that follows.
        // The remote's configured refspecs are replaced, not added to, and tags
        // are not followed, so asking for the index does not also bring down
        // every branch and every tag its source commits carry.
        let refspec = self.refname.as_str();
        let mut handle = self
            .repo
            .find_fetch_remote(Some(remote.into()))
            .with_context(|| format!("finding remote {remote}"))?
            .with_fetch_tags(gix::remote::fetch::Tags::None);
        handle
            .replace_refspecs(Some(refspec), gix::remote::Direction::Fetch)
            .with_context(|| format!("applying refspec {refspec}"))?;

        let prepared = handle
            .connect(gix::remote::Direction::Fetch)
            .with_context(|| format!("connecting to {remote}"))?
            .prepare_fetch(gix::progress::Discard, Default::default())
            .with_context(|| format!("listing the refs on {remote}"))?;

        let Some(theirs) = prepared
            .ref_map()
            .mappings
            .iter()
            .filter(|mapping| mapping.remote.as_name() == Some(refspec.into()))
            .find_map(|mapping| mapping.remote.as_id())
            .map(gix::oid::to_owned)
        else {
            return Ok(None);
        };

        prepared
            .receive(gix::progress::Discard, &Default::default())
            .with_context(|| format!("fetching {refspec} from {remote}"))?;

        Ok(Some(theirs))
    }

    /// Rebuild the local index commits that `theirs` does not have onto it.
    ///
    /// `theirs` is an index tip whose objects are already in this repository,
    /// usually one just fetched from a remote that refused a push. Whatever is
    /// only here is rebuilt on top of it: same source commits, same trees, new
    /// index parents. Nothing in `theirs` is rewritten or dropped. When there is
    /// nothing local, the ref moves to `theirs`; when the local tip already
    /// contains `theirs`, every rebuilt commit is the commit it replaces and the
    /// ref does not move.
    ///
    /// The ref moves by compare-and-swap against the tip this started from, so
    /// a local writer that lands in between is rebuilt as well, not lost.
    pub fn rebuild_onto(&self, theirs: Oid) -> Result<Reconciled> {
        // The incoming chain is checked whole before the ref can move onto it.
        let shared: HashSet<Oid> = self.checked_index_chain(theirs)?.into_iter().collect();

        for _ in 0..self.attempts {
            let mine = self.tip()?;
            let mut local_only = Vec::new();

            for id in self.index_ancestry(mine)? {
                if shared.contains(&id) {
                    break;
                }
                local_only.push(id);
            }

            let rebuilt = local_only.len();
            let mut base = theirs;

            // Oldest first, so the rebuilt chain keeps its original write order.
            for id in local_only.into_iter().rev() {
                let commit = self.repo.find_commit(id)?;
                let source = parent(self.repo, id, 0)?
                    .with_context(|| format!("index commit {id} has no source parent"))?;
                let tree = commit
                    .tree_id()
                    .with_context(|| format!("index commit {id} has no tree"))?
                    .detach();
                base = self.build_commit(source, tree, Some(base))?;
            }

            let reconciled = Reconciled {
                remote: Some(theirs),
                tip: Some(base),
                rebuilt,
            };

            if mine == Some(base) {
                return Ok(reconciled);
            }

            match self.update_ref(mine, base) {
                Ok(()) => return Ok(reconciled),
                Err(err) if is_conflict(&err) => continue,
                Err(err) => {
                    return Err(anyhow::Error::new(err)
                        .context(format!("moving {} onto {theirs}", self.refname)))
                }
            }
        }

        bail!(
            "gave up rebuilding {} onto {theirs} after {} attempts against concurrent writers",
            self.refname,
            self.attempts
        )
    }

    /// Every commit of the index chain at `tip`, newest first, each checked
    /// against the index commit convention.
    ///
    /// A tip from elsewhere is not trusted. Parent 1 of an index commit is the
    /// previous index, but parent 1 of an ordinary merge is not, and a copied
    /// index tree on a commit without a source parent would read as an index
    /// until something asked for its source. Every commit must have one or two
    /// parents, a readable `meta.json`, and a source parent that is the commit
    /// its `meta.json` names.
    fn checked_index_chain(&self, tip: Oid) -> Result<Vec<Oid>> {
        let chain = self.index_ancestry(Some(tip))?;

        for &id in &chain {
            let not_an_index = |why: String| anyhow::anyhow!("{id} is not an index commit: {why}");

            let commit = self
                .repo
                .find_commit(id)
                .with_context(|| format!("{id} is not a commit in this repository"))?;
            let parents = commit.parent_ids().count();
            if !(1..=2).contains(&parents) {
                return Err(not_an_index(format!("it has {parents} parents")));
            }

            let tree = commit.tree().context("reading an index tree")?;
            let meta = read_json::<Meta>(&tree, META_PATH)
                .map_err(|err| not_an_index(format!("{err:#}")))?
                .ok_or_else(|| not_an_index(format!("it has no {META_PATH}")))?;

            let source = parent(self.repo, id, 0)?
                .expect("a commit with a parent")
                .to_string();
            if meta.source_commit != source {
                return Err(not_an_index(format!(
                    "its {META_PATH} names {} but its source parent is {source}",
                    meta.source_commit
                )));
            }
        }

        Ok(chain)
    }

    /// Every index commit reachable from a tip, newest first, following parent 1.
    ///
    /// Deliberately unbounded, unlike [`Store::history`]. Cache lookup may stop
    /// early because stopping early only costs a recomputation. Reconciliation
    /// may not: a cutoff either orphans the commits beyond it, or fails to
    /// recognize commits both sides already share and replays them as if they
    /// were local. Both silently corrupt the history the ref is supposed to
    /// carry, so this walks to the root.
    fn index_ancestry(&self, from: Option<Oid>) -> Result<Vec<Oid>> {
        let mut out = Vec::new();
        let mut current = from;

        while let Some(id) = current {
            out.push(id);
            current = parent(self.repo, id, 1)?;
        }

        Ok(out)
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
            let meta = read_json::<Meta>(&tree, META_PATH)?
                .with_context(|| format!("index commit {id} has no {META_PATH}"))?;

            let source_commit = parent(self.repo, id, 0)?
                .with_context(|| format!("index commit {id} has no source parent"))?;

            out.push(IndexCommit {
                id,
                source_commit,
                meta,
            });

            current = parent(self.repo, id, 1)?;
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
            trees.push(
                commit
                    .tree_id()
                    .context("reading an index tree id")?
                    .detach(),
            );
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

        let meta = read_json::<Meta>(&tree, META_PATH)?
            .with_context(|| format!("index commit {index_commit} has no {META_PATH}"))?;
        let phase2 = read_json::<Phase2>(&tree, PHASE2_PATH)?.unwrap_or_default();

        let mut files = Vec::new();
        if let Some(entry) = tree
            .lookup_entry_by_path(std::path::Path::new(PHASE1_DIR))
            .context("looking for stored phase 1 results")?
        {
            let subtree = self
                .repo
                .find_tree(entry.object_id())
                .context("reading the phase 1 subtree")?;

            for entry in subtree
                .traverse()
                .breadthfirst
                .files()
                .context("walking stored phase 1 results")?
            {
                if entry.mode.is_tree() {
                    continue;
                }
                let blob = self
                    .repo
                    .find_object(entry.oid)
                    .context("reading a stored phase 1 result")?;
                files.push(
                    serde_json::from_slice::<FileAnalysis>(&blob.data)
                        .context("parsing a stored phase 1 result")?,
                );
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
    ) -> std::result::Result<(), gix::Error> {
        let message = format!("beholder index {new_tip}");

        let expected = match expected_tip {
            // Compare-and-swap against the tip this commit was built on.
            Some(expected) => PreviousValue::MustExistAndMatch(gix::refs::Target::Object(expected)),
            // Create only. Fails if another writer created the ref first.
            None => PreviousValue::MustNotExist,
        };

        write_ref(self.repo, &self.refname, expected, new_tip, &message)
    }

    fn build_commit(
        &self,
        source_commit: Oid,
        tree_id: Oid,
        previous_index: Option<Oid>,
    ) -> Result<Oid> {
        self.repo
            .find_commit(source_commit)
            .with_context(|| format!("source commit {source_commit} is not in this repository"))?;

        // Parent 0 is the source. Parent 1, when it exists, is the previous
        // index. The first index for a repository has only parent 0.
        let mut parents = vec![source_commit];
        if let Some(previous) = previous_index {
            self.repo
                .find_commit(previous)
                .with_context(|| format!("previous index {previous} is not in this repository"))?;
            parents.push(previous);
        }

        // A fixed signature keeps an index commit a function of its content and
        // its parents. Two machines analyzing the same source produce the same
        // index commit id.
        let signature = gix::actor::Signature {
            name: "beholder".into(),
            email: "beholder@wyrd.invalid".into(),
            time: gix::date::Time::new(0, 0),
        };

        // The commit is assembled field by field rather than through a builder.
        // Its byte layout is the identity of every index commit ever written, so
        // an index written before this migration and one written after have to
        // serialize the same: no encoding header, no extra headers, the message
        // exactly as given.
        let commit = gix::objs::Commit {
            tree: tree_id,
            parents: parents.into_iter().collect(),
            author: signature.clone(),
            committer: signature,
            encoding: None,
            message: format!("beholder index for {source_commit}\n").into(),
            extra_headers: Vec::new(),
        };

        Ok(self
            .repo
            .write_object(&commit)
            .context("writing the index commit")?
            .detach())
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
        Ok(self.repo.write_blob(&bytes)?.detach())
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
            let Ok(Some(stored)) = read_json::<FileAnalysis>(&tree, &stored_path) else {
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

/// Did another writer get there first?
///
/// The three shapes are the same three the compare-and-swap can lose to: the ref
/// moved, the ref appeared when it was supposed to be absent, or it vanished
/// when it was supposed to be present. Anything else is a real failure and must
/// not be retried.
fn is_conflict(err: &gix::Error) -> bool {
    use gix::refs::file::transaction::prepare::{MustNotExist, ReferenceOutOfDate};

    err.downcast_any_ref::<ReferenceOutOfDate>().is_some()
        || err.downcast_any_ref::<MustNotExist>().is_some()
        // A ref that was required to exist and is gone.
        || err.is_not_found()
}

/// Set a ref, subject to what its previous value is required to be.
fn write_ref(
    repo: &Repository,
    refname: &str,
    expected: PreviousValue,
    new_tip: Oid,
    message: &str,
) -> std::result::Result<(), gix::Error> {
    let name = refname
        .try_into()
        .map_err(|err: gix::refs::name::Error| gix::Error::from_error(err))?;
    repo.edit_reference(RefEdit {
        change: Change::Update {
            log: LogChange {
                mode: RefLog::AndReference,
                force_create_reflog: false,
                message: message.into(),
            },
            expected,
            new: gix::refs::Target::Object(new_tip),
        },
        name,
        deref: false,
    })
    .map(|_| ())
}

/// Parent `n` of a commit, if it has one.
fn parent(repo: &Repository, commit: Oid, n: usize) -> Result<Option<Oid>> {
    Ok(repo
        .find_commit(commit)
        .with_context(|| format!("reading commit {commit}"))?
        .parent_ids()
        .nth(n)
        .map(gix::Id::detach))
}

fn read_json<T: for<'de> Deserialize<'de>>(tree: &gix::Tree<'_>, path: &str) -> Result<Option<T>> {
    let Some(entry) = tree
        .lookup_entry_by_path(std::path::Path::new(path))
        .with_context(|| format!("looking for stored {path}"))?
    else {
        return Ok(None);
    };
    let blob = entry
        .object()
        .with_context(|| format!("reading stored {path}"))?;
    Ok(Some(
        serde_json::from_slice(&blob.data).with_context(|| format!("parsing stored {path}"))?,
    ))
}

/// Build a git tree from a flat map of slash-separated paths to blob ids.
///
/// Entries are sorted the way git sorts them — by name, with a directory
/// compared as if its name ended in `/` — because that ordering is part of the
/// tree's bytes and therefore part of every index commit id.
fn build_tree(repo: &Repository, entries: &BTreeMap<String, Oid>, prefix: &str) -> Result<Oid> {
    use gix::objs::tree::{Entry, EntryKind};

    let mut written: Vec<Entry> = Vec::new();
    let mut subdirectories: BTreeMap<String, BTreeMap<String, Oid>> = BTreeMap::new();

    for (path, oid) in entries {
        match path.split_once('/') {
            None => written.push(Entry {
                mode: EntryKind::Blob.into(),
                filename: path.as_str().into(),
                oid: *oid,
            }),
            Some((head, rest)) => {
                subdirectories
                    .entry(head.to_owned())
                    .or_default()
                    .insert(rest.to_owned(), *oid);
            }
        }
    }

    for (name, children) in &subdirectories {
        let child_prefix = format!("{prefix}{name}/");
        let subtree = build_tree(repo, children, &child_prefix)?;
        written.push(Entry {
            mode: EntryKind::Tree.into(),
            filename: name.as_str().into(),
            oid: subtree,
        });
    }

    written.sort();

    Ok(repo
        .write_object(&gix::objs::Tree { entries: written })
        .with_context(|| format!("writing the tree at {prefix:?}"))?
        .detach())
}

// ---------------------------------------------------------------------------
// transports
// ---------------------------------------------------------------------------

/// How the ref reaches a remote.
///
/// Beholder fetches with gix, over ssh, https and local paths. It does not push:
/// gix has no send-pack, and beholder starts no git process, so pushing is the
/// caller's ordinary `git push` without `+`. What makes the shared ref safe — the
/// compare-and-swap and [`Store::rebuild_onto`] — stays in [`Store`].
mod transport {
    /// Install the TLS backend gix's https transport needs.
    ///
    /// The `reqwest/rustls-no-provider` feature deliberately leaves the crypto
    /// provider unset so the choice is not made for us; see the `gix` entry in
    /// the workspace manifest for why the choice matters. Installing it can only
    /// fail because it already happened, which is not an error.
    pub fn ensure_tls_provider() {
        static ONCE: std::sync::Once = std::sync::Once::new();
        ONCE.call_once(|| {
            let _ = rustls::crypto::ring::default_provider().install_default();
        });
    }
}

pub use transport::ensure_tls_provider;

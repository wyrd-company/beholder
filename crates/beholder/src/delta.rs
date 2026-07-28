//! Delta mode — any two revisions in, changed symbols out.
//!
//! Matching is by symbol identity, never by line number. A symbol that only
//! moved down the file is not a change, which is what makes a reformat quiet.
//!
//! Identity cannot survive a rename or a move between files by construction: the
//! path and the name are what identity is made of. Those are real changes, and
//! delta names them as moves and renames rather than emitting a delete and an
//! unrelated add. That distinction is the difference between a report a reviewer
//! trusts and one they learn to skim.

use std::collections::HashMap;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::analysis::{self, Analysis, Fingerprint};
use crate::{Config, Symbol};

/// What happened to a symbol between two revisions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChangeKind {
    Added,
    Removed,
    /// Same identity, different structure or different complexity.
    Modified,
    /// Same code, different file.
    Moved,
    /// Same body, different name.
    Renamed,
}

/// The metrics a report compares.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Metrics {
    pub id: String,
    pub path: String,
    pub qualified_path: String,
    pub start_line: usize,
    pub end_line: usize,
    pub cognitive_complexity: u32,
}

impl From<&Symbol> for Metrics {
    fn from(symbol: &Symbol) -> Self {
        Self {
            id: symbol.id.clone(),
            path: symbol.path.clone(),
            qualified_path: symbol.qualified_path.clone(),
            start_line: symbol.start_line,
            end_line: symbol.end_line,
            cognitive_complexity: symbol.cognitive_complexity,
        }
    }
}

/// One changed symbol, with before and after metrics.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SymbolChange {
    pub change: ChangeKind,
    pub language: String,
    pub kind: String,
    pub before: Option<Metrics>,
    pub after: Option<Metrics>,
    /// After minus before, when both sides exist.
    pub complexity_delta: Option<i64>,
}

impl SymbolChange {
    /// Every repo-relative path this change touches.
    pub fn paths(&self) -> Vec<&str> {
        let mut paths: Vec<&str> = self
            .before
            .iter()
            .map(|m| m.path.as_str())
            .chain(self.after.iter().map(|m| m.path.as_str()))
            .collect();
        paths.sort_unstable();
        paths.dedup();
        paths
    }
}

/// The result of comparing two revisions.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Delta {
    /// Changed symbols only. A symbol that is byte-for-byte the same work in
    /// the same place never appears here.
    pub changes: Vec<SymbolChange>,
}

/// Compare two analyses by symbol identity.
pub fn compare(before: &Analysis, after: &Analysis) -> Delta {
    let before_by_id: HashMap<&str, &Symbol> =
        before.symbols().map(|s| (s.id.as_str(), s)).collect();
    let after_by_id: HashMap<&str, &Symbol> = after.symbols().map(|s| (s.id.as_str(), s)).collect();

    let mut changes = Vec::new();
    let mut removed: Vec<&Symbol> = Vec::new();
    let mut added: Vec<&Symbol> = Vec::new();

    for symbol in before.symbols() {
        match after_by_id.get(symbol.id.as_str()) {
            None => removed.push(symbol),
            Some(current) => {
                // Line ranges shift under a reformat. Structure and score do not.
                let structural = symbol.content_fingerprint != current.content_fingerprint;
                let scored = symbol.cognitive_complexity != current.cognitive_complexity;

                if structural || scored {
                    changes.push(change(ChangeKind::Modified, Some(symbol), Some(current)));
                }
            }
        }
    }

    for symbol in after.symbols() {
        if !before_by_id.contains_key(symbol.id.as_str()) {
            added.push(symbol);
        }
    }

    changes.extend(pair_up(&mut removed, &mut added));
    changes.extend(
        removed
            .into_iter()
            .map(|s| change(ChangeKind::Removed, Some(s), None)),
    );
    changes.extend(
        added
            .into_iter()
            .map(|s| change(ChangeKind::Added, None, Some(s))),
    );

    sort(&mut changes);
    Delta { changes }
}

/// Recognize moves and renames among the otherwise unmatched symbols.
///
/// Identical code in a new file is a move. An identical body under a new name in
/// the same file is a rename. Matching runs strongest-signal first and consumes
/// both sides, so no symbol is reported twice.
fn pair_up(removed: &mut Vec<&Symbol>, added: &mut Vec<&Symbol>) -> Vec<SymbolChange> {
    let mut changes = Vec::new();

    for (change_kind, key) in [
        (ChangeKind::Moved, Key::Content),
        (ChangeKind::Renamed, Key::Body),
    ] {
        let mut unmatched_removed = Vec::new();

        for old in removed.drain(..) {
            let candidate = added
                .iter()
                .position(|new| key.matches(old, new) && old.kind == new.kind);

            match candidate {
                Some(index) => {
                    let new = added.remove(index);
                    changes.push(change(change_kind, Some(old), Some(new)));
                }
                None => unmatched_removed.push(old),
            }
        }

        *removed = unmatched_removed;
    }

    changes
}

#[derive(Clone, Copy)]
enum Key {
    Content,
    Body,
}

impl Key {
    fn matches(self, old: &Symbol, new: &Symbol) -> bool {
        match self {
            // Identical code, so only its location can have changed.
            Self::Content => {
                old.content_fingerprint == new.content_fingerprint && old.path != new.path
            }
            // Identical body under a different name, in the same file.
            Self::Body => old.body_fingerprint == new.body_fingerprint && old.path == new.path,
        }
    }
}

fn change(kind: ChangeKind, before: Option<&Symbol>, after: Option<&Symbol>) -> SymbolChange {
    let complexity_delta = match (before, after) {
        (Some(b), Some(a)) => {
            Some(i64::from(a.cognitive_complexity) - i64::from(b.cognitive_complexity))
        }
        _ => None,
    };

    let representative = after.or(before).expect("a change has at least one side");

    SymbolChange {
        change: kind,
        language: representative.language.clone(),
        kind: representative.kind.clone(),
        before: before.map(Metrics::from),
        after: after.map(Metrics::from),
        complexity_delta,
    }
}

/// Deterministic output order.
fn sort(changes: &mut [SymbolChange]) {
    changes.sort_by(|a, b| {
        let key = |c: &SymbolChange| {
            let m = c.after.as_ref().or(c.before.as_ref()).expect("a side");
            (m.path.clone(), m.start_line, m.id.clone())
        };
        key(a).cmp(&key(b))
    });
}

// ---------------------------------------------------------------------------
// Revisions
// ---------------------------------------------------------------------------

/// Analyze one revision of a repository, reusing stored results where valid.
pub fn analyze_revision(
    repo: &git2::Repository,
    revision: &str,
    config: &Config,
    store: Option<&crate::Store<'_>>,
) -> Result<Analysis> {
    let fingerprint = Fingerprint::new(config);
    let commit = repo
        .revparse_single(revision)
        .with_context(|| format!("resolving {revision}"))?
        .peel_to_commit()
        .with_context(|| format!("{revision} is not a commit"))?;

    if let Some(store) = store {
        if let Some(stored) = store.find(commit.id(), &fingerprint)? {
            return Ok(stored);
        }
    }

    let files = crate::walk::read_revision(repo, revision, config)?;

    let analysis = match store {
        // Missing results are generated, never faked, and the cache only
        // supplies files whose content is unchanged.
        Some(store) => {
            let cache = store.phase1_cache(&fingerprint)?;
            analysis::run_with_cache(&files, config, &cache).0
        }
        None => analysis::run(&files, config),
    };

    Ok(analysis)
}

/// Compare two revisions, appending newly generated results to the index.
pub fn compare_revisions(
    repo: &git2::Repository,
    before: &str,
    after: &str,
    config: &Config,
    store: Option<&crate::Store<'_>>,
) -> Result<Delta> {
    let before_analysis = analyze_revision(repo, before, config, store)?;
    let after_analysis = analyze_revision(repo, after, config, store)?;

    if let Some(store) = store {
        for (revision, analysis) in [(before, &before_analysis), (after, &after_analysis)] {
            let commit = repo.revparse_single(revision)?.peel_to_commit()?;
            if store
                .find(commit.id(), &Fingerprint::new(config))?
                .is_none()
            {
                store.write(commit.id(), analysis)?;
            }
        }
    }

    Ok(compare(&before_analysis, &after_analysis))
}

// ---------------------------------------------------------------------------
// Identity audit
// ---------------------------------------------------------------------------

/// A change that says more about identity than about the code.
///
/// This is what the symbol identity gate exists to rule out. Two shapes count:
///
/// - a change touching only files that are byte-for-byte identical across the
///   two revisions, so the code did not move but identity did
/// - a rename whose name did not actually change, which means identity shifted
///   under a symbol that stayed exactly where it was
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Phantom {
    pub before_revision: String,
    pub after_revision: String,
    pub reason: PhantomReason,
    pub change: SymbolChange,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PhantomReason {
    /// Every file this change touches is unchanged.
    UnchangedFile,
    /// Identity moved while the qualified name stayed put.
    StableNameNewIdentity,
}

/// What the audit found across a run of revisions.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct IdentityAudit {
    pub comparisons: usize,
    pub symbols_seen: usize,
    pub changes: usize,
    pub added: usize,
    pub removed: usize,
    pub modified: usize,
    pub moved: usize,
    pub renamed: usize,
    pub phantoms: Vec<Phantom>,
}

impl IdentityAudit {
    pub fn passed(&self) -> bool {
        self.phantoms.is_empty()
    }
}

/// Run delta across consecutive revisions and count phantoms.
///
/// `revisions` is oldest first. Each adjacent pair is compared.
pub fn audit_identity(
    repo: &git2::Repository,
    revisions: &[String],
    config: &Config,
) -> Result<IdentityAudit> {
    let mut audit = IdentityAudit::default();
    let mut previous: Option<(String, Analysis)> = None;

    for revision in revisions {
        let analysis = analyze_revision(repo, revision, config, None)?;

        if let Some((before_revision, before)) = previous {
            let delta = compare(&before, &analysis);
            audit.comparisons += 1;
            audit.changes += delta.changes.len();

            let unchanged_files = unchanged(&before, &analysis);

            for change in delta.changes {
                match change.change {
                    ChangeKind::Added => audit.added += 1,
                    ChangeKind::Removed => audit.removed += 1,
                    ChangeKind::Modified => audit.modified += 1,
                    ChangeKind::Moved => audit.moved += 1,
                    ChangeKind::Renamed => audit.renamed += 1,
                }

                if let Some(reason) = phantom_reason(&change, &unchanged_files) {
                    audit.phantoms.push(Phantom {
                        before_revision: before_revision.clone(),
                        after_revision: revision.clone(),
                        reason,
                        change,
                    });
                }
            }
        }

        audit.symbols_seen += analysis.symbols().count();
        previous = Some((revision.clone(), analysis));
    }

    Ok(audit)
}

fn phantom_reason(
    change: &SymbolChange,
    unchanged_files: &std::collections::HashSet<&str>,
) -> Option<PhantomReason> {
    if change.paths().iter().all(|p| unchanged_files.contains(p)) {
        return Some(PhantomReason::UnchangedFile);
    }

    if change.change == ChangeKind::Renamed {
        let before = change.before.as_ref()?;
        let after = change.after.as_ref()?;
        if before.qualified_path == after.qualified_path {
            return Some(PhantomReason::StableNameNewIdentity);
        }
    }

    None
}

/// Paths whose content is byte-for-byte identical in both analyses.
fn unchanged<'a>(before: &'a Analysis, after: &'a Analysis) -> std::collections::HashSet<&'a str> {
    let before_hashes: HashMap<&str, &str> = before
        .files
        .iter()
        .map(|f| (f.path.as_str(), f.content_hash.as_str()))
        .collect();

    after
        .files
        .iter()
        .filter(|f| before_hashes.get(f.path.as_str()) == Some(&f.content_hash.as_str()))
        .map(|f| f.path.as_str())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::walk::SourceFile;

    fn analyze(files: &[(&str, &str)]) -> Analysis {
        let sources: Vec<_> = files
            .iter()
            .map(|(path, contents)| SourceFile {
                path: (*path).to_owned(),
                contents: (*contents).to_owned(),
            })
            .collect();
        analysis::run(&sources, &Config::default())
    }

    #[test]
    fn identical_revisions_report_nothing() {
        let files = [("src/lib.rs", "fn draw(a: bool) {\n    if a {}\n}\n")];
        assert_eq!(
            compare(&analyze(&files), &analyze(&files)),
            Delta::default()
        );
    }

    #[test]
    fn a_line_shift_is_not_a_change() {
        let before = analyze(&[("src/lib.rs", "fn draw() {}\n")]);
        let after = analyze(&[("src/lib.rs", "// a new header comment\n\nfn draw() {}\n")]);

        assert_eq!(compare(&before, &after).changes, vec![]);
    }

    #[test]
    fn a_complexity_change_is_reported_with_both_sides() {
        let before = analyze(&[("src/lib.rs", "fn draw(a: bool) {\n    if a {}\n}\n")]);
        let after = analyze(&[(
            "src/lib.rs",
            "fn draw(a: bool) {\n    if a {\n        if a {}\n    }\n}\n",
        )]);

        let changes = compare(&before, &after).changes;
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].change, ChangeKind::Modified);
        assert_eq!(changes[0].before.as_ref().unwrap().cognitive_complexity, 1);
        assert_eq!(changes[0].after.as_ref().unwrap().cognitive_complexity, 3);
        assert_eq!(changes[0].complexity_delta, Some(2));
    }

    #[test]
    fn a_move_between_files_is_a_move_not_a_delete_and_an_add() {
        let body = "fn draw(a: bool) {\n    if a {}\n}\n";
        let before = analyze(&[("src/old.rs", body)]);
        let after = analyze(&[("src/new.rs", body)]);

        let changes = compare(&before, &after).changes;
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].change, ChangeKind::Moved);
        assert_eq!(changes[0].before.as_ref().unwrap().path, "src/old.rs");
        assert_eq!(changes[0].after.as_ref().unwrap().path, "src/new.rs");
    }

    #[test]
    fn a_rename_is_a_rename_not_a_delete_and_an_add() {
        let before = analyze(&[("src/lib.rs", "fn draw(a: bool) {\n    if a {}\n}\n")]);
        let after = analyze(&[("src/lib.rs", "fn paint(a: bool) {\n    if a {}\n}\n")]);

        let changes = compare(&before, &after).changes;
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].change, ChangeKind::Renamed);
        assert_eq!(changes[0].before.as_ref().unwrap().qualified_path, "draw");
        assert_eq!(changes[0].after.as_ref().unwrap().qualified_path, "paint");
    }

    #[test]
    fn genuinely_new_and_gone_symbols_are_added_and_removed() {
        let before = analyze(&[("src/lib.rs", "fn gone() {}\n")]);
        let after = analyze(&[("src/lib.rs", "fn brand_new(a: bool) {\n    if a {}\n}\n")]);

        let mut kinds: Vec<_> = compare(&before, &after)
            .changes
            .into_iter()
            .map(|c| format!("{:?}", c.change))
            .collect();
        kinds.sort();

        assert_eq!(kinds, vec!["Added", "Removed"]);
    }

    #[test]
    fn methods_sharing_a_name_do_not_swap_places() {
        let source = |inherent: &str, trait_impl: &str| {
            format!(
                "struct Canvas;\ntrait Paint {{ fn draw(&self); }}\nimpl Canvas {{ fn draw(&self) {{ {inherent} }} }}\nimpl Paint for Canvas {{ fn draw(&self) {{ {trait_impl} }} }}\n"
            )
        };

        let before = analyze(&[("src/lib.rs", &source("a();", "b();"))]);
        let after = analyze(&[("src/lib.rs", &source("a();", "c();"))]);

        let changes = compare(&before, &after).changes;
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].change, ChangeKind::Modified);
        assert_eq!(
            changes[0].after.as_ref().unwrap().qualified_path,
            "<Canvas as Paint>::draw"
        );
    }

    #[test]
    fn changing_a_closure_is_a_change_to_its_function_only() {
        let before = analyze(&[("src/lib.rs", "fn run() {\n    let f = |x: u32| x;\n}\n")]);
        let after = analyze(&[(
            "src/lib.rs",
            "fn run() {\n    let f = |x: u32| if x > 0 { x } else { 0 };\n}\n",
        )]);

        let changes = compare(&before, &after).changes;
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].change, ChangeKind::Modified);
        assert_eq!(changes[0].after.as_ref().unwrap().qualified_path, "run");
    }

    #[test]
    fn output_order_is_deterministic() {
        let before = analyze(&[("z.rs", "fn z() {}\n"), ("a.rs", "fn a() {}\n")]);
        let after = analyze(&[
            ("z.rs", "fn z(v: bool) {\n    if v {}\n}\n"),
            ("a.rs", "fn a(v: bool) {\n    if v {}\n}\n"),
        ]);

        let paths: Vec<_> = compare(&before, &after)
            .changes
            .into_iter()
            .map(|c| c.after.unwrap().path)
            .collect();

        assert_eq!(paths, vec!["a.rs", "z.rs"]);
    }
}

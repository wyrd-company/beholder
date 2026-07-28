//! Symbol identity across revisions.
//!
//! Everything downstream of the index is matched by symbol identity, so an
//! identity that drifts when the code does not makes every report untrustworthy.
//! These tests measure that directly: a phantom is a symbol change reported when
//! the code did not change.

use std::path::PathBuf;
use std::process::Command;

use beholder::delta::{self, ChangeKind};
use beholder::Config;
use git2::{Oid, Repository};

// ---------------------------------------------------------------------------
// fixtures
// ---------------------------------------------------------------------------

struct Fixture {
    _dir: tempfile::TempDir,
    repo: Repository,
}

impl Fixture {
    fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();
        let repo = Repository::init(dir.path()).unwrap();
        Self { _dir: dir, repo }
    }

    fn root(&self) -> PathBuf {
        self.repo.workdir().unwrap().to_owned()
    }

    fn write(&self, path: &str, contents: &str) {
        let full = self.root().join(path);
        std::fs::create_dir_all(full.parent().unwrap()).unwrap();
        std::fs::write(full, contents).unwrap();
    }

    fn commit_all(&self, message: &str) -> Oid {
        let mut index = self.repo.index().unwrap();
        index
            .add_all(["*"], git2::IndexAddOption::DEFAULT, None)
            .unwrap();
        index.write().unwrap();
        let tree = self.repo.find_tree(index.write_tree().unwrap()).unwrap();

        let signature =
            git2::Signature::new("test", "test@example.invalid", &git2::Time::new(0, 0)).unwrap();
        let parents: Vec<git2::Commit<'_>> = self
            .repo
            .head()
            .ok()
            .and_then(|h| h.peel_to_commit().ok())
            .into_iter()
            .collect();
        let parent_refs: Vec<&git2::Commit<'_>> = parents.iter().collect();

        self.repo
            .commit(
                Some("HEAD"),
                &signature,
                &signature,
                message,
                &tree,
                &parent_refs,
            )
            .unwrap()
    }
}

/// A file whose formatting rustfmt will visibly change, but whose meaning it
/// will not. Deliberately not the beholder domain.
const CRAMPED: &str = r#"
pub struct Ledger{pub entries:Vec<u32>}
impl Ledger{
pub fn tally(&self,threshold:u32)->u32{
let mut total=0;
for entry in &self.entries{
if *entry>threshold{if entry%2==0{total+=entry;}else{total+=1;}}else{total+=0;}
}
total}
pub fn empty(&self)->bool{self.entries.is_empty()&&self.entries.capacity()==0}
}
pub enum Posting{Debit(u32),Credit(u32)}
pub fn settle(a:&Ledger,b:&Ledger)->u32{a.tally(0)+b.tally(0)}
"#;

fn rustfmt(source: &str) -> Option<String> {
    let dir = tempfile::tempdir().ok()?;
    let path = dir.path().join("input.rs");
    std::fs::write(&path, source).ok()?;

    let status = Command::new("rustfmt")
        .arg("--edition")
        .arg("2021")
        .arg(&path)
        .status()
        .ok()?;

    status
        .success()
        .then(|| std::fs::read_to_string(&path).ok())?
}

// ---------------------------------------------------------------------------
// the formatting-only commit
// ---------------------------------------------------------------------------

#[test]
fn a_formatting_only_commit_reports_no_changed_symbols() {
    let Some(formatted) = rustfmt(CRAMPED) else {
        eprintln!("skipping: rustfmt is not available");
        return;
    };

    assert_ne!(
        formatted, CRAMPED,
        "the fixture must actually be reformatted or this test proves nothing"
    );

    let fixture = Fixture::new();
    fixture.write("src/ledger.rs", CRAMPED);
    let before = fixture.commit_all("cramped");

    fixture.write("src/ledger.rs", &formatted);
    let after = fixture.commit_all("reformat only");

    assert_ne!(before, after);

    let delta = delta::compare_revisions(
        &fixture.repo,
        &before.to_string(),
        &after.to_string(),
        &Config::default(),
        None,
    )
    .unwrap();

    assert_eq!(
        delta.changes,
        vec![],
        "a reformat is not a change: {:#?}",
        delta.changes
    );
}

#[test]
fn a_formatting_only_commit_still_shifts_line_ranges() {
    // Guards the test above from passing for the wrong reason: the analysis is
    // seeing different line numbers and reporting no change anyway.
    let Some(formatted) = rustfmt(CRAMPED) else {
        eprintln!("skipping: rustfmt is not available");
        return;
    };

    let before = beholder::phase1::analyze("src/ledger.rs", CRAMPED);
    let after = beholder::phase1::analyze("src/ledger.rs", &formatted);

    let ranges = |a: &beholder::FileAnalysis| {
        a.symbols
            .iter()
            .map(|s| (s.start_line, s.end_line))
            .collect::<Vec<_>>()
    };

    assert!(!before.symbols.is_empty());
    assert_ne!(ranges(&before), ranges(&after), "line ranges must differ");
    assert_eq!(
        before.symbols.iter().map(|s| &s.id).collect::<Vec<_>>(),
        after.symbols.iter().map(|s| &s.id).collect::<Vec<_>>(),
        "identity must not"
    );
}

// ---------------------------------------------------------------------------
// the failure cases named in the gate
// ---------------------------------------------------------------------------

#[test]
fn adding_a_symbol_does_not_disturb_its_neighbours() {
    let fixture = Fixture::new();
    fixture.write(
        "src/ledger.rs",
        "pub fn tally(n: u32) -> u32 {\n    if n > 0 {\n        n\n    } else {\n        0\n    }\n}\n",
    );
    let before = fixture.commit_all("one function");

    fixture.write(
        "src/ledger.rs",
        "pub fn settle() {}\n\npub fn tally(n: u32) -> u32 {\n    if n > 0 {\n        n\n    } else {\n        0\n    }\n}\n",
    );
    let after = fixture.commit_all("insert a function above it");

    let delta = delta::compare_revisions(
        &fixture.repo,
        &before.to_string(),
        &after.to_string(),
        &Config::default(),
        None,
    )
    .unwrap();

    assert_eq!(delta.changes.len(), 1, "{:#?}", delta.changes);
    assert_eq!(delta.changes[0].change, ChangeKind::Added);
    assert_eq!(
        delta.changes[0].after.as_ref().unwrap().qualified_path,
        "settle"
    );
}

#[test]
fn moving_a_symbol_between_files_is_one_move_not_two_changes() {
    let body = "pub fn tally(n: u32) -> u32 {\n    if n > 0 {\n        n\n    } else {\n        0\n    }\n}\n";

    let fixture = Fixture::new();
    fixture.write("src/old_home.rs", body);
    let before = fixture.commit_all("original home");

    std::fs::remove_file(fixture.root().join("src/old_home.rs")).unwrap();
    fixture.write("src/new_home.rs", body);
    let after = fixture.commit_all("move it");

    let delta = delta::compare_revisions(
        &fixture.repo,
        &before.to_string(),
        &after.to_string(),
        &Config::default(),
        None,
    )
    .unwrap();

    assert_eq!(delta.changes.len(), 1, "{:#?}", delta.changes);
    assert_eq!(delta.changes[0].change, ChangeKind::Moved);
}

// ---------------------------------------------------------------------------
// review reproductions
// ---------------------------------------------------------------------------

#[test]
fn deleting_one_cfg_variant_reports_one_removal_and_nothing_else() {
    // Reviewer reproduction. Positional ordinals made the surviving variant
    // change identity when its twin was deleted, so one deletion surfaced as a
    // removal plus a spurious rename, and audit-identity flagged a phantom.
    let both = "#[cfg(unix)]\npub fn platform() -> u32 {\n    1\n}\n\n#[cfg(windows)]\npub fn platform() -> u32 {\n    2\n}\n";
    let one = "#[cfg(windows)]\npub fn platform() -> u32 {\n    2\n}\n";

    let fixture = Fixture::new();
    fixture.write("src/platform.rs", both);
    let before = fixture.commit_all("both variants");

    fixture.write("src/platform.rs", one);
    let after = fixture.commit_all("drop the unix variant");

    let delta = delta::compare_revisions(
        &fixture.repo,
        &before.to_string(),
        &after.to_string(),
        &Config::default(),
        None,
    )
    .unwrap();

    assert_eq!(delta.changes.len(), 1, "{:#?}", delta.changes);
    assert_eq!(delta.changes[0].change, ChangeKind::Removed);
    assert_eq!(
        delta.changes[0]
            .before
            .as_ref()
            .unwrap()
            .discriminator
            .as_deref(),
        Some("cfg(unix)")
    );

    let audit = delta::audit_identity(
        &fixture.repo,
        &[before.to_string(), after.to_string()],
        &Config::default(),
    )
    .unwrap();

    assert!(audit.passed(), "{:#?}", audit.phantoms);
    assert_eq!(audit.renamed, 0);
}

#[test]
fn a_macro_trailing_comma_edit_is_reported_as_a_change() {
    // Reviewer reproduction. The optional-trailing rule reached inside macro
    // token trees, so a semantic edit produced an empty delta.
    let fixture = Fixture::new();
    fixture.write(
        "src/ledger.rs",
        "pub fn tally() {\n    count_fields!(entries)\n}\n",
    );
    let before = fixture.commit_all("one form");

    fixture.write(
        "src/ledger.rs",
        "pub fn tally() {\n    count_fields!(entries,)\n}\n",
    );
    let after = fixture.commit_all("a different macro input");

    let delta = delta::compare_revisions(
        &fixture.repo,
        &before.to_string(),
        &after.to_string(),
        &Config::default(),
        None,
    )
    .unwrap();

    assert_eq!(delta.changes.len(), 1, "{:#?}", delta.changes);
    assert_eq!(delta.changes[0].change, ChangeKind::Modified);
}

// ---------------------------------------------------------------------------
// real history
// ---------------------------------------------------------------------------

/// A Wyrd Rust repository to audit, when one is checked out next to this one.
fn fixture_repository() -> Option<PathBuf> {
    if let Ok(path) = std::env::var("BEHOLDER_AUDIT_REPO") {
        return Some(PathBuf::from(path));
    }

    ["/workspaces/tools/tagver", "/workspaces/tools/intentional"]
        .into_iter()
        .map(PathBuf::from)
        .find(|p| p.join(".git").exists())
}

fn first_parent_revisions(repo: &Repository, depth: usize) -> Vec<String> {
    let mut walk = repo.revwalk().unwrap();
    walk.push_head().unwrap();
    walk.set_sorting(git2::Sort::TOPOLOGICAL).unwrap();
    walk.simplify_first_parent().unwrap();

    let mut revisions: Vec<String> = walk.take(depth).map(|id| id.unwrap().to_string()).collect();
    revisions.reverse();
    revisions
}

#[test]
fn real_history_produces_no_phantom_changes() {
    let Some(path) = fixture_repository() else {
        eprintln!("skipping: no fixture repository checked out");
        return;
    };

    let repo = Repository::open(&path).unwrap();
    let revisions = first_parent_revisions(&repo, 30);

    assert!(
        revisions.len() > 5,
        "the audit needs real history, found {} revisions in {}",
        revisions.len(),
        path.display()
    );

    let audit = delta::audit_identity(&repo, &revisions, &Config::default()).unwrap();

    assert!(
        audit.symbols_seen > 0,
        "the fixture repository has no symbols"
    );
    assert!(audit.changes > 0, "the fixture history changed nothing");
    assert!(
        audit.passed(),
        "{} phantom changes across {} comparisons of {}: {:#?}",
        audit.phantoms.len(),
        audit.comparisons,
        path.display(),
        audit.phantoms
    );
}

#[test]
fn the_audit_counts_real_changes_and_ignores_a_reformat() {
    // Guards the audit from passing vacuously. Over a three-commit history it
    // must see the real edit, see nothing in the reformat, and find no phantom
    // in the file that was never touched.
    let Some(formatted) = rustfmt(CRAMPED) else {
        eprintln!("skipping: rustfmt is not available");
        return;
    };

    let fixture = Fixture::new();
    fixture.write("src/ledger.rs", CRAMPED);
    fixture.write("src/untouched.rs", "pub fn settle() {}\n");
    let first = fixture.commit_all("first");

    fixture.write(
        "src/ledger.rs",
        &format!("{CRAMPED}\npub fn audit(n:u32)->u32{{if n>0{{n}}else{{0}}}}\n"),
    );
    let second = fixture.commit_all("add a function");

    fixture.write("src/ledger.rs", &formatted);
    let third = fixture.commit_all("reformat and drop the addition");

    let audit = delta::audit_identity(
        &fixture.repo,
        &[first.to_string(), second.to_string(), third.to_string()],
        &Config::default(),
    )
    .unwrap();

    assert_eq!(audit.comparisons, 2);
    assert_eq!(audit.added, 1, "the second commit added one function");
    assert_eq!(audit.removed, 1, "the third commit removed it again");
    assert_eq!(
        audit.modified, 0,
        "nothing else changed structurally: {:#?}",
        audit.phantoms
    );
    assert!(audit.passed(), "{:#?}", audit.phantoms);
}

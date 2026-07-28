//! The custom git ref is v1 product behavior, so it is tested as product
//! behavior: the commit convention, the validation rules, concurrent writers,
//! and portability through ordinary fetch and push.

use std::path::Path;

use beholder::analysis::{self, Analysis, Fingerprint};
use beholder::store::{Store, REFSPEC};
use beholder::walk::SourceFile;
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

    fn path(&self) -> &Path {
        self.repo.workdir().unwrap()
    }

    /// Write and commit one file, returning the new commit id.
    fn commit(&self, path: &str, contents: &str) -> Oid {
        let full = self.path().join(path);
        std::fs::create_dir_all(full.parent().unwrap()).unwrap();
        std::fs::write(&full, contents).unwrap();

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
                &format!("commit {path}"),
                &tree,
                &parent_refs,
            )
            .unwrap()
    }
}

/// Analyze a revision exactly as beholder would.
///
/// Stored results are validated against the tree they claim to describe, so a
/// test that stored a hand-built file set would only be proving that the
/// validation is missing.
fn analyze_at(repo: &Repository, source: Oid) -> Analysis {
    let files =
        beholder::walk::read_revision(repo, &source.to_string(), &Config::default()).unwrap();
    analysis::run(&files, &Config::default())
}

fn config() -> Config {
    Config::default()
}

// ---------------------------------------------------------------------------
// the index commit convention
// ---------------------------------------------------------------------------

#[test]
fn the_first_index_has_only_the_source_parent() {
    let fixture = Fixture::new();
    let source = fixture.commit("src/lib.rs", "fn draw() {}\n");
    let store = Store::open(&fixture.repo);

    let index = store
        .write(source, &analyze_at(&fixture.repo, source))
        .unwrap();
    let commit = fixture.repo.find_commit(index).unwrap();

    assert_eq!(commit.parent_count(), 1);
    assert_eq!(commit.parent_id(0).unwrap(), source);
}

#[test]
fn later_indexes_carry_the_source_then_the_previous_index() {
    let fixture = Fixture::new();
    let store = Store::open(&fixture.repo);

    let first_source = fixture.commit("src/lib.rs", "fn draw() {}\n");
    let first_index = store
        .write(first_source, &analyze_at(&fixture.repo, first_source))
        .unwrap();

    let second_source = fixture.commit("src/lib.rs", "fn draw() {}\nfn erase() {}\n");
    let second_index = store
        .write(second_source, &analyze_at(&fixture.repo, second_source))
        .unwrap();

    let commit = fixture.repo.find_commit(second_index).unwrap();
    assert_eq!(commit.parent_count(), 2);
    assert_eq!(
        commit.parent_id(0).unwrap(),
        second_source,
        "parent 0 is the source"
    );
    assert_eq!(
        commit.parent_id(1).unwrap(),
        first_index,
        "parent 1 is the previous index"
    );
}

#[test]
fn history_walks_parent_one_newest_first() {
    let fixture = Fixture::new();
    let store = Store::open(&fixture.repo);
    let mut sources = Vec::new();

    for n in 0..3 {
        let contents = format!("fn draw() {{}}\n// revision {n}\n");
        let source = fixture.commit("src/lib.rs", &contents);
        store
            .write(source, &analyze_at(&fixture.repo, source))
            .unwrap();
        sources.push(source);
    }

    let history = store.history(10).unwrap();
    let walked: Vec<_> = history.iter().map(|e| e.source_commit).collect();

    sources.reverse();
    assert_eq!(walked, sources);
}

#[test]
fn the_ref_lives_outside_refs_heads() {
    let fixture = Fixture::new();
    let store = Store::open(&fixture.repo);
    let source = fixture.commit("src/lib.rs", "fn draw() {}\n");
    store
        .write(source, &analyze_at(&fixture.repo, source))
        .unwrap();

    assert_eq!(store.refname(), "refs/beholder/index");
    assert!(!store.refname().starts_with("refs/heads/"));
    assert!(fixture.repo.find_reference("refs/beholder/index").is_ok());
}

#[test]
fn index_commits_are_reproducible() {
    // Two machines analyzing the same source produce the same index commit.
    let build = || {
        let fixture = Fixture::new();
        let source = fixture.commit("src/lib.rs", "fn draw() {}\n");
        let store = Store::open(&fixture.repo);
        let index = store
            .write(source, &analyze_at(&fixture.repo, source))
            .unwrap();
        (source, index)
    };

    let (source_a, index_a) = build();
    let (source_b, index_b) = build();

    assert_eq!(source_a, source_b);
    assert_eq!(index_a, index_b);
}

// ---------------------------------------------------------------------------
// reuse and validation
// ---------------------------------------------------------------------------

#[test]
fn a_stored_analysis_round_trips() {
    let fixture = Fixture::new();
    let store = Store::open(&fixture.repo);
    let source = fixture.commit("src/lib.rs", "fn draw(a: bool) {\n    if a {}\n}\n");
    let original = analyze_at(&fixture.repo, source);

    store.write(source, &original).unwrap();

    let found = store
        .find(source, &Fingerprint::new(&config()), &config())
        .unwrap()
        .expect("a valid stored analysis");

    assert_eq!(found, original);
}

#[test]
fn a_stored_analysis_of_another_commit_is_not_reused() {
    let fixture = Fixture::new();
    let store = Store::open(&fixture.repo);
    let stored_source = fixture.commit("src/lib.rs", "fn draw() {}\n");
    store
        .write(stored_source, &analyze_at(&fixture.repo, stored_source))
        .unwrap();

    let other_source = fixture.commit("src/lib.rs", "fn erase() {}\n");

    assert!(store
        .find(other_source, &Fingerprint::new(&config()), &config())
        .unwrap()
        .is_none());
}

#[test]
fn a_stored_analysis_under_a_different_configuration_is_not_reused() {
    let fixture = Fixture::new();
    let store = Store::open(&fixture.repo);
    let source = fixture.commit("src/lib.rs", "fn draw() {}\n");
    store
        .write(source, &analyze_at(&fixture.repo, source))
        .unwrap();

    let other = Config::from_toml(r#"ignored_paths = ["vendor/"]"#).unwrap();

    assert!(store
        .find(source, &Fingerprint::new(&other), &other)
        .unwrap()
        .is_none());
}

#[test]
fn a_stored_analysis_under_a_different_schema_or_tool_version_is_not_reused() {
    let fixture = Fixture::new();
    let store = Store::open(&fixture.repo);
    let source = fixture.commit("src/lib.rs", "fn draw() {}\n");
    store
        .write(source, &analyze_at(&fixture.repo, source))
        .unwrap();

    let mut future_schema = Fingerprint::new(&config());
    future_schema.schema_version += 1;
    assert!(store
        .find(source, &future_schema, &config())
        .unwrap()
        .is_none());

    let mut future_tool = Fingerprint::new(&config());
    future_tool.tool_version = "999.0.0".into();
    assert!(store
        .find(source, &future_tool, &config())
        .unwrap()
        .is_none());

    let mut other_languages = Fingerprint::new(&config());
    other_languages.language_table = "different".into();
    assert!(store
        .find(source, &other_languages, &config())
        .unwrap()
        .is_none());

    let mut other_paths = Fingerprint::new(&config());
    other_paths.path_rules = "absolute".into();
    assert!(store
        .find(source, &other_paths, &config())
        .unwrap()
        .is_none());
}

#[test]
fn phase1_results_are_reused_only_for_unchanged_content() {
    let fixture = Fixture::new();
    let store = Store::open(&fixture.repo);

    let unchanged = ("src/keep.rs", "fn keep() {}\n");
    let after = ("src/edit.rs", "fn edit(a: bool) {\n    if a {}\n}\n");

    fixture.commit("src/keep.rs", unchanged.1);
    let source = fixture.commit("src/edit.rs", "fn edit() {}\n");
    store
        .write(source, &analyze_at(&fixture.repo, source))
        .unwrap();

    let fingerprint = Fingerprint::new(&config());
    let cache = store.phase1_cache(&fingerprint).unwrap();

    let sources: Vec<_> = [unchanged, after]
        .iter()
        .map(|(path, contents)| SourceFile {
            path: (*path).to_owned(),
            contents: (*contents).to_owned(),
        })
        .collect();

    let (analysis, stats) = analysis::run_with_cache(&sources, &Config::default(), &cache);

    assert_eq!(stats.reused, 1, "only the unchanged file is reusable");
    assert_eq!(stats.computed, 1);

    let edited = analysis
        .symbols()
        .find(|s| s.path == "src/edit.rs")
        .unwrap();
    assert_eq!(
        edited.cognitive_complexity, 1,
        "the edited file was recomputed"
    );
}

#[test]
fn phase1_reuse_does_not_depend_on_the_checkout_location() {
    // The same repository, cloned to a different directory, reuses the same
    // stored results. Nothing in the cache key is machine-local.
    let origin = Fixture::new();
    let files = [("src/lib.rs", "fn draw(a: bool) {\n    if a {}\n}\n")];
    let source = origin.commit("src/lib.rs", files[0].1);
    Store::open(&origin.repo)
        .write(source, &analyze_at(&origin.repo, source))
        .unwrap();

    let clone_dir = tempfile::tempdir().unwrap();
    let clone_path = clone_dir.path().join("clone");
    let clone = Repository::clone(origin.path().to_str().unwrap(), &clone_path).unwrap();
    Store::open(&clone).fetch("origin").unwrap();

    let fingerprint = Fingerprint::new(&config());
    let cache = Store::open(&clone).phase1_cache(&fingerprint).unwrap();

    let sources: Vec<_> = files
        .iter()
        .map(|(path, contents)| SourceFile {
            path: (*path).to_owned(),
            contents: (*contents).to_owned(),
        })
        .collect();

    let (_, stats) = analysis::run_with_cache(&sources, &Config::default(), &cache);
    assert_eq!(stats.reused, 1);
    assert_eq!(stats.computed, 0);
}

// ---------------------------------------------------------------------------
// payload validation
// ---------------------------------------------------------------------------

#[test]
fn a_stored_analysis_of_different_content_is_not_reused() {
    // Metadata can agree while the payload describes something else entirely.
    // Without a payload check, a result computed from a dirty working tree
    // would be served as the analysis of the commit it was filed under.
    let fixture = Fixture::new();
    let store = Store::open(&fixture.repo);
    let source = fixture.commit("src/lib.rs", "fn draw(a: bool) {\n    if a {}\n}\n");

    let mut poisoned = analyze_at(&fixture.repo, source);
    poisoned.files[0] = beholder::phase1::analyze("src/lib.rs", "fn draw() {}\n");

    store.write(source, &poisoned).unwrap();

    assert!(store
        .find(source, &Fingerprint::new(&config()), &config())
        .unwrap()
        .is_none());
}

#[test]
fn a_stored_analysis_missing_a_file_is_not_reused() {
    let fixture = Fixture::new();
    let store = Store::open(&fixture.repo);
    fixture.commit("src/keep.rs", "fn keep() {}\n");
    let source = fixture.commit("src/other.rs", "fn other() {}\n");

    let mut truncated = analyze_at(&fixture.repo, source);
    truncated.files.pop();

    store.write(source, &truncated).unwrap();

    assert!(store
        .find(source, &Fingerprint::new(&config()), &config())
        .unwrap()
        .is_none());
}

#[test]
fn a_stored_analysis_with_an_extra_file_is_not_reused() {
    let fixture = Fixture::new();
    let store = Store::open(&fixture.repo);
    let source = fixture.commit("src/lib.rs", "fn draw() {}\n");

    let mut padded = analyze_at(&fixture.repo, source);
    padded
        .files
        .push(beholder::phase1::analyze("src/ghost.rs", "fn ghost() {}\n"));

    store.write(source, &padded).unwrap();

    assert!(store
        .find(source, &Fingerprint::new(&config()), &config())
        .unwrap()
        .is_none());
}

#[test]
fn a_non_repo_relative_path_is_refused_on_the_way_in() {
    let fixture = Fixture::new();
    let store = Store::open(&fixture.repo);
    let source = fixture.commit("src/lib.rs", "fn draw() {}\n");

    let mut escaping = analyze_at(&fixture.repo, source);
    escaping.files[0].path = "/etc/passwd".into();

    assert!(store.write(source, &escaping).is_err());
}

#[test]
fn validate_payload_names_what_went_wrong() {
    use beholder::store::{validate_payload, Invalid};

    let fixture = Fixture::new();
    let source = fixture.commit("src/lib.rs", "fn draw() {}\n");
    let analysis = analyze_at(&fixture.repo, source);
    let manifest =
        beholder::walk::revision_manifest(&fixture.repo, &source.to_string(), &config()).unwrap();

    assert!(validate_payload(&analysis, &manifest).is_ok());

    let mut wrong_path = analysis.clone();
    wrong_path.files[0].path = "../escape.rs".into();
    assert!(matches!(
        validate_payload(&wrong_path, &manifest),
        Err(Invalid::Path { .. })
    ));

    let mut wrong_hash = analysis.clone();
    wrong_hash.files[0].content_hash = "0".repeat(64);
    assert!(matches!(
        validate_payload(&wrong_hash, &manifest),
        Err(Invalid::ContentHash { .. })
    ));

    let mut wrong_set = analysis;
    wrong_set.files.clear();
    assert!(matches!(
        validate_payload(&wrong_set, &manifest),
        Err(Invalid::FileSet { .. })
    ));
}

// ---------------------------------------------------------------------------
// portability
// ---------------------------------------------------------------------------

#[test]
fn a_fresh_clone_can_fetch_the_ref_and_reuse_the_analysis() {
    let origin = Fixture::new();
    let files = [("src/lib.rs", "fn draw(a: bool) {\n    if a {}\n}\n")];
    let source = origin.commit("src/lib.rs", files[0].1);
    let original = analyze_at(&origin.repo, source);
    Store::open(&origin.repo).write(source, &original).unwrap();

    let clone_dir = tempfile::tempdir().unwrap();
    let clone_path = clone_dir.path().join("clone");
    let clone = Repository::clone(origin.path().to_str().unwrap(), &clone_path).unwrap();

    // Nothing is available before the ref is fetched.
    assert!(Store::open(&clone).tip().unwrap().is_none());

    Store::open(&clone).fetch("origin").unwrap();

    let store = Store::open(&clone);
    assert!(store.tip().is_ok());
    let found = store
        .find(source, &Fingerprint::new(&config()), &config())
        .unwrap()
        .expect("the fetched analysis");

    assert_eq!(found, original);
}

#[test]
fn the_ref_pushes_with_an_ordinary_refspec() {
    let origin_dir = tempfile::tempdir().unwrap();
    let origin_path = origin_dir.path().join("origin.git");
    Repository::init_bare(&origin_path).unwrap();

    let fixture = Fixture::new();
    fixture
        .repo
        .remote("origin", origin_path.to_str().unwrap())
        .unwrap();

    let files = [("src/lib.rs", "fn draw() {}\n")];
    let source = fixture.commit("src/lib.rs", files[0].1);
    let store = Store::open(&fixture.repo);
    let index = store
        .write(source, &analyze_at(&fixture.repo, source))
        .unwrap();

    store.push("origin").unwrap();

    let origin = Repository::open(&origin_path).unwrap();
    assert_eq!(
        origin
            .find_reference("refs/beholder/index")
            .unwrap()
            .target()
            .unwrap(),
        index
    );
    // Pushing the index carried the analyzed source objects with it.
    assert!(origin.find_commit(source).is_ok());
    assert_eq!(REFSPEC, "+refs/beholder/*:refs/beholder/*");
}

#[test]
fn a_second_pusher_rebuilds_instead_of_overwriting() {
    // Compare-and-swap on the local ref only protects writers in one clone. A
    // forced push would let the second machine drop the first machine's index
    // without ever noticing it existed.
    let origin_dir = tempfile::tempdir().unwrap();
    let origin_path = origin_dir.path().join("origin.git");
    Repository::init_bare(&origin_path).unwrap();
    let origin_url = origin_path.to_str().unwrap().to_owned();

    // A shared history both machines already have.
    let seed = Fixture::new();
    seed.repo.remote("origin", &origin_url).unwrap();
    let first_source = seed.commit("src/first.rs", "fn first() {}\n");
    let second_source = seed.commit("src/second.rs", "fn second() {}\n");
    seed.repo
        .find_remote("origin")
        .unwrap()
        .push(&["refs/heads/master:refs/heads/master"], None)
        .or_else(|_| {
            seed.repo
                .find_remote("origin")
                .unwrap()
                .push(&["refs/heads/main:refs/heads/main"], None)
        })
        .unwrap();

    let clone_of = |name: &str| {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(name);
        let repo = Repository::clone(&origin_url, &path).unwrap();
        (dir, repo)
    };

    let (_a_dir, a) = clone_of("a");
    let (_b_dir, b) = clone_of("b");

    // Machine A indexes one commit and pushes first.
    let a_index = Store::open(&a)
        .write(first_source, &analyze_at(&a, first_source))
        .unwrap();
    Store::open(&a).push("origin").unwrap();

    // Machine B never saw A's index and indexes a different commit.
    let b_index = Store::open(&b)
        .write(second_source, &analyze_at(&b, second_source))
        .unwrap();
    assert_eq!(Store::open(&b).tip().unwrap(), Some(b_index));

    Store::open(&b).push("origin").unwrap();

    // A's index survived, and B's now sits on top of it.
    let origin = Repository::open(&origin_path).unwrap();
    let tip = origin
        .find_reference("refs/beholder/index")
        .unwrap()
        .target()
        .unwrap();

    assert_ne!(
        tip, b_index,
        "B's original index was built on nothing; a rebuilt one must differ"
    );

    let tip_commit = origin.find_commit(tip).unwrap();
    assert_eq!(tip_commit.parent_id(0).unwrap(), second_source);
    assert_eq!(
        tip_commit.parent_id(1).unwrap(),
        a_index,
        "B rebuilt on A's tip rather than replacing it"
    );

    let store = Store::open(&b);
    let sources: Vec<_> = store
        .history(10)
        .unwrap()
        .into_iter()
        .map(|e| e.source_commit)
        .collect();
    assert_eq!(sources, vec![second_source, first_source]);
}

// ---------------------------------------------------------------------------
// concurrent writers
// ---------------------------------------------------------------------------

#[test]
fn a_concurrent_writer_forces_a_retry_and_loses_nothing() {
    let fixture = Fixture::new();
    let mine = ("src/mine.rs", "fn mine() {}\n");
    let theirs = ("src/theirs.rs", "fn theirs() {}\n");

    let first_source = fixture.commit("src/mine.rs", mine.1);
    let their_source = fixture.commit("src/theirs.rs", theirs.1);

    // Their index is already on the ref.
    let their_index = Store::open(&fixture.repo)
        .write(their_source, &analyze_at(&fixture.repo, their_source))
        .unwrap();

    // My write reads the tip, then a third writer lands another index before my
    // ref update goes through.
    let mut interloper_index = None;
    let mut probe = |attempt: usize| {
        if attempt == 0 {
            let extra = fixture.commit("src/extra.rs", "fn extra() {}\n");
            interloper_index = Some(
                Store::open(&fixture.repo)
                    .write(extra, &analyze_at(&fixture.repo, extra))
                    .unwrap(),
            );
        }
    };

    let my_index = Store::open(&fixture.repo)
        .write_with_probe(
            first_source,
            &analyze_at(&fixture.repo, first_source),
            &mut probe,
        )
        .unwrap();

    let interloper_index = interloper_index.unwrap();
    let store = Store::open(&fixture.repo);

    assert_eq!(
        store.tip().unwrap(),
        Some(my_index),
        "my write won in the end"
    );

    let commit = fixture.repo.find_commit(my_index).unwrap();
    assert_eq!(
        commit.parent_id(1).unwrap(),
        interloper_index,
        "the retry rebuilt against the new beholder parent"
    );

    // Every earlier result is still reachable and still valid.
    let fingerprint = Fingerprint::new(&config());
    let history: Vec<_> = store
        .history(10)
        .unwrap()
        .into_iter()
        .map(|e| e.id)
        .collect();
    assert_eq!(history, vec![my_index, interloper_index, their_index]);
    assert!(store
        .find(their_source, &fingerprint, &config())
        .unwrap()
        .is_some());
}

#[test]
fn a_concurrent_first_writer_does_not_get_clobbered() {
    let fixture = Fixture::new();
    let mine = ("src/mine.rs", "fn mine() {}\n");
    let source = fixture.commit("src/mine.rs", mine.1);

    // The ref does not exist when my write starts, but it does by the time the
    // update runs.
    let mut theirs = None;
    let mut probe = |attempt: usize| {
        if attempt == 0 {
            let other = fixture.commit("src/theirs.rs", "fn theirs() {}\n");
            theirs = Some(
                Store::open(&fixture.repo)
                    .write(other, &analyze_at(&fixture.repo, other))
                    .unwrap(),
            );
        }
    };

    let my_index = Store::open(&fixture.repo)
        .write_with_probe(source, &analyze_at(&fixture.repo, source), &mut probe)
        .unwrap();

    let their_index = theirs.unwrap();
    let commit = fixture.repo.find_commit(my_index).unwrap();

    assert_eq!(commit.parent_count(), 2);
    assert_eq!(commit.parent_id(1).unwrap(), their_index);
}

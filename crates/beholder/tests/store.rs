//! The custom git ref is v1 product behavior, so it is tested as product
//! behavior: the commit convention, the validation rules, concurrent writers,
//! and portability through ordinary fetch and push.

mod support;

use beholder::analysis::{self, Analysis, Fingerprint};
use beholder::store::{Store, REFSPEC};
use beholder::walk::SourceFile;
use beholder::Config;
use gix::ObjectId as Oid;
use gix::Repository;
use support::{parent, parent_count, ref_target, Fixture};

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
    assert_eq!(parent_count(&fixture.repo, index), 1);
    assert_eq!(parent(&fixture.repo, index, 0).unwrap(), source);
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

    assert_eq!(parent_count(&fixture.repo, second_index), 2);
    assert_eq!(
        parent(&fixture.repo, second_index, 0).unwrap(),
        second_source,
        "parent 0 is the source"
    );
    assert_eq!(
        parent(&fixture.repo, second_index, 1).unwrap(),
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
    assert!(ref_target(&fixture.repo, "refs/beholder/index").is_some());
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
    let clone = support::clone(origin.path().to_str().unwrap(), &clone_path);
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
    let clone = support::clone(origin.path().to_str().unwrap(), &clone_path);

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

/// A bare remote and two clones of one seeded history, for the tests about
/// more than one writer.
struct Shared {
    _dirs: Vec<tempfile::TempDir>,
    origin_path: std::path::PathBuf,
    first: Oid,
    second: Oid,
}

impl Shared {
    fn new() -> Self {
        let origin_dir = tempfile::tempdir().unwrap();
        let origin_path = origin_dir.path().join("origin.git");
        support::init_bare(&origin_path);

        let seed = Fixture::new();
        seed.add_remote("origin", origin_path.to_str().unwrap());
        let first = seed.commit("src/first.rs", "fn first() {}\n");
        let second = seed.commit("src/second.rs", "fn second() {}\n");
        support::push_default_branch(seed.path());

        Self {
            _dirs: vec![origin_dir],
            origin_path,
            first,
            second,
        }
    }

    fn clone(&mut self, name: &str) -> Repository {
        let dir = tempfile::tempdir().unwrap();
        let repo = support::clone(self.origin_path.to_str().unwrap(), &dir.path().join(name));
        self._dirs.push(dir);
        repo
    }
}

#[test]
fn a_fetch_keeps_local_index_commits_on_top_of_the_remote() {
    // The ref is fetched without `+` semantics on the local side: whatever was
    // only written here survives, rebuilt on the tip that arrived.
    let mut shared = Shared::new();
    let a = shared.clone("a");
    let b = shared.clone("b");

    let a_index = Store::open(&a)
        .write(shared.first, &analyze_at(&a, shared.first))
        .unwrap();
    assert!(support::push_index(a.workdir().unwrap()));

    Store::open(&b)
        .write(shared.second, &analyze_at(&b, shared.second))
        .unwrap();

    let reconciled = Store::open(&b).fetch("origin").unwrap();

    assert_eq!(reconciled.remote, Some(a_index));
    assert_eq!(reconciled.rebuilt, 1);

    let tip = Store::open(&b).tip().unwrap().unwrap();
    assert_eq!(reconciled.tip, Some(tip));
    assert_eq!(parent(&b, tip, 0).unwrap(), shared.second);
    assert_eq!(parent(&b, tip, 1).unwrap(), a_index);

    // What the fetch left is a fast-forward of the remote.
    assert!(support::push_index(b.workdir().unwrap()));
}

#[test]
fn fetching_a_remote_with_no_index_leaves_the_local_ref_alone() {
    let mut shared = Shared::new();
    let a = shared.clone("a");

    let cold = Store::open(&a).fetch("origin").unwrap();
    assert_eq!(cold.remote, None);
    assert_eq!(cold.tip, None);

    let mine = Store::open(&a)
        .write(shared.first, &analyze_at(&a, shared.first))
        .unwrap();
    let kept = Store::open(&a).fetch("origin").unwrap();
    assert_eq!(kept.remote, None);
    assert_eq!(kept.tip, Some(mine));
    assert_eq!(Store::open(&a).tip().unwrap(), Some(mine));
}

#[test]
fn a_fetch_asks_for_the_index_and_nothing_else() {
    let origin = Fixture::new();
    let source = origin.commit("src/lib.rs", "fn draw() {}\n");
    support::git(origin.path(), &["tag", "-a", "release", "-m", "a release"]);
    Store::open(&origin.repo)
        .write(source, &analyze_at(&origin.repo, source))
        .unwrap();

    let local = Fixture::new();
    local.add_remote("origin", origin.path().to_str().unwrap());
    // Reopened, because a repository reads its configuration when it opens.
    let repo = gix::open(local.path()).unwrap();
    Store::open(&repo).fetch("origin").unwrap();

    assert!(Store::open(&repo).tip().unwrap().is_some());
    assert_eq!(
        support::git(local.path(), &["for-each-ref", "--format=%(refname)"]).trim(),
        "refs/beholder/index",
        "fetching the index must not bring down branches or tags"
    );
}

#[test]
fn rebuilding_onto_a_tip_the_local_ref_already_contains_moves_nothing() {
    // Index commits are a function of their content and parents, so replaying
    // a chain onto its own base reproduces it exactly.
    let mut shared = Shared::new();
    let a = shared.clone("a");
    let store = Store::open(&a);

    let older = store
        .write(shared.first, &analyze_at(&a, shared.first))
        .unwrap();
    let newer = store
        .write(shared.second, &analyze_at(&a, shared.second))
        .unwrap();

    let reconciled = store.rebuild_onto(older).unwrap();
    assert_eq!(reconciled.tip, Some(newer));
    assert_eq!(reconciled.rebuilt, 1);
    assert_eq!(store.tip().unwrap(), Some(newer));
}

#[test]
fn rebuilding_onto_a_commit_that_is_not_an_index_is_refused() {
    let mut shared = Shared::new();
    let a = shared.clone("a");
    let store = Store::open(&a);
    let mine = store
        .write(shared.first, &analyze_at(&a, shared.first))
        .unwrap();

    let err = store.rebuild_onto(shared.second).unwrap_err();
    assert!(
        format!("{err:#}").contains("is not an index commit"),
        "unexpected error: {err:#}"
    );
    assert_eq!(store.tip().unwrap(), Some(mine), "the ref did not move");
}

#[test]
fn rebuilding_onto_a_malformed_index_chain_is_refused() {
    // Parseable metadata alone does not make an index commit. Each forged tip
    // below carries a real index tree, and each is refused before the ref
    // moves.
    let mut shared = Shared::new();
    let a = shared.clone("a");
    let dir = a.workdir().unwrap().to_owned();
    let store = Store::open(&a);

    let genuine = store
        .write(shared.first, &analyze_at(&a, shared.first))
        .unwrap();
    let mine = store
        .write(shared.second, &analyze_at(&a, shared.second))
        .unwrap();
    let tree = support::git(&dir, &["rev-parse", &format!("{genuine}^{{tree}}")]);
    let tree = tree.trim();

    let forge = |parents: &[String]| -> Oid {
        let mut args = vec![
            "commit-tree".to_owned(),
            tree.to_owned(),
            "-m".into(),
            "forged".into(),
        ];
        for p in parents {
            args.push("-p".into());
            args.push(p.clone());
        }
        let args: Vec<&str> = args.iter().map(String::as_str).collect();
        oid(support::git(&dir, &args).trim())
    };

    // No source parent at all.
    let rootless = forge(&[]);
    // A source parent that is not the commit meta.json names.
    let mismatched = forge(&[shared.second.to_string()]);
    // A well-formed tip on top of a forged previous index.
    let on_forged = forge(&[shared.first.to_string(), rootless.to_string()]);

    for (name, tip) in [
        ("rootless", rootless),
        ("mismatched", mismatched),
        ("on a forged chain", on_forged),
    ] {
        let err = store.rebuild_onto(tip).unwrap_err();
        assert!(
            format!("{err:#}").contains("is not an index commit"),
            "{name}: unexpected error: {err:#}"
        );
        assert_eq!(store.tip().unwrap(), Some(mine), "{name}: the ref moved");
    }

    // The genuine chain is still accepted.
    assert_eq!(store.rebuild_onto(genuine).unwrap().tip, Some(mine));
}

#[test]
fn the_ref_pushes_with_an_ordinary_refspec() {
    // Beholder does not push. Ordinary git does, without `+`, and the index
    // has to arrive whole that way.
    let origin_dir = tempfile::tempdir().unwrap();
    let origin_path = origin_dir.path().join("origin.git");
    support::init_bare(&origin_path);

    let fixture = Fixture::new();
    fixture.add_remote("origin", origin_path.to_str().unwrap());

    let files = [("src/lib.rs", "fn draw() {}\n")];
    let source = fixture.commit("src/lib.rs", files[0].1);
    let store = Store::open(&fixture.repo);
    let index = store
        .write(source, &analyze_at(&fixture.repo, source))
        .unwrap();

    assert!(
        support::push_index(fixture.path()),
        "nothing else was there"
    );

    let origin = gix::open(&origin_path).unwrap();
    assert_eq!(ref_target(&origin, "refs/beholder/index").unwrap(), index);
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
    support::init_bare(&origin_path);
    let origin_url = origin_path.to_str().unwrap().to_owned();

    // A shared history both machines already have.
    let seed = Fixture::new();
    seed.add_remote("origin", &origin_url);
    let first_source = seed.commit("src/first.rs", "fn first() {}\n");
    let second_source = seed.commit("src/second.rs", "fn second() {}\n");
    support::push_default_branch(seed.path());

    let clone_of = |name: &str| {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(name);
        let repo = support::clone(&origin_url, &path);
        (dir, repo)
    };

    let (_a_dir, a) = clone_of("a");
    let (_b_dir, b) = clone_of("b");

    // Machine A indexes one commit and pushes first.
    let a_index = Store::open(&a)
        .write(first_source, &analyze_at(&a, first_source))
        .unwrap();
    assert_eq!(support::publish_index(&a), 0);

    // Machine B never saw A's index and indexes a different commit.
    let b_index = Store::open(&b)
        .write(second_source, &analyze_at(&b, second_source))
        .unwrap();
    assert_eq!(Store::open(&b).tip().unwrap(), Some(b_index));

    assert_eq!(
        support::publish_index(&b),
        1,
        "B's first push must be rejected, not forced"
    );

    // A's index survived, and B's now sits on top of it.
    let origin = gix::open(&origin_path).unwrap();
    let tip = ref_target(&origin, "refs/beholder/index").unwrap();

    assert_ne!(
        tip, b_index,
        "B's original index was built on nothing; a rebuilt one must differ"
    );

    assert_eq!(parent(&origin, tip, 0).unwrap(), second_source);
    assert_eq!(
        parent(&origin, tip, 1).unwrap(),
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

#[test]
fn reconciliation_walks_past_the_cache_depth() {
    // Cache lookup is bounded at 64 index commits because stopping early only
    // costs a recomputation. Reconciliation must not be: a cutoff either
    // orphans the local commits beyond it, or fails to recognize the commits
    // both sides share and replays them as duplicates. This diverges deeper
    // than that bound in both directions at once.
    const SHARED: usize = 8;
    const LOCAL_ONLY: usize = 70;

    let origin_dir = tempfile::tempdir().unwrap();
    let origin_path = origin_dir.path().join("origin.git");
    support::init_bare(&origin_path);
    let origin_url = origin_path.to_str().unwrap().to_owned();

    let seed = Fixture::new();
    seed.add_remote("origin", &origin_url);
    let source = seed.commit("src/lib.rs", "fn draw() {}\n");
    // A distinct source for A's late write. Index commits are a function of
    // their content and parents, so two writes of the same analysis onto the
    // same parent would converge to one commit and the divergence would be a
    // commit shallower than intended.
    let late_source = seed.commit("src/lib.rs", "fn draw() {}\nfn erase() {}\n");
    support::push_default_branch(seed.path());

    // A shared prefix, deep enough that a bounded walk from either tip would
    // stop before reaching it.
    let a_dir = tempfile::tempdir().unwrap();
    let a = support::clone(&origin_url, &a_dir.path().join("a"));
    let analysis = analyze_at(&a, source);

    for _ in 0..SHARED {
        Store::open(&a).write(source, &analysis).unwrap();
    }
    assert!(support::push_index(a.workdir().unwrap()));

    let b_dir = tempfile::tempdir().unwrap();
    let b = support::clone(&origin_url, &b_dir.path().join("b"));
    Store::open(&b).fetch("origin").unwrap();

    let shared_tip = Store::open(&b).tip().unwrap().expect("the shared prefix");
    let shared_ids: Vec<Oid> = Store::open(&b)
        .history(1000)
        .unwrap()
        .into_iter()
        .map(|e| e.id)
        .collect();
    assert_eq!(shared_ids.len(), SHARED);

    // B goes long without pushing.
    for _ in 0..LOCAL_ONLY {
        Store::open(&b).write(source, &analysis).unwrap();
    }
    let b_local: Vec<_> = Store::open(&b)
        .history(1000)
        .unwrap()
        .into_iter()
        .map(|e| e.id)
        .take(LOCAL_ONLY)
        .collect();

    // A lands one more first, so B's push is rejected.
    let a_tip = Store::open(&a)
        .write(late_source, &analyze_at(&a, late_source))
        .unwrap();
    assert!(support::push_index(a.workdir().unwrap()));

    assert_eq!(support::publish_index(&b), 1);

    // Everything survives exactly once: the shared prefix, A's commit, and all
    // seventy of B's, rebuilt on top.
    let origin = gix::open(&origin_path).unwrap();
    let pushed = Store::open(&origin).history(1000).unwrap();

    assert_eq!(
        pushed.len(),
        SHARED + 1 + LOCAL_ONLY,
        "expected the shared prefix plus both sides, got {}",
        pushed.len()
    );

    let ids: Vec<Oid> = pushed.iter().map(|e| e.id).collect();
    let unique: std::collections::HashSet<Oid> = ids.iter().copied().collect();
    assert_eq!(unique.len(), ids.len(), "history replayed a commit");

    assert!(ids.contains(&a_tip), "A's index was dropped");
    assert!(ids.contains(&shared_tip), "the shared prefix was replayed");
    assert_eq!(
        &ids[ids.len() - SHARED..],
        &shared_ids[..],
        "the shared prefix is still the same commits in the same order"
    );

    // None of B's local commits were orphaned: each was rebuilt, so none of the
    // originals is on the ref, and the count makes up the difference.
    for id in &b_local {
        assert!(
            !ids.contains(id),
            "a local commit was pushed unreconciled instead of rebuilt"
        );
    }
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

    assert_eq!(
        parent(&fixture.repo, my_index, 1).unwrap(),
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

    assert_eq!(parent_count(&fixture.repo, my_index), 2);
    assert_eq!(parent(&fixture.repo, my_index, 1).unwrap(), their_index);
}

// ---------------------------------------------------------------------------
// indexes written before the move to gix
// ---------------------------------------------------------------------------

/// An index written by beholder 0.1.1 while it still read and wrote git through
/// git2 (libgit2), made by `tests/fixtures/make-git2-index.sh`.
///
/// The ids are recorded, not derived. An index commit's id is a function of the
/// bytes of its tree and commit objects, so these literals are what an index
/// written before the migration is, and nothing the gix code does can move them.
const GIT2_BUNDLE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/git2-index.bundle"
);
const GIT2_SOURCES: [&str; 2] = [
    "90d0b64748a24be942ebee5d6ef4e4987a49b0df",
    "606d87ad05b3bd4eae6589ef12abd04848d65f27",
];
const GIT2_INDEXES: [&str; 2] = [
    "12279d7af101447ddff12297b341933f8eab74e6",
    "58cdbdd3b5b1db433f4106805625eeb09f5fdf47",
];

fn oid(hex: &str) -> Oid {
    Oid::from_hex(hex.as_bytes()).unwrap()
}

/// A repository holding the git2-written history and index.
fn git2_written() -> Fixture {
    let fixture = Fixture::new();
    support::git(
        fixture.path(),
        &[
            "fetch",
            "--quiet",
            GIT2_BUNDLE,
            "+refs/heads/main:refs/heads/source",
            "+refs/beholder/index:refs/beholder/index",
        ],
    );
    fixture
}

#[test]
fn an_index_written_through_git2_reads_back() {
    let fixture = git2_written();
    let store = Store::open(&fixture.repo);

    let history = store.history(10).unwrap();
    let ids: Vec<Oid> = history.iter().map(|e| e.id).collect();
    let sources: Vec<Oid> = history.iter().map(|e| e.source_commit).collect();
    assert_eq!(ids, vec![oid(GIT2_INDEXES[1]), oid(GIT2_INDEXES[0])]);
    assert_eq!(sources, vec![oid(GIT2_SOURCES[1]), oid(GIT2_SOURCES[0])]);

    for (index, source) in GIT2_INDEXES.iter().zip(GIT2_SOURCES) {
        let stored = store.read_analysis(oid(index)).unwrap();
        let manifest = beholder::walk::revision_manifest(&fixture.repo, source, &config()).unwrap();
        beholder::store::validate_payload(&stored, &manifest)
            .unwrap_or_else(|why| panic!("{index} no longer matches {source}: {why}"));
        assert!(!stored.files.is_empty());
    }
}

#[test]
fn an_index_written_through_git2_is_reused_file_by_file() {
    // Reuse is gated on the fingerprint, which carries the tool version, so the
    // stored fingerprint is the one asked for. What this proves is that every
    // phase 1 result git2 wrote is found and accepted by the gix reader.
    let fixture = git2_written();
    let store = Store::open(&fixture.repo);
    let tip = oid(GIT2_INDEXES[1]);
    let stored = store.read_analysis(tip).unwrap();

    let cache = store.phase1_cache(&stored.fingerprint).unwrap();
    let files = beholder::walk::read_revision(&fixture.repo, GIT2_SOURCES[1], &config()).unwrap();
    let (rerun, stats) = analysis::run_with_cache(&files, &config(), &cache);

    assert_eq!(stats.computed, 0, "every file should come from the store");
    assert_eq!(stats.reused, files.len());
    assert_eq!(rerun.files, stored.files);
}

#[test]
fn rewriting_an_index_written_through_git2_reproduces_its_ids() {
    // The tree is assembled entry by entry and the commit field by field. If
    // either serialized differently from libgit2, a reconciled or rewritten
    // index would get a new id, and two machines on either side of the
    // migration would stop converging on the same commits.
    let fixture = git2_written();
    let original = Store::open(&fixture.repo);
    let rewrite = Store::with_ref(&fixture.repo, "refs/beholder/rewrite");

    for (index, source) in GIT2_INDEXES.iter().zip(GIT2_SOURCES) {
        let stored = original.read_analysis(oid(index)).unwrap();
        let written = rewrite.write(oid(source), &stored).unwrap();
        assert_eq!(
            written,
            oid(index),
            "rewriting {source} changed its index id"
        );
    }
}

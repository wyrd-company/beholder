//! Running the two phases over a file set.
//!
//! Phase 1 is per file and depends only on that file's path and content. Phase
//! 2 is a function of the complete phase 1 output and is attributable to no
//! single file. The boundary is what makes phase 1 results storable and phase 2
//! results not reusable across differing file sets.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::phase1::{self, FileAnalysis};
use crate::walk::SourceFile;
use crate::{Symbol, SCHEMA_VERSION, TOOL_VERSION};

/// Everything that must match before a stored result may be reused.
///
/// Every component is a function of the tool and its configuration. None of it
/// is machine-local, so a fingerprint computed in CI matches one computed in a
/// fresh clone.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Fingerprint {
    pub schema_version: u32,
    pub tool_version: String,
    pub config: String,
    pub language_table: String,
    /// How paths in stored records are spelled.
    pub path_rules: String,
}

/// The only path spelling beholder stores or accepts.
pub const PATH_RULES: &str = "repo-relative-posix";

impl Fingerprint {
    pub fn new(config: &crate::Config) -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            tool_version: TOOL_VERSION.to_owned(),
            config: config.fingerprint(),
            language_table: crate::lang::table_fingerprint(),
            path_rules: PATH_RULES.to_owned(),
        }
    }
}

/// Does this path obey the stored-path rules?
pub fn is_repo_relative(path: &str) -> bool {
    !path.is_empty()
        && !path.starts_with('/')
        && !path.contains('\\')
        && !path
            .split('/')
            .any(|c| c == ".." || c == "." || c.is_empty())
}

/// Phase 2 — a function of the complete phase 1 output.
///
/// Reference resolution, fan-in/fan-out and cohesion components land here. What
/// exists today is the within-language percentile rank, which already has the
/// defining phase 2 property: adding or removing any file can change the value
/// for a symbol in a file that did not change.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Phase2 {
    /// Symbol id to percentile rank of its cognitive complexity within its own
    /// language, 0.0 to 100.0. Never compared across languages.
    pub complexity_percentile: BTreeMap<String, f32>,
}

/// A complete analysis of one file set.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Analysis {
    pub fingerprint: Fingerprint,
    /// Phase 1 output, stably sorted by path.
    pub files: Vec<FileAnalysis>,
    pub phase2: Phase2,
}

impl Analysis {
    /// Every symbol, in the order files were analyzed.
    pub fn symbols(&self) -> impl Iterator<Item = &Symbol> {
        self.files.iter().flat_map(|f| f.symbols.iter())
    }
}

/// Supplies previously computed phase 1 results.
///
/// A cache may only return a result whose content hash matches, which is what
/// keeps phase 1 a pure function of content even when it is not recomputed.
pub trait Phase1Cache {
    fn get(&self, path: &str, content_hash: &str) -> Option<FileAnalysis>;
}

/// A cache that never hits. Used when no store is available.
pub struct NoCache;

impl Phase1Cache for NoCache {
    fn get(&self, _path: &str, _content_hash: &str) -> Option<FileAnalysis> {
        None
    }
}

/// How much of phase 1 was reused rather than recomputed.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct CacheStats {
    pub reused: usize,
    pub computed: usize,
}

/// Run both phases over a file set.
pub fn run(files: &[SourceFile], config: &crate::Config) -> Analysis {
    run_with_cache(files, config, &NoCache).0
}

/// Run both phases, reusing stored phase 1 results where they are valid.
pub fn run_with_cache(
    files: &[SourceFile],
    config: &crate::Config,
    cache: &dyn Phase1Cache,
) -> (Analysis, CacheStats) {
    let mut analyses = Vec::with_capacity(files.len());
    let mut stats = CacheStats::default();

    for file in files {
        let content_hash = crate::hash::content_hash(file.contents.as_bytes());

        match cache.get(&file.path, &content_hash) {
            Some(stored) if stored.content_hash == content_hash && stored.path == file.path => {
                stats.reused += 1;
                analyses.push(stored);
            }
            _ => {
                stats.computed += 1;
                analyses.push(phase1::analyze(&file.path, &file.contents));
            }
        }
    }

    analyses.sort_by(|a, b| a.path.cmp(&b.path));
    let phase2 = phase2(&analyses);

    (
        Analysis {
            fingerprint: Fingerprint::new(config),
            files: analyses,
            phase2,
        },
        stats,
    )
}

/// Percentile rank of every symbol's complexity within its own language.
fn phase2(files: &[FileAnalysis]) -> Phase2 {
    let mut by_language: BTreeMap<&str, Vec<u32>> = BTreeMap::new();

    for symbol in files.iter().flat_map(|f| f.symbols.iter()) {
        by_language
            .entry(symbol.language.as_str())
            .or_default()
            .push(symbol.cognitive_complexity);
    }

    for scores in by_language.values_mut() {
        scores.sort_unstable();
    }

    let mut complexity_percentile = BTreeMap::new();

    for symbol in files.iter().flat_map(|f| f.symbols.iter()) {
        let scores = &by_language[symbol.language.as_str()];
        let at_or_below = scores.partition_point(|s| *s <= symbol.cognitive_complexity);
        let percentile = at_or_below as f32 * 100.0 / scores.len() as f32;
        complexity_percentile.insert(symbol.id.clone(), percentile);
    }

    Phase2 {
        complexity_percentile,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn source(path: &str, contents: &str) -> SourceFile {
        SourceFile {
            path: path.to_owned(),
            contents: contents.to_owned(),
        }
    }

    #[test]
    fn path_rules_reject_anything_but_repo_relative_posix() {
        assert!(is_repo_relative("crates/a/src/lib.rs"));
        assert!(!is_repo_relative("/abs/path.rs"));
        assert!(!is_repo_relative("a\\b.rs"));
        assert!(!is_repo_relative("../escape.rs"));
        assert!(!is_repo_relative("./here.rs"));
        assert!(!is_repo_relative(""));
    }

    #[test]
    fn fingerprint_changes_when_configuration_changes() {
        let a = Fingerprint::new(&crate::Config::default());
        let b = Fingerprint::new(&crate::Config::from_toml(r#"ignored_paths = ["x"]"#).unwrap());
        assert_ne!(a, b);
    }

    #[test]
    fn phase2_ranks_within_a_language() {
        let files = vec![
            source("a.rs", "fn simple() {}\n"),
            source(
                "b.rs",
                "fn hairy(a: bool, b: bool) {\n    if a {\n        if b {}\n    }\n}\n",
            ),
        ];

        let analysis = run(&files, &crate::Config::default());
        let simple = analysis.phase2.complexity_percentile["rust:a.rs:function:simple"];
        let hairy = analysis.phase2.complexity_percentile["rust:b.rs:function:hairy"];

        assert!(simple < hairy, "{simple} should rank below {hairy}");
        assert_eq!(hairy, 100.0);
    }

    #[test]
    fn phase2_depends_on_the_whole_set() {
        let hairy = source(
            "b.rs",
            "fn hairy(a: bool, b: bool) {\n    if a {\n        if b {}\n    }\n}\n",
        );

        let simple = source("a.rs", "fn simple() {}\n");
        let hairier = source(
            "c.rs",
            "fn hairier(a: bool, b: bool, c: bool) {\n    if a {\n        if b {\n            if c {}\n        }\n    }\n}\n",
        );

        let alone = run(&[simple.clone(), hairy.clone()], &crate::Config::default());
        let crowded = run(&[simple, hairy, hairier], &crate::Config::default());

        let id = "rust:b.rs:function:hairy";
        // The file did not change, but its rank did.
        assert_ne!(
            alone.phase2.complexity_percentile[id],
            crowded.phase2.complexity_percentile[id]
        );
    }

    #[test]
    fn tier_zero_files_are_analyzed_too() {
        let files = vec![source("config.yml", "a:\n  b: 1\n  c: 2\n  d: 3\n")];
        let analysis = run(&files, &crate::Config::default());

        assert_eq!(analysis.files.len(), 1);
        assert_eq!(analysis.files[0].tier, 0);
        assert!(analysis.files[0].density > 0.0);
    }

    #[test]
    fn a_matching_cache_entry_is_reused() {
        struct Always(FileAnalysis);
        impl Phase1Cache for Always {
            fn get(&self, path: &str, content_hash: &str) -> Option<FileAnalysis> {
                (self.0.path == path && self.0.content_hash == content_hash).then(|| self.0.clone())
            }
        }

        let file = source("a.rs", "fn simple() {}\n");
        let stored = phase1::analyze(&file.path, &file.contents);

        let (_, stats) = run_with_cache(&[file], &crate::Config::default(), &Always(stored));
        assert_eq!(
            stats,
            CacheStats {
                reused: 1,
                computed: 0
            }
        );
    }

    #[test]
    fn a_stale_cache_entry_is_ignored() {
        struct Stale(FileAnalysis);
        impl Phase1Cache for Stale {
            fn get(&self, _path: &str, _content_hash: &str) -> Option<FileAnalysis> {
                Some(self.0.clone())
            }
        }

        let stored = phase1::analyze("a.rs", "fn was_here() {}\n");
        let file = source("a.rs", "fn is_here_now() {}\n");

        let (analysis, stats) = run_with_cache(&[file], &crate::Config::default(), &Stale(stored));

        assert_eq!(
            stats,
            CacheStats {
                reused: 0,
                computed: 1
            }
        );
        assert_eq!(analysis.files[0].symbols[0].name, "is_here_now");
    }
}

//! Risk ranking, and the report structure every surface consumes.
//!
//! Ranking decides what surfaces and what stays quiet, which makes it a
//! calculation rather than a presentation concern. Every surface — SARIF, a
//! pull request comment, a local review — renders the same [`Report`].
//!
//! Two rules govern the arithmetic and neither is negotiable. Raw complexity is
//! never compared across languages, so scores are turned into percentile ranks
//! within a language before anything is merged. And fan-in is a measured
//! quantity with a known error profile, so how it is used follows the
//! measurement rather than intuition; see `docs/resolver-accuracy.md`.
//!
//! # What reading the output showed
//!
//! The ranking was run retroactively over fourteen merged changes across
//! `intentional`, `tagver` and `agent-svgtools` — every Rust-touching merged
//! pull request in the workspace, which is two, plus twelve pull-request-shaped
//! commits chosen for having enough content to judge.
//!
//! Twelve of the fourteen put a substantial piece of the change's actual
//! subject at the top. The two that did not are changes that moved almost no
//! complexity, and the reports said so rather than inventing something.
//!
//! [`FanInBasis::Damped`] is the recommended basis. Both bases agree on the top
//! five in five of fourteen reports, and where they disagree damping is right.
//! The clearest case: a three-line predicate, `DiscoveryConfig::is_empty`,
//! ranks second in its whole change under [`FanInBasis::Raw`] on the strength
//! of 27 references — none of them stated by the source, all of them
//! `Vec::is_empty` calls the resolver could not tell apart. Damping drops it
//! out of the top five and promotes real logic instead. Across all fourteen
//! reports, damping cuts such promotions from three to one.
//!
//! Two limitations are worth carrying forward. A change that hardens behaviour
//! without adding branches ranks at the bottom, because the complexity delta is
//! the multiplicand and it is near zero. And fan-in in this corpus is small —
//! mostly zero to eight — so complexity delta dominates the ordering and the
//! weighting only matters in the tail. That tail is where the collisions live,
//! which is why the basis matters, but the ranking would look broadly similar
//! with no fan-in at all.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::analysis::Analysis;
use crate::delta::{ChangeKind, Delta, SymbolChange};
use crate::graph::Degree;
use crate::resolve::Accuracy;

/// Which fan-in a ranking weights by.
///
/// Both are produced for every report. The production basis is not settled;
/// measurement rejected filtering to high confidence alone, and the two
/// candidates left are compared on real changes rather than on statistics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FanInBasis {
    /// Every resolved edge. The benchmark: it tracks the oracle's ordering
    /// best, and carries the collision noise that comes with that.
    Raw,
    /// Every edge, with the low-confidence contribution damped rather than
    /// discarded. Low-confidence fan-in is where the resolver's errors
    /// concentrate, but discarding it loses more signal than it removes noise.
    Damped,
}

impl FanInBasis {
    pub fn label(self) -> &'static str {
        match self {
            Self::Raw => "raw",
            Self::Damped => "damped",
        }
    }

    /// The fan-in this basis attributes to a symbol.
    ///
    /// Damping is smooth. A hard cap makes two symbols either side of it
    /// indistinguishable and moves the ordering discontinuously as code
    /// changes; `sqrt` keeps every low-confidence reference contributing
    /// something while stopping a hundred of them from dominating.
    pub fn fan_in(self, degree: &Degree) -> f64 {
        let high = degree.fan_in_high_confidence as f64;
        let low = degree.fan_in.saturating_sub(degree.fan_in_high_confidence) as f64;

        match self {
            Self::Raw => high + low,
            Self::Damped => high + low.sqrt(),
        }
    }
}

/// Why a symbol's fan-in was not used as an ordinary weight.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WeightCaveat {
    /// Fan-in on trait methods was measured as anti-correlated with the truth,
    /// so it is reported and not multiplied in. A dispatch beholder cannot see
    /// is worse than no information, because it points the wrong way.
    TraitMethodFanInUnreliable,
}

/// One changed symbol, ranked.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RankedChange {
    pub id: String,
    pub path: String,
    pub qualified_path: String,
    pub language: String,
    pub kind: String,
    pub change: ChangeKind,
    pub start_line: usize,
    pub end_line: usize,
    pub complexity_before: Option<u32>,
    pub complexity_after: Option<u32>,
    /// After minus before. Signed: a large simplification is worth seeing too.
    pub complexity_delta: i64,
    /// Every fan-in, so a consumer can audit the weighting rather than trust it.
    pub fan_in: usize,
    pub fan_in_high_confidence: usize,
    /// The weight this ranking actually applied.
    pub fan_in_weight: f64,
    pub caveat: Option<WeightCaveat>,
    /// Raw score. Never compared across languages.
    pub score: f64,
    /// Rank of `score` within this symbol's own language, 0.0 to 100.0. This is
    /// the only quantity that is comparable across languages.
    pub percentile: f64,
}

/// What every surface renders.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Report {
    pub before: String,
    pub after: String,
    /// Which fan-in the ranking weighted by.
    pub basis: FanInBasis,
    /// Ranked most risky first. Includes everything; a surface applies the
    /// threshold rather than the ranking hiding things from it.
    pub changes: Vec<RankedChange>,
    /// Percentile at or above which a surface should say something.
    pub threshold_percentile: f64,
    /// The resolver's error bar, per language.
    pub accuracy: BTreeMap<String, Accuracy>,
}

impl Report {
    /// Changes a surface should actually show.
    ///
    /// A percentile alone cannot decide this. Percentiles are relative to the
    /// other changes in the same diff, so in a change where nothing got more
    /// complex every symbol still ranks somewhere — and with scores tied at
    /// zero, everything ranks at the top. A score of zero means the change
    /// carries no complexity movement at all, and no amount of relative
    /// ranking makes that worth a reviewer's attention.
    pub fn surfaced(&self) -> impl Iterator<Item = &RankedChange> {
        self.changes
            .iter()
            .filter(|change| change.score > 0.0 && change.percentile >= self.threshold_percentile)
    }

    /// Is there anything worth saying? Silence is a valid report.
    pub fn is_quiet(&self) -> bool {
        self.surfaced().next().is_none()
    }
}

/// Default percentile at or above which a change is worth surfacing.
pub const DEFAULT_THRESHOLD: f64 = 80.0;

/// Rank a delta.
///
/// `after` supplies the graph, because risk is about the code as it now stands.
pub fn rank(
    before_revision: &str,
    after_revision: &str,
    delta: &Delta,
    after: &Analysis,
    basis: FanInBasis,
    threshold_percentile: f64,
) -> Report {
    let mut changes: Vec<RankedChange> = delta
        .changes
        .iter()
        .map(|change| score(change, after, basis))
        .collect();

    assign_percentiles(&mut changes);

    // Most risky first. Percentile is the merge key across languages; score
    // breaks ties only within one language, and the id keeps it deterministic.
    changes.sort_by(|a, b| {
        b.percentile
            .partial_cmp(&a.percentile)
            .expect("no NaN")
            .then(b.score.partial_cmp(&a.score).expect("no NaN"))
            .then(a.id.cmp(&b.id))
    });

    Report {
        before: before_revision.to_owned(),
        after: after_revision.to_owned(),
        basis,
        changes,
        threshold_percentile,
        accuracy: after.phase2.graph.accuracy.clone(),
    }
}

fn score(change: &SymbolChange, after: &Analysis, basis: FanInBasis) -> RankedChange {
    let side = change
        .after
        .as_ref()
        .or(change.before.as_ref())
        .expect("a side");
    let degree = after
        .phase2
        .graph
        .degree
        .get(&side.id)
        .cloned()
        .unwrap_or_default();

    let complexity_delta = change.complexity_delta.unwrap_or_else(|| {
        // An added or removed symbol is a change of its whole complexity.
        match (&change.before, &change.after) {
            (None, Some(after)) => i64::from(after.cognitive_complexity),
            (Some(before), None) => -i64::from(before.cognitive_complexity),
            _ => 0,
        }
    });

    let trait_method = side.qualified_path.contains(" as ");
    let caveat = trait_method.then_some(WeightCaveat::TraitMethodFanInUnreliable);

    // A symbol with no fan-in is still a change worth ranking, so the weight
    // starts at one rather than zeroing the score.
    let fan_in_weight = if trait_method {
        1.0
    } else {
        1.0 + basis.fan_in(&degree)
    };

    RankedChange {
        id: side.id.clone(),
        path: side.path.clone(),
        qualified_path: side.qualified_path.clone(),
        language: language_of(&side.id),
        kind: change.kind.clone(),
        change: change.change,
        start_line: side.start_line,
        end_line: side.end_line,
        complexity_before: change.before.as_ref().map(|m| m.cognitive_complexity),
        complexity_after: change.after.as_ref().map(|m| m.cognitive_complexity),
        complexity_delta,
        fan_in: degree.fan_in,
        fan_in_high_confidence: degree.fan_in_high_confidence,
        fan_in_weight,
        caveat,
        score: complexity_delta.abs() as f64 * fan_in_weight,
        percentile: 0.0,
    }
}

/// Percentile rank of each score within its own language.
///
/// Raw scores from different languages measure different things, so they are
/// never compared. Merging happens on the percentile and nothing else.
fn assign_percentiles(changes: &mut [RankedChange]) {
    let mut by_language: BTreeMap<String, Vec<f64>> = BTreeMap::new();

    for change in changes.iter() {
        by_language
            .entry(change.language.clone())
            .or_default()
            .push(change.score);
    }
    for scores in by_language.values_mut() {
        scores.sort_by(|a, b| a.partial_cmp(b).expect("no NaN"));
    }

    for change in changes.iter_mut() {
        let scores = &by_language[&change.language];
        // Midrank, so a run of equal scores shares one percentile instead of
        // every one of them being handed the top of the range.
        let below = scores.partition_point(|s| *s < change.score) as f64;
        let at_or_below = scores.partition_point(|s| *s <= change.score) as f64;
        change.percentile = (below + at_or_below) * 50.0 / scores.len() as f64;
    }
}

fn language_of(id: &str) -> String {
    id.split(':').next().unwrap_or_default().to_owned()
}

// ---------------------------------------------------------------------------
// Human rendering
// ---------------------------------------------------------------------------

/// Render a report for a person to read.
///
/// The surfaces in step 7 render the same [`Report`]; this exists so the
/// ranking can be judged before any of them are built.
pub fn render(report: &Report) -> String {
    use std::fmt::Write;

    let mut out = String::new();
    let surfaced: Vec<&RankedChange> = report.surfaced().collect();

    let _ = writeln!(
        out,
        "{}..{}  {} changed symbols, {} above the {:.0}th percentile  [{} fan-in]",
        short(&report.before),
        short(&report.after),
        report.changes.len(),
        surfaced.len(),
        report.threshold_percentile,
        report.basis.label(),
    );

    if report.changes.is_empty() {
        let _ = writeln!(out, "\nNo symbol changed.");
        return out;
    }
    if surfaced.is_empty() {
        let _ = writeln!(out, "\nNothing crossed the threshold.");
        return out;
    }

    let _ = writeln!(out);
    for (rank, change) in surfaced.iter().enumerate() {
        let delta = match change.complexity_delta {
            d if d > 0 => format!("+{d}"),
            d => d.to_string(),
        };
        let _ = writeln!(
            out,
            "{:>2}. {} {}:{}-{}",
            rank + 1,
            change.qualified_path,
            change.path,
            change.start_line,
            change.end_line
        );
        let _ = writeln!(
            out,
            "    {:?} · complexity {}{} · fan-in {} ({} stated){}",
            change.change,
            delta,
            match (change.complexity_before, change.complexity_after) {
                (Some(before), Some(after)) => format!(" ({before} -> {after})"),
                _ => String::new(),
            },
            change.fan_in,
            change.fan_in_high_confidence,
            match change.caveat {
                Some(WeightCaveat::TraitMethodFanInUnreliable) =>
                    " · trait method, fan-in not weighted",
                None => "",
            }
        );
    }

    for (language, accuracy) in &report.accuracy {
        if let (Some(precision), Some(recall)) = (accuracy.precision_range, accuracy.recall_range) {
            let _ = writeln!(
                out,
                "\nfan-in for {language} comes from a heuristic resolver measured at \
                 precision {:.2}-{:.2}, recall {:.2}-{:.2}",
                precision.0, precision.1, recall.0, recall.1
            );
        }
    }

    out
}

fn short(revision: &str) -> String {
    if revision.len() > 8 && revision.chars().all(|c| c.is_ascii_hexdigit()) {
        revision.chars().take(8).collect()
    } else {
        revision.to_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::delta;
    use crate::walk::SourceFile;

    fn analyze(files: &[(&str, &str)]) -> Analysis {
        let sources: Vec<_> = files
            .iter()
            .map(|(path, contents)| SourceFile {
                path: (*path).to_owned(),
                contents: (*contents).to_owned(),
            })
            .collect();
        crate::analysis::run(&sources, &crate::Config::default())
    }

    fn report(before: &[(&str, &str)], after: &[(&str, &str)], basis: FanInBasis) -> Report {
        let before_analysis = analyze(before);
        let after_analysis = analyze(after);
        let delta = delta::compare(&before_analysis, &after_analysis);
        rank(
            "before",
            "after",
            &delta,
            &after_analysis,
            basis,
            DEFAULT_THRESHOLD,
        )
    }

    #[test]
    fn an_unchanged_revision_is_quiet() {
        let files = [("src/lib.rs", "fn a(x: bool) {\n    if x {}\n}\n")];
        let report = report(&files, &files, FanInBasis::Raw);

        assert!(report.changes.is_empty());
        assert!(report.is_quiet());
    }

    #[test]
    fn depended_on_code_outranks_isolated_code_at_equal_complexity() {
        let before = [
            ("src/hot.rs", "pub fn hot(x: bool) {\n    if x {}\n}\n"),
            ("src/cold.rs", "pub fn cold(x: bool) {\n    if x {}\n}\n"),
            (
                "src/callers.rs",
                "use crate::hot::hot;\nfn a() {\n    hot(true);\n}\nfn b() {\n    hot(true);\n}\n",
            ),
        ];
        let after = [
            (
                "src/hot.rs",
                "pub fn hot(x: bool) {\n    if x {\n        if x {}\n    }\n}\n",
            ),
            (
                "src/cold.rs",
                "pub fn cold(x: bool) {\n    if x {\n        if x {}\n    }\n}\n",
            ),
            (
                "src/callers.rs",
                "use crate::hot::hot;\nfn a() {\n    hot(true);\n}\nfn b() {\n    hot(true);\n}\n",
            ),
        ];

        let report = report(&before, &after, FanInBasis::Raw);
        let ranked: Vec<&str> = report.changes.iter().map(|c| c.path.as_str()).collect();

        // Identical complexity delta; the one others depend on comes first.
        assert_eq!(ranked.first(), Some(&"src/hot.rs"), "{ranked:?}");
        assert!(report.changes[0].fan_in > 0);
    }

    #[test]
    fn a_trait_method_is_not_weighted_by_its_fan_in() {
        let source = |body: &str| {
            format!(
                "pub struct Ledger;\npub trait Tally {{ fn run(&self); }}\nimpl Tally for Ledger {{\n    fn run(&self) {{ {body} }}\n}}\n"
            )
        };

        let before = [("src/lib.rs", source("()").as_str())].map(|(p, _)| (p, source("()")));
        let after = [("src/lib.rs", "")].map(|(p, _)| (p, source("if true {}")));

        let before: Vec<(&str, &str)> = before.iter().map(|(p, s)| (*p, s.as_str())).collect();
        let after: Vec<(&str, &str)> = after.iter().map(|(p, s)| (*p, s.as_str())).collect();

        let report = report(&before, &after, FanInBasis::Raw);
        let ranked = report
            .changes
            .iter()
            .find(|c| c.qualified_path.contains(" as "))
            .expect("the trait method changed");

        assert_eq!(
            ranked.caveat,
            Some(WeightCaveat::TraitMethodFanInUnreliable)
        );
        assert_eq!(
            ranked.fan_in_weight, 1.0,
            "fan-in must not multiply a trait method's score"
        );
    }

    #[test]
    fn damping_reduces_the_pull_of_low_confidence_fan_in() {
        let degree = Degree {
            fan_in: 100,
            fan_out: 0,
            fan_in_high_confidence: 2,
            fan_out_high_confidence: 0,
        };

        assert_eq!(FanInBasis::Raw.fan_in(&degree), 100.0);
        // 2 stated references plus the square root of 98 guessed ones.
        assert!(FanInBasis::Damped.fan_in(&degree) < 15.0);
        assert!(FanInBasis::Damped.fan_in(&degree) > 2.0);
    }

    #[test]
    fn damping_is_smooth() {
        // Two symbols one reference apart must not become indistinguishable,
        // which is what a hard cap does either side of its limit.
        let at = |low: usize| {
            FanInBasis::Damped.fan_in(&Degree {
                fan_in: low,
                fan_out: 0,
                fan_in_high_confidence: 0,
                fan_out_high_confidence: 0,
            })
        };

        for low in [4, 5, 6, 20, 50] {
            assert!(at(low) < at(low + 1), "damping flattened at {low}");
        }
    }

    #[test]
    fn percentiles_merge_across_languages_but_scores_do_not() {
        let report = report(
            &[("src/a.rs", "fn a() {}\n")],
            &[(
                "src/a.rs",
                "fn a(x: bool) {\n    if x {\n        if x {}\n    }\n}\n",
            )],
            FanInBasis::Raw,
        );

        for change in &report.changes {
            assert!((0.0..=100.0).contains(&change.percentile));
        }
    }

    #[test]
    fn a_change_with_no_complexity_movement_stays_quiet() {
        // Renaming a local, reordering statements: a real edit, no new risk.
        let report = report(
            &[(
                "src/a.rs",
                "fn a() {\n    let first = 1;\n    let second = 2;\n}\n",
            )],
            &[(
                "src/a.rs",
                "fn a() {\n    let second = 2;\n    let first = 1;\n}\n",
            )],
            FanInBasis::Raw,
        );

        assert_eq!(report.changes.len(), 1, "the edit is still reported");
        assert!(report.is_quiet(), "but it does not surface");
    }

    #[test]
    fn tied_scores_share_a_percentile() {
        let report = report(
            &[("src/a.rs", "fn a() {}\nfn b() {}\nfn c() {}\n")],
            &[(
                "src/a.rs",
                "fn a(x: bool) {\n    if x {}\n}\nfn b(x: bool) {\n    if x {}\n}\nfn c(x: bool) {\n    if x {}\n}\n",
            )],
            FanInBasis::Raw,
        );

        let percentiles: Vec<f64> = report.changes.iter().map(|c| c.percentile).collect();
        assert!(
            percentiles.iter().all(|p| *p == percentiles[0]),
            "identical changes must rank identically: {percentiles:?}"
        );
        assert_eq!(percentiles[0], 50.0, "and at the middle, not the top");
    }

    #[test]
    fn output_is_deterministic() {
        let before = [("src/a.rs", "fn a() {}\nfn b() {}\n")];
        let after = [(
            "src/a.rs",
            "fn a(x: bool) {\n    if x {}\n}\nfn b(x: bool) {\n    if x {}\n}\n",
        )];

        assert_eq!(
            report(&before, &after, FanInBasis::Raw),
            report(&before, &after, FanInBasis::Raw)
        );
    }
}

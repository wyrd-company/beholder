//! SARIF 2.1.0 output.
//!
//! A surface, not a second ranking. Everything here is a rendering of
//! [`crate::risk::Report`]; what surfaces and in what order was decided before
//! this module saw it.
//!
//! Only changes above the report's threshold become results. SARIF drives
//! inline annotations on a pull request, and annotating every changed symbol
//! would bury the ones worth reading.

use serde::{Deserialize, Serialize};

use crate::risk::{RankedChange, Report, WeightCaveat};

const SCHEMA: &str = "https://json.schemastore.org/sarif-2.1.0.json";
const VERSION: &str = "2.1.0";

/// A SARIF log.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Log {
    #[serde(rename = "$schema")]
    pub schema: String,
    pub version: String,
    pub runs: Vec<Run>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Run {
    pub tool: Tool,
    pub results: Vec<SarifResult>,
    #[serde(rename = "columnKind")]
    pub column_kind: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Tool {
    pub driver: Driver,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Driver {
    pub name: String,
    pub version: String,
    #[serde(rename = "informationUri")]
    pub information_uri: String,
    pub rules: Vec<Rule>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Rule {
    pub id: String,
    pub name: String,
    #[serde(rename = "shortDescription")]
    pub short_description: Message,
    #[serde(rename = "fullDescription")]
    pub full_description: Message,
    #[serde(rename = "defaultConfiguration")]
    pub default_configuration: Configuration,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Configuration {
    pub level: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Message {
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SarifResult {
    #[serde(rename = "ruleId")]
    pub rule_id: String,
    pub level: String,
    pub message: Message,
    pub locations: Vec<Location>,
    #[serde(rename = "partialFingerprints")]
    pub partial_fingerprints: std::collections::BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Location {
    #[serde(rename = "physicalLocation")]
    pub physical_location: PhysicalLocation,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PhysicalLocation {
    #[serde(rename = "artifactLocation")]
    pub artifact_location: ArtifactLocation,
    pub region: Region,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ArtifactLocation {
    pub uri: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Region {
    #[serde(rename = "startLine")]
    pub start_line: usize,
    #[serde(rename = "endLine")]
    pub end_line: usize,
}

/// The one rule beholder reports under.
const RULE_ID: &str = "beholder/risky-change";

/// Render a report as SARIF.
pub fn render(report: &Report) -> Log {
    Log {
        schema: SCHEMA.to_owned(),
        version: VERSION.to_owned(),
        runs: vec![Run {
            tool: Tool {
                driver: Driver {
                    name: "beholder".to_owned(),
                    version: crate::TOOL_VERSION.to_owned(),
                    information_uri: "https://github.com/wyrd-company/beholder".to_owned(),
                    rules: vec![Rule {
                        id: RULE_ID.to_owned(),
                        name: "RiskyChange".to_owned(),
                        short_description: Message {
                            text: "A changed symbol ranked risky".to_owned(),
                        },
                        full_description: Message {
                            text: "Cognitive complexity moved in a symbol that other code \
                                   depends on. Ranked by complexity delta weighted by fan-in, \
                                   as a percentile within its own language."
                                .to_owned(),
                        },
                        default_configuration: Configuration {
                            level: "note".to_owned(),
                        },
                    }],
                },
            },
            results: report.surfaced().map(result).collect(),
            // Declared rather than left to a consumer's assumption. Every
            // region beholder emits is whole lines, so nothing here depends on
            // it yet; a column would.
            column_kind: "utf16CodeUnits".to_owned(),
        }],
    }
}

fn result(change: &RankedChange) -> SarifResult {
    let mut partial_fingerprints = std::collections::BTreeMap::new();
    // Symbol identity is already stable across line shifts, which is exactly
    // what a partial fingerprint is for: it keeps an annotation attached to the
    // same symbol as the file moves around it.
    partial_fingerprints.insert("beholderSymbol/v1".to_owned(), change.id.clone());

    SarifResult {
        rule_id: RULE_ID.to_owned(),
        level: level(change).to_owned(),
        message: Message {
            text: describe(change),
        },
        locations: vec![Location {
            physical_location: PhysicalLocation {
                artifact_location: ArtifactLocation {
                    uri: change.path.clone(),
                },
                region: Region {
                    start_line: change.start_line,
                    end_line: change.end_line,
                },
            },
        }],
        partial_fingerprints,
    }
}

/// Beholder reports; it does not gate. Nothing it emits is an error.
fn level(change: &RankedChange) -> &'static str {
    if change.percentile >= 95.0 {
        "warning"
    } else {
        "note"
    }
}

fn describe(change: &RankedChange) -> String {
    let delta = if change.complexity_delta > 0 {
        format!("+{}", change.complexity_delta)
    } else {
        change.complexity_delta.to_string()
    };

    let mut text = format!(
        "{} — complexity {delta}, fan-in {} ({} stated), {:.0}th percentile for {}",
        change.qualified_path,
        change.fan_in,
        change.fan_in_high_confidence,
        change.percentile,
        change.language,
    );

    if change.caveat == Some(WeightCaveat::TraitMethodFanInUnreliable) {
        text.push_str(". Fan-in on trait methods is not weighted: dispatch is not resolvable here");
    }

    text
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::risk::{rank, FanInBasis, DEFAULT_THRESHOLD};
    use crate::walk::SourceFile;

    fn report(before: &[(&str, &str)], after: &[(&str, &str)]) -> Report {
        let analyze = |files: &[(&str, &str)]| {
            let sources: Vec<_> = files
                .iter()
                .map(|(path, contents)| SourceFile {
                    path: (*path).to_owned(),
                    contents: (*contents).to_owned(),
                })
                .collect();
            crate::analysis::run(&sources, &crate::Config::default())
        };
        let before_analysis = analyze(before);
        let after_analysis = analyze(after);
        let delta = crate::delta::compare(&before_analysis, &after_analysis);
        rank(
            "before",
            "after",
            &delta,
            &before_analysis,
            &after_analysis,
            FanInBasis::Damped,
            DEFAULT_THRESHOLD,
        )
    }

    fn risky() -> Report {
        report(
            &[("src/a.rs", "fn a() {}\nfn b() {}\n")],
            &[(
                "src/a.rs",
                "fn a(x: bool) {\n    if x {\n        if x {\n            if x {}\n        }\n    }\n}\nfn b() {}\n",
            )],
        )
    }

    #[test]
    fn a_log_declares_the_schema_and_version() {
        let log = render(&risky());
        assert_eq!(log.version, "2.1.0");
        assert!(log.schema.contains("sarif-2.1.0"));
        assert_eq!(log.runs.len(), 1);
    }

    #[test]
    fn only_surfaced_changes_become_results() {
        // One deep change among five shallow identical ones: the threshold has
        // something to cut, and the crowd does not out-rank the outlier.
        let wide = report(
            &[("src/a.rs", "fn a() {}\nfn b() {}\nfn c() {}\nfn d() {}\nfn e() {}\nfn f() {}\n")],
            &[(
                "src/a.rs",
                "fn a(x: bool) {\n    if x {\n        if x {\n            if x {}\n        }\n    }\n}\nfn b(x: bool) {\n    if x {}\n}\nfn c(x: bool) {\n    if x {}\n}\nfn d(x: bool) {\n    if x {}\n}\nfn e(x: bool) {\n    if x {}\n}\nfn f(x: bool) {\n    if x {}\n}\n",
            )],
        );
        let log = render(&wide);

        assert_eq!(log.runs[0].results.len(), wide.surfaced().count());
        assert!(
            log.runs[0].results.len() < wide.changes.len(),
            "{} of {} surfaced",
            log.runs[0].results.len(),
            wide.changes.len()
        );
    }

    #[test]
    fn a_quiet_report_produces_no_results() {
        let files = [("src/a.rs", "fn a(x: bool) {\n    if x {}\n}\n")];
        let log = render(&report(&files, &files));

        assert!(log.runs[0].results.is_empty());
    }

    #[test]
    fn a_result_points_at_the_symbols_line_range() {
        let report = risky();
        let log = render(&report);
        let first = &log.runs[0].results[0];
        let surfaced = report.surfaced().next().unwrap();

        assert_eq!(
            first.locations[0].physical_location.artifact_location.uri,
            surfaced.path
        );
        assert_eq!(
            first.locations[0].physical_location.region.start_line,
            surfaced.start_line
        );
        assert!(crate::analysis::is_repo_relative(
            &first.locations[0].physical_location.artifact_location.uri
        ));
    }

    #[test]
    fn nothing_is_reported_as_an_error() {
        // Beholder reports and ranking decides what surfaces. It never gates.
        let log = render(&risky());
        for result in &log.runs[0].results {
            assert!(
                result.level == "note" || result.level == "warning",
                "{} is not a reporting level",
                result.level
            );
        }
    }

    #[test]
    fn a_result_fingerprints_the_symbol_not_the_line() {
        let log = render(&risky());
        let fingerprint = &log.runs[0].results[0].partial_fingerprints["beholderSymbol/v1"];

        assert!(fingerprint.starts_with("rust:"));
        assert!(!fingerprint.chars().any(|c| c.is_ascii_digit()));
    }

    #[test]
    fn output_is_deterministic() {
        assert_eq!(render(&risky()), render(&risky()));
    }
}

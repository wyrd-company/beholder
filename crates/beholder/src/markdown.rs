//! The sticky pull request comment.
//!
//! One comment per pull request, rewritten in place on every push. That is only
//! possible if the body is a pure function of the [`Report`]: an unchanged diff
//! has to render byte-identical text, or every push would look like an edit.
//! Nothing here reads the clock, the run number, or the surface it is posted to.
//!
//! Silence is a first-class outcome. A report with nothing above the threshold
//! renders [`None`], and the surface posts nothing at all.

use crate::risk::{RankedChange, Report, WeightCaveat};

/// Marks the comment beholder owns, so a surface can find its own comment again
/// without keeping state anywhere else.
pub const STICKY_MARKER: &str = "<!-- beholder:sticky:v1 -->";

/// Render the comment body, or `None` when nothing is worth saying.
pub fn render(report: &Report) -> Option<String> {
    if report.is_quiet() {
        return None;
    }
    Some(body(report))
}

/// Render the body a surface should leave behind when a report has gone quiet.
///
/// Only for a comment that already exists. Beholder never opens a conversation
/// to say it has nothing to say, but a comment left from an earlier push is
/// making a claim about code that has since changed, and letting that stand
/// would be worse than either posting or not posting.
pub fn render_quiet(report: &Report) -> String {
    format!(
        "{STICKY_MARKER}\n### beholder\n\nNo changed symbol crossed the {:.0}th percentile. \
         {} symbol{} changed.\n\n{}\n",
        report.threshold_percentile,
        report.changes.len(),
        plural(report.changes.len()),
        footer(report),
    )
}

fn body(report: &Report) -> String {
    use std::fmt::Write;

    let surfaced: Vec<&RankedChange> = report.surfaced().collect();
    let mut out = String::new();

    let _ = writeln!(out, "{STICKY_MARKER}");
    let _ = writeln!(out, "### beholder\n");
    let _ = writeln!(
        out,
        "{} of {} changed symbol{} ranked risky.\n",
        surfaced.len(),
        report.changes.len(),
        plural(report.changes.len()),
    );

    let _ = writeln!(out, "| # | symbol | change | complexity | fan-in |");
    let _ = writeln!(out, "| --: | --- | --- | --: | --: |");
    for (rank, change) in surfaced.iter().enumerate() {
        let _ = writeln!(
            out,
            "| {} | `{}`<br>`{}:{}-{}` | {} | {} | {} |",
            rank + 1,
            change.qualified_path,
            change.path,
            change.start_line,
            change.end_line,
            describe_change(change),
            complexity(change),
            fan_in(change),
        );
    }

    let _ = writeln!(out, "\n{}", footer(report));
    out
}

fn describe_change(change: &RankedChange) -> String {
    format!("{:?}", change.change).to_lowercase()
}

fn complexity(change: &RankedChange) -> String {
    let delta = if change.complexity_delta > 0 {
        format!("+{}", change.complexity_delta)
    } else {
        change.complexity_delta.to_string()
    };

    match (change.complexity_before, change.complexity_after) {
        (Some(before), Some(after)) => format!("{delta} ({before} → {after})"),
        _ => delta,
    }
}

fn fan_in(change: &RankedChange) -> String {
    match change.caveat {
        // Say why the number is there but not weighted, rather than showing a
        // weight that quietly ignored it.
        Some(WeightCaveat::TraitMethodFanInUnreliable) => {
            format!("{} (trait method, not weighted)", change.fan_in)
        }
        None => format!(
            "{} ({} stated)",
            change.fan_in, change.fan_in_high_confidence
        ),
    }
}

/// What the reader needs in order to distrust the table correctly.
fn footer(report: &Report) -> String {
    let mut lines = vec![format!(
        "Ranked by complexity delta weighted by fan-in ({} basis), as a percentile within each \
         language. Beholder reports; it does not gate.",
        report.basis.label(),
    )];

    for (language, accuracy) in &report.accuracy {
        match (accuracy.precision_range, accuracy.recall_range) {
            (Some(precision), Some(recall)) => lines.push(format!(
                "{language} fan-in comes from a heuristic resolver measured at precision \
                 {:.2}–{:.2}, recall {:.2}–{:.2}.",
                precision.0, precision.1, recall.0, recall.1
            )),
            _ => lines.push(format!(
                "{language} fan-in comes from a heuristic resolver whose accuracy is not measured."
            )),
        }
    }

    format!("<sub>{}</sub>", lines.join(" "))
}

fn plural(count: usize) -> &'static str {
    if count == 1 {
        ""
    } else {
        "s"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analysis::Analysis;
    use crate::delta;
    use crate::risk::{rank, FanInBasis, DEFAULT_THRESHOLD};
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

    fn report(before: &[(&str, &str)], after: &[(&str, &str)]) -> Report {
        let before_analysis = analyze(before);
        let after_analysis = analyze(after);
        let changed = delta::compare(&before_analysis, &after_analysis);
        rank(
            "0f1e2d3c4b5a6978",
            "6978a5b4c3d2e1f0",
            &changed,
            &before_analysis,
            &after_analysis,
            FanInBasis::Damped,
            DEFAULT_THRESHOLD,
        )
    }

    const SIMPLE: &str = "pub fn tally(items: &[u32]) -> u32 {\n    items.iter().sum()\n}\n";
    const COMPLEX: &str = "pub fn tally(items: &[u32]) -> u32 {\n    let mut total = 0;\n    \
                           for item in items {\n        if *item > 0 {\n            if *item < 10 \
                           {\n                total += item;\n            }\n        }\n    }\n    \
                           total\n}\n";

    #[test]
    fn a_quiet_report_renders_nothing() {
        let files = [("src/lib.rs", SIMPLE)];
        assert_eq!(render(&report(&files, &files)), None);
    }

    #[test]
    fn a_risky_change_renders_one_marked_comment() {
        let body = render(&report(
            &[("src/lib.rs", SIMPLE)],
            &[("src/lib.rs", COMPLEX)],
        ))
        .expect("a complexity increase is not quiet");

        assert!(body.starts_with(STICKY_MARKER), "{body}");
        assert_eq!(body.matches(STICKY_MARKER).count(), 1, "{body}");
        assert!(body.contains("tally"), "{body}");
        assert!(body.contains("src/lib.rs:1-"), "{body}");
        assert!(body.contains("does not gate"), "{body}");
    }

    /// The whole sticky mechanism rests on this: an unchanged diff must produce
    /// the identical body, or every re-run would rewrite the comment.
    #[test]
    fn the_same_report_renders_the_same_body() {
        let before = [("src/lib.rs", SIMPLE)];
        let after = [("src/lib.rs", COMPLEX)];

        assert_eq!(
            render(&report(&before, &after)),
            render(&report(&before, &after))
        );
    }

    #[test]
    fn a_quiet_body_is_still_marked_and_says_so() {
        let files = [("src/lib.rs", SIMPLE)];
        let body = render_quiet(&report(&files, &files));

        assert!(body.starts_with(STICKY_MARKER), "{body}");
        assert!(body.contains("No changed symbol crossed"), "{body}");
    }
}

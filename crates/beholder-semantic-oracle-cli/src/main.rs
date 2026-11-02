// ---
// relationships:
//   implements: semantic-oracle-model
// ---
//! `semantic-oracle`: authoring and validation commands for semantic-oracle
//! datasets.
//!
//! The commands read a project directory and its `semantic-oracle.yml`. All
//! behavior lives in `beholder-semantic-oracle`; this binary parses
//! arguments, reads files and writes output.
//!
//! Exit status: 0 when the command succeeds or the dataset passes, 1 when
//! the dataset has an error finding or a capture is not unique, 2 when the
//! command cannot run.

use std::{
    io::Write,
    path::{Path, PathBuf},
    process::ExitCode,
};

use anyhow::{bail, Context, Result};
use beholder_semantic_oracle::{
    audit_with_reports,
    locator::{capture, SourceText},
    parse_yaml,
    project::{inventory, GitProject, ProjectFiles, DATASET_FILE},
    validate, Finding, Report, Severity,
};
use clap::{Parser, Subcommand};
use serde_json::{json, Value};

#[derive(Parser)]
#[command(name = "semantic-oracle", version, about)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Print the locator of a span of source text as a YAML flow mapping.
    ///
    /// The text must occur exactly once where the options look; otherwise
    /// the command lists the positions where it occurs.
    Locate {
        /// The document's canonical path, relative to the project root.
        path: String,
        /// The exact source text of the span.
        text: String,
        /// Only occurrences that start on this line.
        #[arg(long)]
        line: Option<usize>,
        /// Only the occurrence that starts at this column.
        #[arg(long, requires = "line")]
        column: Option<usize>,
        /// The project root.
        #[arg(long, default_value = ".")]
        project: PathBuf,
    },
    /// Print the `documents` block for the project's tracked files, with
    /// current digests and the dataset's existing classifications.
    Inventory {
        /// The project root.
        #[arg(default_value = ".")]
        project: PathBuf,
    },
    /// Validate a dataset against the model, its profile schema and the
    /// project's files.
    Validate {
        /// The project root, or the dataset file in it.
        #[arg(default_value = ".")]
        project: PathBuf,
    },
    /// Validate a dataset, audit it against its profile, and compare its
    /// signatures with compiler reports. A slot no report states is listed as
    /// unchecked, and has not passed.
    Audit {
        /// The project root, or the dataset file in it.
        #[arg(default_value = ".")]
        project: PathBuf,
        /// A compiler report from a `semantic-oracle:report:<language>` task.
        /// Repeat it for each context.
        #[arg(long = "report")]
        reports: Vec<PathBuf>,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match run(cli.command) {
        Ok(code) => code,
        Err(cause) => {
            eprintln!("semantic-oracle: {cause:#}");
            ExitCode::from(2)
        }
    }
}

fn run(command: Command) -> Result<ExitCode> {
    let mut out = std::io::stdout().lock();
    match command {
        Command::Locate {
            path,
            text,
            line,
            column,
            project,
        } => {
            let file = project.join(&path);
            let bytes = std::fs::read(&file).with_context(|| format!("read {}", file.display()))?;
            let source = SourceText::decode(&bytes)
                .with_context(|| format!("{} is not UTF-8", file.display()))?;
            match capture(&source, &path, &text, line, column) {
                Ok(locator) => {
                    writeln!(out, "{}", locator.to_yaml())?;
                    Ok(ExitCode::SUCCESS)
                }
                Err(problem) => {
                    eprintln!("semantic-oracle: {path}: {problem}");
                    Ok(ExitCode::from(1))
                }
            }
        }
        Command::Inventory { project } => {
            let files = GitProject::open(&project)?;
            let dataset_file = files.root().join(DATASET_FILE);
            let dataset = match std::fs::read_to_string(&dataset_file) {
                Ok(text) => Some(
                    parse_yaml(&text)
                        .with_context(|| format!("parse {}", dataset_file.display()))?,
                ),
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
                Err(error) => {
                    return Err(error).with_context(|| format!("read {}", dataset_file.display()))
                }
            };
            write!(out, "{}", inventory(&files, dataset.as_ref())?)?;
            Ok(ExitCode::SUCCESS)
        }
        Command::Validate { project } => check(&mut out, &project, &validate),
        Command::Audit { project, reports } => {
            let reports = reports
                .iter()
                .map(|path| {
                    let text = std::fs::read_to_string(path)
                        .with_context(|| format!("read {}", path.display()))?;
                    serde_json::from_str(&text).with_context(|| format!("parse {}", path.display()))
                })
                .collect::<Result<Vec<Value>>>()?;
            let audit = |dataset: &Value, files: Option<&dyn ProjectFiles>| {
                audit_with_reports(dataset, files, &reports)
            };
            check(&mut out, &project, &audit)
        }
    }
}

type Check<'c> = &'c dyn Fn(&Value, Option<&dyn ProjectFiles>) -> Report;

/// Runs a check and writes its findings as JSON Lines, one finding per line
/// followed by one summary line.
fn check(out: &mut impl Write, target: &Path, check: Check<'_>) -> Result<ExitCode> {
    let (root, dataset_file) = locate_dataset(target)?;
    let text = std::fs::read_to_string(&dataset_file)
        .with_context(|| format!("read {}", dataset_file.display()))?;
    let report = match parse_yaml(&text) {
        Ok(dataset) => {
            let files = GitProject::open(&root)?;
            check(&dataset, Some(&files))
        }
        Err(cause) => Report {
            findings: vec![Finding {
                severity: Severity::Error,
                rule: "model:yaml".into(),
                at: String::new(),
                message: format!("{cause:#}"),
            }],
            ..Report::default()
        },
    };
    for finding in &report.findings {
        writeln!(out, "{}", finding.to_json())?;
    }
    for unchecked in &report.unchecked {
        writeln!(out, "{}", unchecked.to_json())?;
    }
    writeln!(out, "{}", json!({ "summary": report.summary() }))?;
    Ok(if report.passes() {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    })
}

/// The project root and dataset file for a target that is either.
fn locate_dataset(target: &Path) -> Result<(PathBuf, PathBuf)> {
    if target.is_dir() {
        return Ok((target.to_owned(), target.join(DATASET_FILE)));
    }
    if target.file_name().and_then(|name| name.to_str()) != Some(DATASET_FILE) {
        bail!(
            "{} is neither a project directory nor a {DATASET_FILE}",
            target.display()
        );
    }
    let root = match target.parent() {
        Some(parent) if !parent.as_os_str().is_empty() => parent.to_owned(),
        _ => PathBuf::from("."),
    };
    Ok((root, target.to_owned()))
}

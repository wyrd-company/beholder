//! Machine-first output.
//!
//! JSONL is the primary format. One record per line, stably sorted, with
//! repo-relative paths and nothing about the machine that produced it — so two
//! runs over the same source produce byte-identical files.

use std::io::Write;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::analysis::Analysis;

/// One line of `symbols.jsonl`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SymbolRecord {
    pub id: String,
    pub path: String,
    pub language: String,
    pub kind: String,
    pub name: String,
    pub qualified_path: String,
    pub start_line: usize,
    pub end_line: usize,
    pub cognitive_complexity: u32,
    /// Percentile rank within this symbol's own language. Never comparable
    /// across languages.
    pub complexity_percentile: f32,
    pub tier: u8,
    pub content_fingerprint: String,
}

/// One line of `files.jsonl`. Written for every file the walk visits, including
/// files no grammar claims.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FileRecord {
    pub path: String,
    pub language: Option<String>,
    pub tier: u8,
    pub density: f32,
    pub lines: usize,
    pub symbol_count: usize,
    pub content_hash: String,
}

/// Symbol records in output order: by path, then by position within the file.
pub fn symbol_records(analysis: &Analysis) -> Vec<SymbolRecord> {
    let mut records: Vec<SymbolRecord> = analysis
        .symbols()
        .map(|s| SymbolRecord {
            id: s.id.clone(),
            path: s.path.clone(),
            language: s.language.clone(),
            kind: s.kind.clone(),
            name: s.name.clone(),
            qualified_path: s.qualified_path.clone(),
            start_line: s.start_line,
            end_line: s.end_line,
            cognitive_complexity: s.cognitive_complexity,
            complexity_percentile: analysis
                .phase2
                .complexity_percentile
                .get(&s.id)
                .copied()
                .unwrap_or_default(),
            tier: s.tier,
            content_fingerprint: s.content_fingerprint.clone(),
        })
        .collect();

    records.sort_by(|a, b| {
        a.path
            .cmp(&b.path)
            .then(a.start_line.cmp(&b.start_line))
            .then(a.id.cmp(&b.id))
    });

    records
}

/// File records in output order: by path.
pub fn file_records(analysis: &Analysis) -> Vec<FileRecord> {
    let mut records: Vec<FileRecord> = analysis
        .files
        .iter()
        .map(|f| FileRecord {
            path: f.path.clone(),
            language: f.language.clone(),
            tier: f.tier,
            density: f.density,
            lines: f.lines,
            symbol_count: f.symbols.len(),
            content_hash: f.content_hash.clone(),
        })
        .collect();

    records.sort_by(|a, b| a.path.cmp(&b.path));
    records
}

/// Write records as JSONL.
pub fn write<T: Serialize>(out: &mut dyn Write, records: &[T]) -> Result<()> {
    for record in records {
        serde_json::to_writer(&mut *out, record).context("serializing a JSONL record")?;
        out.write_all(b"\n").context("writing a JSONL record")?;
    }
    Ok(())
}

/// Render records as a JSONL string.
pub fn to_string<T: Serialize>(records: &[T]) -> Result<String> {
    let mut buffer = Vec::new();
    write(&mut buffer, records)?;
    Ok(String::from_utf8(buffer).expect("JSON is always valid UTF-8"))
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
        crate::analysis::run(&sources, &crate::Config::default())
    }

    #[test]
    fn symbol_records_carry_the_documented_fields() {
        let analysis = analyze(&[("src/lib.rs", "fn draw(a: bool) {\n    if a {}\n}\n")]);
        let records = symbol_records(&analysis);

        assert_eq!(records.len(), 1);
        let record = &records[0];
        assert_eq!(record.path, "src/lib.rs");
        assert_eq!(record.kind, "function");
        assert_eq!(record.name, "draw");
        assert_eq!(record.qualified_path, "draw");
        assert_eq!((record.start_line, record.end_line), (1, 3));
        assert_eq!(record.cognitive_complexity, 1);
        assert_eq!(record.tier, 1);
    }

    #[test]
    fn every_file_gets_a_record_including_files_without_a_grammar() {
        let analysis = analyze(&[
            ("src/lib.rs", "fn draw() {}\n"),
            ("data/config.yml", "a:\n  b: 1\n  c: 2\n  d: 3\n"),
        ]);

        let paths: Vec<_> = file_records(&analysis)
            .into_iter()
            .map(|r| (r.path, r.tier))
            .collect();

        assert_eq!(
            paths,
            vec![
                ("data/config.yml".to_string(), 0),
                ("src/lib.rs".to_string(), 1)
            ]
        );
    }

    #[test]
    fn output_is_deterministic_and_stably_sorted() {
        let files = [
            ("z/last.rs", "fn z() {}\n"),
            ("a/first.rs", "fn b() {}\nfn a() {}\n"),
        ];

        let first = to_string(&symbol_records(&analyze(&files))).unwrap();
        let second = to_string(&symbol_records(&analyze(&files))).unwrap();

        assert_eq!(first, second);

        let order: Vec<_> = first
            .lines()
            .map(|l| serde_json::from_str::<SymbolRecord>(l).unwrap().id)
            .collect();
        assert_eq!(
            order,
            vec![
                "rust:a/first.rs:function:b",
                "rust:a/first.rs:function:a",
                "rust:z/last.rs:function:z",
            ]
        );
    }

    #[test]
    fn no_record_carries_an_absolute_path() {
        let analysis = analyze(&[("src/lib.rs", "fn draw() {}\n")]);
        for record in symbol_records(&analysis) {
            assert!(crate::analysis::is_repo_relative(&record.path));
            assert!(crate::analysis::is_repo_relative(
                record.id.split(':').nth(1).unwrap()
            ));
        }
    }
}

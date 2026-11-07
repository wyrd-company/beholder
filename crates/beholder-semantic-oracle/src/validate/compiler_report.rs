//! `profile:signature-compiler`: compares signatures with compiler reports.
//!
//! A report is a JSON file that a per-language script produces outside the
//! project tree. This crate runs no compiler. It reads:
//!
//! ```json
//! {
//!   "format": "semantic-oracle-compiler-report/1",
//!   "language": "rust",
//!   "context": "default",
//!   "tool": {"name": "rustdoc", "version": "1.99.1"},
//!   "covers": ["returns", "parameters[].type", "auto-traits[]"],
//!   "members": [
//!     {"path": "tally::make", "kind": "callable", "unit": {"package-version": "1.0.0"},
//!      "slots": {"returns": "@{tally::Counter}"}, "unstated": []}
//!   ]
//! }
//! ```
//!
//! `path` is the entity's canonical path and `kind`, where a language needs one, its
//! common kind, and `unit` any attributes that tell two units with one path apart. A report's `@{...}` holds a canonical
//! path, and a slot key is the model's slot path, with each key that names an
//! entity also written as `@{path}`: `bases[@{tally::Tally}].type`. `covers` lists the slot shapes,
//! with each bracket's content removed, that the report can state; `unstated`
//! lists a shape prefix or an exact slot path it could not state for one member,
//! such as `auto-traits` or `auto-traits[Unpin]`. A slot outside `covers` or in
//! `unstated` is unchecked and has not passed. A build that fails produces no
//! report. `failed` lists what a report could not read, as a warning. Where a report covers a slot and states another
//! value, or states one the record lacks, that is a disagreement to settle
//! against the language's rule.

use std::collections::{BTreeMap, BTreeSet};

use serde_json::Value;

use super::{list, text, Checker, Class, ContextCounts, Contexts, Unchecked};
use crate::{
    canonical::{
        canonical_auto_trait, canonical_key_types, canonicalize, canonicalize_bounds,
        named_id_spans, PathsAreIds,
    },
    signature::{slots, SlotClass},
};

const RULE: &str = "profile:signature-compiler";
const FORMAT: &str = "semantic-oracle-compiler-report/1";

/// A slot path with each bracket's content removed.
fn shape_of(path: &str) -> String {
    let mut out = String::with_capacity(path.len());
    let mut depth = 0usize;
    for c in path.chars() {
        match c {
            '[' => {
                if depth == 0 {
                    out.push('[');
                }
                depth += 1;
            }
            ']' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    out.push(']');
                }
            }
            _ if depth == 0 => out.push(c),
            _ => {}
        }
    }
    out
}

/// What the comparison found, for the report.
#[derive(Default)]
pub(super) struct Comparison {
    pub(super) unchecked: Vec<Unchecked>,
    pub(super) agreed: usize,
    pub(super) contexts: BTreeMap<String, ContextCounts>,
}

/// One slot in one context.
enum Outcome {
    Agreed,
    Unchecked(String),
    Disagrees(String),
}

struct Reported<'r> {
    covers: BTreeSet<&'r str>,
    tool: String,
    /// By path. One path can name several kinds, in Rust's namespaces, each with
    /// its own `kind`; a report in one namespace has none.
    members: BTreeMap<&'r str, Vec<&'r Value>>,
}

impl<'r> Reported<'r> {
    /// A report's member for an entity's path and kind.
    fn member(&self, path: &str, kind: &str, unit: Option<&Value>) -> Option<&'r Value> {
        let found = self.members.get(path)?;
        found
            .iter()
            .find(|member| {
                text(member, "kind").is_none_or(|theirs| theirs == kind) && same_unit(member, unit)
            })
            .copied()
    }
}

/// Whether a report's member can be in this unit: each attribute it states agrees.
/// Two units can hold one path, such as two versions of one crate.
fn same_unit(member: &Value, unit: Option<&Value>) -> bool {
    let Some(stated) = member.get("unit").and_then(Value::as_object) else {
        return true;
    };
    stated.iter().all(|(key, value)| {
        unit.and_then(|unit| unit.get(key))
            .is_none_or(|theirs| theirs == value)
    })
}

/// Rewrites each `@{id}` of a fact to `@{path}`, as a report writes names. Each
/// name the lexer reads outside a string literal is replaced once, in place, so
/// that no replacement is read again.
fn to_paths(checker: &Checker<'_>, text_value: &str) -> String {
    let mut out = String::with_capacity(text_value.len());
    let mut last = 0;
    for (span, id) in named_id_spans(checker.rules.language, text_value) {
        if let Some(path) = checker
            .ids
            .get(id.as_str())
            .and_then(|(_, r)| text(r, "path"))
        {
            out.push_str(&text_value[last..span.start]);
            out.push_str(&format!("@{{{path}}}"));
            last = span.end;
        }
    }
    out.push_str(&text_value[last..]);
    out
}

/// A slot path with each `bases` and `implementations` key in canonical spelling,
/// so that a key compares as its types do. The rest of the path stays as written.
fn canonical_key(language: &str, path: &str) -> String {
    let mut out = String::with_capacity(path.len());
    let mut rest = path;
    let mut at_segment = true;
    while let Some(c) = rest.chars().next() {
        let keyed = ["bases[", "implementations["]
            .into_iter()
            .find(|open| at_segment && rest.starts_with(open));
        if let Some(open) = keyed {
            let body = &rest[open.len()..];
            let end = closing_bracket(body).unwrap_or(body.len());
            out.push_str(open);
            out.push_str(&canonical_key_types(language, &body[..end], &PathsAreIds));
            out.push(']');
            rest = body.get(end + 1..).unwrap_or("");
            at_segment = false;
            continue;
        }
        out.push(c);
        at_segment = c == '.';
        rest = &rest[c.len_utf8()..];
    }
    out
}

/// The byte index of the `]` that closes a key, past nested brackets and string
/// literals.
fn closing_bracket(body: &str) -> Option<usize> {
    let mut depth = 0usize;
    let mut quote: Option<char> = None;
    let mut escaped = false;
    for (i, c) in body.char_indices() {
        match (quote, c) {
            (Some(_), _) if escaped => escaped = false,
            (Some(_), '\\') => escaped = true,
            (Some(open), _) if c == open => quote = None,
            (Some(_), _) => {}
            (None, '"' | '`') => quote = Some(c),
            (None, '[') => depth += 1,
            (None, ']') if depth == 0 => return Some(i),
            (None, ']') => depth -= 1,
            _ => {}
        }
    }
    None
}

/// Whether a report's release is the context's: equal, or the context's value
/// followed by more dotted components, as `go1.27` holds `go1.27.3`.
fn release_holds(declared: &str, reported: &str) -> bool {
    reported == declared
        || reported
            .strip_prefix(declared)
            .is_some_and(|rest| rest.starts_with('.'))
}

impl<'a> Checker<'a> {
    /// A report counts for a context only when it ran under the release the
    /// context's meaning depends on.
    fn release_matches(&self, report: &Value, context: &str) -> Result<(), String> {
        let parameter = self.rules.release_parameter;
        let declared = list(self.dataset, "contexts")
            .iter()
            .find(|record| text(record, "id") == Some(context))
            .and_then(|record| record.get("parameters"))
            .and_then(|parameters| text(parameters, parameter));
        let Some(declared) = declared else {
            return Err(format!(
                "context `{context}` has no `{parameter}`, so no report can be checked against it"
            ));
        };
        match text(report, "release") {
            None => Err(format!(
                "the report states no `release`; context `{context}` needs `{parameter}` `{declared}`"
            )),
            Some(reported) if release_holds(declared, reported) => Ok(()),
            Some(reported) => Err(format!(
                "the report ran under release `{reported}`, but context `{context}` has \
                 `{parameter}` `{declared}`; its slots are unchecked"
            )),
        }
    }

    /// A report that states its `mode` counts only for a context of that mode, such
    /// as a Go build report for a context with `mode: test`.
    fn mode_matches(&self, report: &Value, context: &str) -> Result<(), String> {
        let Some(reported) = text(report, "mode") else {
            return Ok(());
        };
        let declared = list(self.dataset, "contexts")
            .iter()
            .find(|record| text(record, "id") == Some(context))
            .and_then(|record| record.get("parameters"))
            .and_then(|parameters| text(parameters, "mode"));
        match declared {
            Some(declared) if declared != reported => Err(format!(
                "the report ran in mode `{reported}`, but context `{context}` has \
                 `mode` `{declared}`; its slots are unchecked"
            )),
            _ => Ok(()),
        }
    }

    fn reports_by_context<'r>(&mut self, reports: &'r [Value]) -> BTreeMap<&'r str, Reported<'r>> {
        let mut by_context = BTreeMap::new();
        for (i, report) in reports.iter().enumerate() {
            let at = format!("/reports/{i}");
            if text(report, "format") != Some(FORMAT) {
                self.error(RULE, at, format!("a compiler report has format `{FORMAT}`"));
                continue;
            }
            let language = text(report, "language").unwrap_or_default();
            if language != self.rules.language {
                let message = format!(
                    "this report is for `{language}`; the dataset's profile is `{}`",
                    self.rules.language
                );
                self.error(RULE, at, message);
                continue;
            }
            let Some(context) = text(report, "context").filter(|c| self.all.contains(c)) else {
                let message = format!(
                    "the report's context `{}` is not declared",
                    text(report, "context").unwrap_or_default()
                );
                self.error(RULE, at, message);
                continue;
            };
            if let Err(message) = self.release_matches(report, context) {
                self.error(RULE, format!("{at}/release"), message);
                continue;
            }
            if let Err(message) = self.mode_matches(report, context) {
                self.error(RULE, format!("{at}/mode"), message);
                continue;
            }
            for failed in list(report, "failed").iter().filter_map(Value::as_str) {
                self.warning(
                    RULE,
                    format!("/reports/{i}"),
                    format!("the report could not read `{failed}`; its slots are unchecked"),
                );
            }
            let mut members: BTreeMap<&str, Vec<&Value>> = BTreeMap::new();
            for member in list(report, "members") {
                if let Some(path) = text(member, "path") {
                    members.entry(path).or_default().push(member);
                }
            }
            let covers = list(report, "covers")
                .iter()
                .filter_map(Value::as_str)
                .collect();
            let tool = report
                .get("tool")
                .map(|tool| {
                    format!(
                        "{} {}",
                        text(tool, "name").unwrap_or_default(),
                        text(tool, "version").unwrap_or_default()
                    )
                })
                .unwrap_or_default();
            by_context.insert(
                context,
                Reported {
                    covers,
                    tool,
                    members,
                },
            );
        }
        by_context
    }

    /// Compares every signature slot with the reports, one context at a time.
    pub(super) fn compare_with_reports(&mut self, reports: &[Value]) -> Comparison {
        let by_context = self.reports_by_context(reports);
        let mut comparison = Comparison::default();
        let records = self
            .dataset
            .get("public-api")
            .map(|public_api| list(public_api, "signatures"))
            .unwrap_or(&[]);
        for (index, record) in records.iter().enumerate() {
            let Some(id) = text(record, "entity") else {
                continue;
            };
            let Some((Class::Entity, entity)) = self.ids.get(id).copied() else {
                continue;
            };
            let path = text(entity, "path").unwrap_or_default();
            let kind = text(entity, "kind").unwrap_or_default();
            let unit = self.unit_of_entity(entity);
            let contexts = self
                .declared(record)
                .unwrap_or_else(|| self.built_contexts(id));
            let at = format!("/public-api/signatures/{index}");
            for slot in slots(record) {
                let (mut agreed_in, mut unchecked_in) = (Vec::new(), Vec::new());
                let (mut reasons, mut disagreements) = (Vec::new(), Vec::new());
                for context in &contexts {
                    let counts = comparison
                        .contexts
                        .entry((*context).to_owned())
                        .or_default();
                    match self.compare_in_context(&slot, (path, kind, unit), context, &by_context) {
                        Outcome::Agreed => {
                            counts.agreed += 1;
                            agreed_in.push((*context).to_owned());
                        }
                        Outcome::Unchecked(reason) => {
                            counts.unchecked += 1;
                            unchecked_in.push((*context).to_owned());
                            if !reasons.contains(&reason) {
                                reasons.push(reason);
                            }
                        }
                        Outcome::Disagrees(message) => {
                            counts.errors += 1;
                            disagreements.push(message);
                        }
                    }
                }
                let pointer = format!("{at}{}", slot.pointer);
                if !disagreements.is_empty() {
                    for message in disagreements {
                        self.error(RULE, pointer.clone(), message);
                    }
                } else if unchecked_in.is_empty() {
                    comparison.agreed += 1;
                } else {
                    comparison.unchecked.push(Unchecked {
                        at: pointer,
                        slot: slot.path.clone(),
                        class: slot.class.as_str(),
                        reason: reasons.join("; "),
                        agreed_in,
                        unchecked_in,
                    });
                }
            }
            self.check_reported_slots_exist(
                record,
                (path, kind, unit),
                &contexts,
                &at,
                &by_context,
            );
        }
        self.check_reported_members(&by_context);
        comparison
    }

    fn compare_in_context(
        &self,
        slot: &crate::signature::Slot<'_>,
        (path, kind, unit): (&str, &str, Option<&Value>),
        context: &str,
        by_context: &BTreeMap<&str, Reported<'_>>,
    ) -> Outcome {
        if slot.alternative.is_some() {
            return Outcome::Unchecked(
                "a report states one definition, not its alternatives".to_owned(),
            );
        }
        let Some(report) = by_context.get(context) else {
            return Outcome::Unchecked(format!("no compiler report for context `{context}`"));
        };
        let Some(member) = report.member(path, kind, unit) else {
            return Outcome::Unchecked(format!("{} does not state `{path}`", report.tool));
        };
        let shape = shape_of(&slot.path);
        // A report keys a slot by paths, as it writes every name, and a key's types
        // compare in canonical spelling.
        let language = self.rules.language;
        let key = canonical_key(language, &to_paths(self, &slot.path));
        let unstated = list(member, "unstated")
            .iter()
            .filter_map(Value::as_str)
            .any(|unstated| {
                shape.starts_with(unstated) || key == canonical_key(language, unstated)
            });
        if unstated || !report.covers.contains(shape.as_str()) {
            return Outcome::Unchecked(format!("{} cannot state this slot", report.tool));
        }
        let reported = member
            .get("slots")
            .and_then(Value::as_object)
            .and_then(|slots| {
                slots
                    .iter()
                    .find(|(stated, _)| canonical_key(language, stated) == key)
            })
            .and_then(|(_, value)| value.as_str());
        let Some(reported) = reported else {
            return Outcome::Disagrees(format!(
                "`{}` is `{}`; {} states no such slot `{key}` in `{context}`",
                slot.path, slot.text, report.tool
            ));
        };
        let (fact, seen) = self.comparable(slot, reported);
        if fact == seen {
            Outcome::Agreed
        } else {
            Outcome::Disagrees(format!(
                "`{}` is `{fact}`; {} states `{seen}` in `{context}`",
                slot.path, report.tool
            ))
        }
    }

    /// A fact and a reported value, in one spelling with paths for names.
    fn comparable(&self, slot: &crate::signature::Slot<'_>, reported: &str) -> (String, String) {
        let fact = to_paths(self, &slot.text);
        match slot.class {
            SlotClass::DeclaredType | SlotClass::InferredType => {
                let spell = if slot.path.ends_with("bounds") {
                    canonicalize_bounds
                } else {
                    canonicalize
                };
                (
                    spell(self.rules.language, &fact, &PathsAreIds).text,
                    spell(self.rules.language, reported, &PathsAreIds).text,
                )
            }
            SlotClass::AutoTrait => (
                canonical_auto_trait(&fact, &PathsAreIds),
                canonical_auto_trait(reported, &PathsAreIds),
            ),
            _ => (fact, reported.to_owned()),
        }
    }

    /// The unit record of an entity with one unit.
    fn unit_of_entity(&self, entity: &'a Value) -> Option<&'a Value> {
        text(entity, "unit")
            .and_then(|unit| self.ids.get(unit))
            .map(|(_, record)| *record)
    }

    /// A slot a report states that its record lacks.
    fn check_reported_slots_exist(
        &mut self,
        record: &'a Value,
        (path, kind, unit): (&str, &str, Option<&Value>),
        contexts: &Contexts<'a>,
        at: &str,
        by_context: &BTreeMap<&str, Reported<'_>>,
    ) {
        let written: BTreeSet<String> = slots(record)
            .into_iter()
            .map(|slot| canonical_key(self.rules.language, &to_paths(self, &slot.path)))
            .collect();
        let alternatives = record.get("alternatives").is_some();
        for context in contexts {
            let Some(report) = by_context.get(context) else {
                continue;
            };
            let Some(member) = report.member(path, kind, unit) else {
                continue;
            };
            let Some(stated) = member.get("slots").and_then(Value::as_object) else {
                continue;
            };
            let unstated: Vec<&str> = list(member, "unstated")
                .iter()
                .filter_map(Value::as_str)
                .collect();
            for (slot, value) in stated {
                let key = canonical_key(self.rules.language, slot);
                if alternatives
                    || written.contains(&key)
                    || unstated.iter().any(|prefix| {
                        slot.starts_with(prefix)
                            || key == canonical_key(self.rules.language, prefix)
                    })
                {
                    continue;
                }
                let value = value.as_str().unwrap_or_default();
                let tool = &report.tool;
                self.error(
                    RULE,
                    at.to_owned(),
                    format!(
                        "{tool} states `{slot}` as `{value}` in `{context}`; the record lacks it"
                    ),
                );
            }
        }
    }

    /// A member with no record that a report states slots for.
    fn check_reported_members(&mut self, by_context: &BTreeMap<&str, Reported<'_>>) {
        let recorded: BTreeSet<&str> = self
            .dataset
            .get("public-api")
            .map(|public_api| list(public_api, "signatures"))
            .unwrap_or(&[])
            .iter()
            .filter_map(|record| text(record, "entity"))
            .collect();
        let mut problems = Vec::new();
        for member in self.public_api_members() {
            let Some(id) = text(member, "entity") else {
                continue;
            };
            let Some((Class::Entity, entity)) = self.ids.get(id).copied() else {
                continue;
            };
            if recorded.contains(id) {
                continue;
            }
            let path = text(entity, "path").unwrap_or_default();
            let kind = text(entity, "kind").unwrap_or_default();
            let unit = self.unit_of_entity(entity);
            for (context, report) in by_context {
                let stated = report
                    .member(path, kind, unit)
                    .and_then(|member| member.get("slots"))
                    .and_then(Value::as_object);
                if let Some(first) = stated.and_then(|slots| slots.keys().next()) {
                    let tool = &report.tool;
                    problems.push(format!(
                        "{tool} states `{first}` for `{id}` in `{context}`; no record states it"
                    ));
                }
            }
        }
        problems.sort();
        problems.dedup();
        for message in problems {
            self.error(RULE, "/public-api/signatures".into(), message);
        }
    }
}

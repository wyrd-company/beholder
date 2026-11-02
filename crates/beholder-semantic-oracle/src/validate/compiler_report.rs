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
//!     {"path": "tally::make", "slots": {"returns": "@{tally::Counter}"}, "unstated": []}
//!   ]
//! }
//! ```
//!
//! `path` is the entity's canonical path, a report's `@{...}` holds a canonical
//! path, and a slot key is the model's slot path. `covers` lists the slot shapes,
//! with each bracket's content removed, that the report can state; `unstated`
//! lists a shape prefix or an exact slot path it could not state for one member,
//! such as `auto-traits` or `auto-traits[Unpin]`. A slot outside `covers` or in
//! `unstated` is unchecked and has not passed. A build that fails produces no
//! report. `failed` lists what a report could not read, as a warning. Where a report covers a slot and states another
//! value, or states one the record lacks, that is a disagreement to settle
//! against the language's rule.

use std::collections::{BTreeMap, BTreeSet};

use serde_json::Value;

use super::{list, text, Checker, Class, Contexts, Unchecked};
use crate::{
    canonical::{canonical_auto_trait, canonicalize, canonicalize_bounds, PathsAreIds},
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

struct Reported<'r> {
    covers: BTreeSet<&'r str>,
    tool: String,
    members: BTreeMap<&'r str, &'r Value>,
}

/// Rewrites each `@{id}` of a fact to `@{path}`, as a report writes names.
fn to_paths(checker: &Checker<'_>, text_value: &str) -> String {
    let mut out = text_value.to_owned();
    for id in crate::canonical::named_ids(checker.rules.language, text_value) {
        if let Some(path) = checker
            .ids
            .get(id.as_str())
            .and_then(|(_, r)| text(r, "path"))
        {
            out = out.replace(&format!("@{{{id}}}"), &format!("@{{{path}}}"));
        }
    }
    out
}

impl<'a> Checker<'a> {
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
            for failed in list(report, "failed").iter().filter_map(Value::as_str) {
                self.warning(
                    RULE,
                    format!("/reports/{i}"),
                    format!("the report could not read `{failed}`; its slots are unchecked"),
                );
            }
            let members = list(report, "members")
                .iter()
                .filter_map(|member| Some((text(member, "path")?, member)))
                .collect();
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

    /// Compares every signature slot with the reports: `(unchecked, agreed)`.
    pub(super) fn compare_with_reports(&mut self, reports: &[Value]) -> (Vec<Unchecked>, usize) {
        let by_context = self.reports_by_context(reports);
        let mut unchecked = Vec::new();
        let mut agreed = 0;
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
            let contexts = self
                .declared(record)
                .unwrap_or_else(|| self.built_contexts(id));
            let at = format!("/public-api/signatures/{index}");
            for slot in slots(record) {
                let (reasons, disagreements) =
                    self.compare_slot(&slot, path, &contexts, &by_context);
                let pointer = format!("{at}{}", slot.pointer);
                if !disagreements.is_empty() {
                    for message in disagreements {
                        self.error(RULE, pointer.clone(), message);
                    }
                } else if reasons.is_empty() {
                    agreed += 1;
                } else {
                    unchecked.push(Unchecked {
                        at: pointer,
                        slot: slot.path.clone(),
                        class: slot.class.as_str(),
                        reason: reasons.join("; "),
                    });
                }
            }
            self.check_reported_slots_exist(record, path, &contexts, &at, &by_context);
        }
        self.check_reported_members(&by_context);
        (unchecked, agreed)
    }

    fn compare_slot(
        &self,
        slot: &crate::signature::Slot<'_>,
        path: &str,
        contexts: &Contexts<'a>,
        by_context: &BTreeMap<&str, Reported<'_>>,
    ) -> (Vec<String>, Vec<String>) {
        let mut reasons = Vec::new();
        let mut disagreements = Vec::new();
        if slot.alternative.is_some() {
            reasons.push("a report states one definition, not its alternatives".to_owned());
            return (reasons, disagreements);
        }
        let shape = shape_of(&slot.path);
        for context in contexts {
            let Some(report) = by_context.get(context) else {
                reasons.push(format!("no compiler report for context `{context}`"));
                continue;
            };
            let Some(member) = report.members.get(path) else {
                reasons.push(format!("{} does not state `{path}`", report.tool));
                continue;
            };
            let unstated = list(member, "unstated")
                .iter()
                .filter_map(Value::as_str)
                .any(|unstated| shape.starts_with(unstated) || slot.path == unstated);
            if unstated || !report.covers.contains(shape.as_str()) {
                reasons.push(format!("{} cannot state this slot", report.tool));
                continue;
            }
            let reported = member
                .get("slots")
                .and_then(|slots| slots.get(&slot.path))
                .and_then(Value::as_str);
            let Some(reported) = reported else {
                disagreements.push(format!(
                    "`{}` is `{}`; {} states no such slot in `{context}`",
                    slot.path, slot.text, report.tool
                ));
                continue;
            };
            let (fact, seen) = self.comparable(slot, reported);
            if fact != seen {
                disagreements.push(format!(
                    "`{}` is `{fact}`; {} states `{seen}` in `{context}`",
                    slot.path, report.tool
                ));
            }
        }
        (reasons, disagreements)
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

    /// A slot a report states that its record lacks.
    fn check_reported_slots_exist(
        &mut self,
        record: &'a Value,
        path: &str,
        contexts: &Contexts<'a>,
        at: &str,
        by_context: &BTreeMap<&str, Reported<'_>>,
    ) {
        let written: BTreeSet<String> = slots(record).into_iter().map(|slot| slot.path).collect();
        let alternatives = record.get("alternatives").is_some();
        for context in contexts {
            let Some(report) = by_context.get(context) else {
                continue;
            };
            let Some(member) = report.members.get(path) else {
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
                if alternatives
                    || written.contains(slot)
                    || unstated.iter().any(|prefix| slot.starts_with(prefix))
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
            for (context, report) in by_context {
                let stated = report
                    .members
                    .get(path)
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

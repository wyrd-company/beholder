//! Dataset validation and the profile audit.
//!
//! Validation applies the model's validation criteria 1 to 6 and the
//! model-level part of criterion 7: the dataset schema and versions, id
//! uniqueness and resolution, the document inventory and digests, locators,
//! fact contexts against unit and document membership, dispositions,
//! extension facts, and the consistency of public API entries with
//! effective visibility. The audit adds criterion 9 as far as the profile
//! rules are mechanical: document file classes, out-of-profile facts, and
//! profile-specific value rules, with coverage notes for indexed documents
//! that hold no fact in a context.
//!
//! Every finding names the rule it enforces: `model:<rule>` for a rule of
//! `semantic-oracle-model`, `<language>:<rule>` for a rule of a profile, and
//! `schema:<keyword>` for the dataset schema.

use std::{
    cmp::Ordering,
    collections::{BTreeMap, BTreeSet},
    fmt,
};

use serde_json::{json, Value};

mod public_api;

use crate::{
    dataset_validator,
    locator::{Locator, LocatorError, Position, SourceText},
    project::{sha256_hex, ProjectFiles, DATASET_FILE},
    rules::{Names, ProfileRules},
    Profile, MODEL_VERSION,
};

/// Whether a finding fails the dataset.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    Error,
    Warning,
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Error => "error",
            Self::Warning => "warning",
        })
    }
}

/// One finding: the rule, the JSON Pointer of the dataset location, and a
/// one-line message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finding {
    pub severity: Severity,
    pub rule: String,
    pub at: String,
    pub message: String,
}

impl Finding {
    pub fn to_json(&self) -> Value {
        json!({
            "severity": self.severity.to_string(),
            "rule": self.rule,
            "at": self.at,
            "message": self.message,
        })
    }
}

/// The findings of one check, in a deterministic order: by dataset
/// location, then rule, then message.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Report {
    pub findings: Vec<Finding>,
}

impl Report {
    fn new(mut findings: Vec<Finding>) -> Self {
        findings.sort_by(|a, b| {
            compare_pointers(&a.at, &b.at)
                .then_with(|| a.rule.cmp(&b.rule))
                .then_with(|| a.message.cmp(&b.message))
                .then_with(|| a.severity.cmp(&b.severity))
        });
        findings.dedup();
        Self { findings }
    }

    pub fn errors(&self) -> usize {
        self.count(Severity::Error)
    }

    pub fn warnings(&self) -> usize {
        self.count(Severity::Warning)
    }

    fn count(&self, severity: Severity) -> usize {
        self.findings
            .iter()
            .filter(|finding| finding.severity == severity)
            .count()
    }

    /// Whether the dataset passes: no finding is an error.
    pub fn passes(&self) -> bool {
        self.errors() == 0
    }

    /// The summary record: error and warning counts.
    pub fn summary(&self) -> Value {
        json!({ "errors": self.errors(), "warnings": self.warnings() })
    }
}

/// Compares JSON Pointers segment by segment, numerically where both
/// segments are array indexes.
fn compare_pointers(a: &str, b: &str) -> Ordering {
    let mut left = a.split('/');
    let mut right = b.split('/');
    loop {
        match (left.next(), right.next()) {
            (None, None) => return Ordering::Equal,
            (None, Some(_)) => return Ordering::Less,
            (Some(_), None) => return Ordering::Greater,
            (Some(x), Some(y)) => {
                let order = match (x.parse::<u64>(), y.parse::<u64>()) {
                    (Ok(x), Ok(y)) => x.cmp(&y),
                    _ => x.cmp(y),
                };
                if order != Ordering::Equal {
                    return order;
                }
            }
        }
    }
}

/// Validates a dataset. With `files`, the inventory, digests and locator
/// text are checked against the project; without, those checks are
/// skipped.
pub fn validate(dataset: &Value, files: Option<&dyn ProjectFiles>) -> Report {
    run(dataset, files, false)
}

/// Validates a dataset and audits it against its profile.
pub fn audit(dataset: &Value, files: Option<&dyn ProjectFiles>) -> Report {
    run(dataset, files, true)
}

fn run(dataset: &Value, files: Option<&dyn ProjectFiles>, audit: bool) -> Report {
    let mut findings = Vec::new();
    let rules = versions(dataset, &mut findings);
    if !findings.is_empty() {
        return Report::new(findings);
    }
    schema(dataset, &mut findings);
    let Some(rules) = rules.filter(|_| findings.is_empty()) else {
        if findings.is_empty() {
            // The schema requires a known profile, so this is unreachable
            // unless the schema and the profile list disagree.
            findings.push(error(
                "model:profile-version",
                "/profile".into(),
                "the dataset names no profile this validator defines".into(),
            ));
        }
        return Report::new(findings);
    };
    let mut checker = Checker::new(dataset, rules, files);
    checker.validate();
    if audit {
        checker.audit();
    }
    findings.extend(checker.finish());
    Report::new(findings)
}

/// Checks the model and profile versions first, so that a dataset of
/// another version gets one finding instead of every schema error. A
/// missing or malformed version is left to the schema.
fn versions(dataset: &Value, findings: &mut Vec<Finding>) -> Option<&'static ProfileRules> {
    if let Some(model) = dataset.get("model").and_then(Value::as_str) {
        if model != MODEL_VERSION {
            findings.push(error(
                "model:version",
                "/model".into(),
                format!("model version `{model}` is not `{MODEL_VERSION}`"),
            ));
            return None;
        }
    }
    let version = dataset.get("profile").and_then(Value::as_str)?;
    match Profile::by_version(version) {
        Some(profile) => ProfileRules::for_language(profile.language),
        None => {
            findings.push(error(
                "model:profile-version",
                "/profile".into(),
                format!("model version 1 defines no profile `{version}`"),
            ));
            None
        }
    }
}

fn schema(dataset: &Value, findings: &mut Vec<Finding>) {
    let validator = match dataset_validator() {
        Ok(validator) => validator,
        Err(cause) => {
            findings.push(error("schema", String::new(), format!("{cause:#}")));
            return;
        }
    };
    for problem in validator.iter_errors(dataset) {
        findings.push(error(
            &format!("schema:{}", problem.kind().keyword()),
            problem.instance_path().to_string(),
            problem.to_string(),
        ));
    }
}

fn error(rule: &str, at: String, message: String) -> Finding {
    Finding {
        severity: Severity::Error,
        rule: rule.into(),
        at,
        message,
    }
}

fn warning(rule: &str, at: String, message: String) -> Finding {
    Finding {
        severity: Severity::Warning,
        rule: rule.into(),
        at,
        message,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Class {
    Context,
    Unit,
    Distribution,
    Entity,
    External,
    Reference,
    Relation,
}

impl fmt::Display for Class {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Context => "context",
            Self::Unit => "unit",
            Self::Distribution => "distribution",
            Self::Entity => "entity",
            Self::External => "external entity",
            Self::Reference => "reference",
            Self::Relation => "relation",
        })
    }
}

const RECORD_LISTS: [(&str, Class); 7] = [
    ("contexts", Class::Context),
    ("units", Class::Unit),
    ("distributions", Class::Distribution),
    ("entities", Class::Entity),
    ("externals", Class::External),
    ("references", Class::Reference),
    ("relations", Class::Relation),
];

type Contexts<'a> = BTreeSet<&'a str>;

/// An entity's identity: its units, canonical path and kind.
type Identity<'a> = (Vec<&'a str>, &'a str, &'a str);

/// One indexed document's text, when the project files are available.
enum Text {
    /// The digest matches; locators are checked against the text.
    Current(SourceText),
    /// The digest differs; every locator in the document is stale.
    Stale {
        at: String,
        actual: String,
        locators: usize,
    },
    /// The file is missing or not UTF-8; reported once.
    Unreadable,
}

struct Checker<'a> {
    dataset: &'a Value,
    rules: &'static ProfileRules,
    files: Option<&'a dyn ProjectFiles>,
    ids: BTreeMap<&'a str, (Class, &'a Value)>,
    all: Contexts<'a>,
    unit_contexts: BTreeMap<&'a str, Contexts<'a>>,
    entity_contexts: BTreeMap<&'a str, Contexts<'a>>,
    documents: BTreeMap<&'a str, &'a Value>,
    members: BTreeMap<&'a str, Vec<(&'a str, Contexts<'a>)>>,
    texts: BTreeMap<&'a str, Text>,
    used_externals: BTreeSet<&'a str>,
    /// `(document, context)` pairs that hold at least one fact site.
    sites: BTreeSet<(&'a str, &'a str)>,
    /// The units that own the fact whose sites are being checked. A site's
    /// document is a member of one of them, unless the profile defines an
    /// augmentation form. Empty when the owner is unknown.
    site_owners: Vec<&'a str>,
    findings: Vec<Finding>,
}

fn list<'a>(value: &'a Value, key: &str) -> &'a [Value] {
    value
        .get(key)
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or(&[])
}

/// Signature members that hold no type or value text.
const SLOTS_WITHOUT_NAMES: [&str; 13] = [
    "abi",
    "access",
    "at",
    "contexts",
    "delete",
    "entity",
    "kind",
    "mode",
    "modifiers",
    "name",
    "position",
    "tag",
    "variance",
];

/// The ids that `@{id}` names in a type or value text. A double-quoted string
/// literal names nothing.
fn named_ids(slot: &str) -> Vec<&str> {
    let mut ids = Vec::new();
    let mut literal = false;
    let mut chars = slot.char_indices();
    while let Some((i, c)) = chars.next() {
        match c {
            '"' => literal = !literal,
            '\\' if literal => {
                chars.next();
            }
            '@' if !literal && slot[i..].starts_with("@{") => {
                if let Some(end) = slot[i..].find('}') {
                    ids.push(&slot[i + 2..i + end]);
                }
            }
            _ => {}
        }
    }
    ids
}

fn text<'a>(value: &'a Value, key: &str) -> Option<&'a str> {
    value.get(key).and_then(Value::as_str)
}

fn names(contexts: &Contexts<'_>) -> String {
    contexts
        .iter()
        .map(|context| format!("`{context}`"))
        .collect::<Vec<_>>()
        .join(", ")
}

/// Whether a value is the list form of a context-variant attribute.
fn variant_list(value: &Value) -> Option<&[Value]> {
    let items = value.as_array()?;
    let all_entries = !items.is_empty()
        && items.iter().all(|item| {
            item.as_object().is_some_and(|entry| {
                entry.len() == 2 && entry.contains_key("contexts") && entry.contains_key("value")
            })
        });
    all_entries.then_some(items.as_slice())
}

/// The value of a context-variant attribute in one context.
fn value_in<'v>(value: &'v Value, context: &str) -> Option<&'v Value> {
    match variant_list(value) {
        Some(entries) => entries
            .iter()
            .find(|entry| {
                list(entry, "contexts")
                    .iter()
                    .any(|c| c.as_str() == Some(context))
            })
            .and_then(|entry| entry.get("value")),
        None => Some(value),
    }
}

impl<'a> Checker<'a> {
    fn new(
        dataset: &'a Value,
        rules: &'static ProfileRules,
        files: Option<&'a dyn ProjectFiles>,
    ) -> Self {
        Self {
            dataset,
            rules,
            files,
            ids: BTreeMap::new(),
            all: BTreeSet::new(),
            unit_contexts: BTreeMap::new(),
            entity_contexts: BTreeMap::new(),
            documents: BTreeMap::new(),
            members: BTreeMap::new(),
            texts: BTreeMap::new(),
            used_externals: BTreeSet::new(),
            sites: BTreeSet::new(),
            site_owners: Vec::new(),
            findings: Vec::new(),
        }
    }

    fn list(&self, key: &str) -> &'a [Value] {
        list(self.dataset, key)
    }

    /// The membership entries of `public-api`.
    fn public_api_members(&self) -> &'a [Value] {
        self.dataset
            .get("public-api")
            .map(|public_api| list(public_api, "members"))
            .unwrap_or(&[])
    }

    fn error(&mut self, rule: &str, at: String, message: String) {
        self.findings.push(error(rule, at, message));
    }

    fn warning(&mut self, rule: &str, at: String, message: String) {
        self.findings.push(warning(rule, at, message));
    }

    fn profile_rule(&self, rule: &str) -> String {
        format!("{}:{rule}", self.rules.language)
    }

    fn finish(mut self) -> Vec<Finding> {
        let stale: Vec<(String, String, usize)> = self
            .texts
            .values()
            .filter_map(|text| match text {
                Text::Stale {
                    at,
                    actual,
                    locators,
                } => Some((at.clone(), actual.clone(), *locators)),
                _ => None,
            })
            .collect();
        for (at, actual, locators) in stale {
            self.error(
                "model:locator-stale",
                at,
                format!(
                    "the file's SHA-256 is {actual}; the {locators} locator(s) in it are stale"
                ),
            );
        }
        self.findings
    }

    // ------------------------------------------------------------ validate

    fn validate(&mut self) {
        self.collect_ids();
        self.collect_contexts();
        self.check_contexts();
        self.check_units();
        self.check_documents();
        self.check_entities();
        self.check_externals();
        self.check_references();
        self.check_relations();
        self.check_public_api();
        self.check_public_api_recomputed();
        self.check_signature_ids();
        self.check_extensions();
        self.check_unused_externals();
    }

    fn collect_ids(&mut self) {
        let mut first: BTreeMap<&'a str, String> = BTreeMap::new();
        for (key, class) in RECORD_LISTS {
            for (i, record) in self.list(key).iter().enumerate() {
                let Some(id) = text(record, "id") else {
                    continue;
                };
                let at = format!("/{key}/{i}/id");
                if let Some(previous) = first.get(id) {
                    let message = format!("id `{id}` is also the id at {previous}");
                    self.error("model:id-unique", at, message);
                } else {
                    first.insert(id, format!("/{key}/{i}"));
                    self.ids.insert(id, (class, record));
                }
            }
        }
    }

    fn collect_contexts(&mut self) {
        self.all = self
            .list("contexts")
            .iter()
            .filter_map(|context| text(context, "id"))
            .collect();
        for unit in self.list("units") {
            let Some(id) = text(unit, "id") else { continue };
            let contexts = self.declared(unit).unwrap_or_else(|| self.all.clone());
            self.unit_contexts.insert(id, contexts);
        }
        for entity in self.list("entities") {
            let Some(id) = text(entity, "id") else {
                continue;
            };
            let contexts = self
                .declared(entity)
                .unwrap_or_else(|| self.units_contexts(entity));
            self.entity_contexts.entry(id).or_insert(contexts);
        }
    }

    /// The declared contexts a record lists, or `None` when it lists none.
    fn declared(&self, record: &'a Value) -> Option<Contexts<'a>> {
        let contexts = record.get("contexts")?.as_array()?;
        Some(
            contexts
                .iter()
                .filter_map(Value::as_str)
                .filter(|context| self.all.contains(context))
                .collect(),
        )
    }

    /// The contexts in which the units of an entity are built.
    fn units_contexts(&self, entity: &'a Value) -> Contexts<'a> {
        let mut contexts = BTreeSet::new();
        for unit in self.units_of(entity) {
            if let Some(built) = self.unit_contexts.get(unit) {
                contexts.extend(built.iter().copied());
            }
        }
        contexts
    }

    fn units_of(&self, entity: &'a Value) -> Vec<&'a str> {
        match text(entity, "unit") {
            Some(unit) => vec![unit],
            None => list(entity, "units")
                .iter()
                .filter_map(Value::as_str)
                .collect(),
        }
    }

    /// The contexts an entity exists in; empty for an unknown id.
    fn entity_contexts(&self, id: &str) -> Contexts<'a> {
        self.entity_contexts.get(id).cloned().unwrap_or_default()
    }

    /// The contexts in which the unit of an entity, or a unit, is built:
    /// the default contexts of a fact that it owns. Empty for an id that
    /// names neither.
    fn built_contexts(&self, id: &str) -> Contexts<'a> {
        match self.ids.get(id) {
            Some((Class::Entity, record)) => self.units_contexts(record),
            Some((Class::Unit, _)) => self.unit_contexts.get(id).cloned().unwrap_or_default(),
            _ => BTreeSet::new(),
        }
    }

    /// Reports contexts of a fact in which the project entity `id` does not
    /// exist. Other ids are not checked.
    fn check_exists(&mut self, at: String, id: &str, contexts: &Contexts<'a>) {
        if !matches!(self.ids.get(id), Some((Class::Entity, _))) {
            return;
        }
        let exists = self.entity_contexts(id);
        let missing: Contexts<'a> = contexts.difference(&exists).copied().collect();
        if !missing.is_empty() {
            self.error(
                "model:context-scope",
                at,
                format!("entity `{id}` does not exist in {}", names(&missing)),
            );
        }
    }

    /// Reports undeclared context ids in a record's `contexts`.
    fn check_declared(&mut self, at: &str, record: &'a Value) {
        for (k, context) in list(record, "contexts").iter().enumerate() {
            if let Some(context) = context.as_str() {
                if !self.all.contains(context) {
                    self.error(
                        "model:context-declared",
                        format!("{at}/contexts/{k}"),
                        format!("context `{context}` is not declared"),
                    );
                }
            }
        }
    }

    /// The contexts a fact holds in: its own list, or `default`. Reports
    /// listed contexts outside `allowed`.
    fn fact_contexts(
        &mut self,
        at: &str,
        record: &'a Value,
        default: Contexts<'a>,
        allowed: &Contexts<'a>,
        what: &str,
    ) -> Contexts<'a> {
        self.check_declared(at, record);
        match self.declared(record) {
            Some(contexts) => {
                let outside: Contexts<'a> = contexts.difference(allowed).copied().collect();
                if !outside.is_empty() {
                    self.error(
                        "model:context-scope",
                        format!("{at}/contexts"),
                        format!(
                            "{} {} not context(s) of {what}",
                            names(&outside),
                            are(&outside)
                        ),
                    );
                }
                // Later checks see only the contexts in scope, so that one
                // wrong context is one finding.
                contexts.intersection(allowed).copied().collect()
            }
            None => default,
        }
    }

    /// Resolves an id that must name a record of one of `classes`.
    fn expect(&mut self, at: String, id: &'a str, classes: &[Class]) -> Option<&'a Value> {
        match self.ids.get(id) {
            None => {
                self.error("model:id-reference", at, format!("`{id}` names no record"));
                None
            }
            Some((class, record)) if classes.contains(class) => {
                if *class == Class::External {
                    self.used_externals.insert(id);
                }
                Some(*record)
            }
            Some((class, _)) => {
                let expected: Vec<String> = classes.iter().map(Class::to_string).collect();
                self.error(
                    "model:id-reference",
                    at,
                    format!("`{id}` names a {class}, not a {}", expected.join(" or ")),
                );
                None
            }
        }
    }

    fn expect_member(&mut self, at: &str, record: &'a Value, member: &str, classes: &[Class]) {
        if let Some(id) = text(record, member) {
            self.expect(format!("{at}/{member}"), id, classes);
        }
    }

    /// Checks that the context lists of a context-variant attribute are
    /// disjoint and together equal `contexts`.
    fn check_partition(&mut self, at: String, value: &'a Value, contexts: &Contexts<'a>) {
        let Some(entries) = variant_list(value) else {
            return;
        };
        let mut seen: Contexts<'a> = BTreeSet::new();
        for (k, entry) in entries.iter().enumerate() {
            self.check_declared(&format!("{at}/{k}"), entry);
            for context in list(entry, "contexts").iter().filter_map(Value::as_str) {
                if !seen.insert(context) {
                    self.error(
                        "model:context-partition",
                        format!("{at}/{k}/contexts"),
                        format!("context `{context}` is listed twice"),
                    );
                }
            }
        }
        let missing: Contexts<'a> = contexts.difference(&seen).copied().collect();
        let extra: Contexts<'a> = seen
            .difference(contexts)
            .copied()
            .filter(|context| self.all.contains(context))
            .collect();
        if !missing.is_empty() {
            self.error(
                "model:context-partition",
                at.clone(),
                format!("no entry covers {}", names(&missing)),
            );
        }
        if !extra.is_empty() {
            self.error(
                "model:context-partition",
                at,
                format!(
                    "{} {} not context(s) of the record",
                    names(&extra),
                    are(&extra)
                ),
            );
        }
    }

    fn check_contexts(&mut self) {
        let contexts = self.list("contexts");
        for (i, context) in contexts.iter().enumerate() {
            for (j, earlier) in contexts[..i].iter().enumerate() {
                if context.get("parameters") == earlier.get("parameters") {
                    self.error(
                        "model:context-distinct",
                        format!("/contexts/{i}/parameters"),
                        format!(
                            "parameters equal those of context `{}`",
                            text(earlier, "id").unwrap_or(&format!("/contexts/{j}"))
                        ),
                    );
                    break;
                }
            }
            if let Some(parameters) = context.get("parameters") {
                for name in self.rules.sorted_parameters {
                    self.check_sorted(
                        format!("/contexts/{i}/parameters/{name}"),
                        parameters.get(*name),
                    );
                }
            }
        }
    }

    fn check_sorted(&mut self, at: String, value: Option<&Value>) {
        let Some(items) = value.and_then(Value::as_array) else {
            return;
        };
        let strings: Vec<&str> = items.iter().filter_map(Value::as_str).collect();
        if strings.windows(2).any(|pair| pair[0] > pair[1]) {
            let rule = self.profile_rule("sorted");
            self.error(&rule, at, "the list is not sorted".into());
        }
    }

    fn check_units(&mut self) {
        let mut keys: BTreeMap<Vec<String>, &'a str> = BTreeMap::new();
        for (i, unit) in self.list("units").iter().enumerate() {
            let at = format!("/units/{i}");
            let id = text(unit, "id").unwrap_or_default();
            self.check_declared(&at, unit);
            self.expect_member(&at, unit, "distribution", &[Class::Distribution]);
            if let Some(root) = text(unit, "root") {
                if let Some(entity) = self.expect(format!("{at}/root"), root, &[Class::Entity]) {
                    let kind = text(entity, "kind").unwrap_or_default();
                    if !matches!(kind, "module" | "namespace") {
                        self.error(
                            "model:unit-root",
                            format!("{at}/root"),
                            format!("root `{root}` is a {kind}, not a module or namespace"),
                        );
                    }
                    if !self.units_of(entity).contains(&id) {
                        self.error(
                            "model:unit-root",
                            format!("{at}/root"),
                            format!("root `{root}` is not an entity of unit `{id}`"),
                        );
                    }
                }
            }
            let contexts = self.unit_contexts.get(id).cloned().unwrap_or_default();
            if let Some(record) = unit.as_object() {
                for (member, value) in record {
                    if member != "contexts" {
                        self.check_partition(format!("{at}/{member}"), value, &contexts);
                    }
                }
            }
            for name in self.rules.sorted_unit_attributes {
                let Some(value) = unit.get(*name) else {
                    continue;
                };
                match variant_list(value) {
                    Some(entries) => {
                        for (k, entry) in entries.iter().enumerate() {
                            self.check_sorted(format!("{at}/{name}/{k}/value"), entry.get("value"));
                        }
                    }
                    None => self.check_sorted(format!("{at}/{name}"), Some(value)),
                }
            }
            // A unit key names one unit in the whole dataset; a
            // context-variant member can give a unit several keys.
            let unit_keys: BTreeSet<Vec<String>> = contexts
                .iter()
                .map(|context| {
                    self.rules
                        .unit_key
                        .iter()
                        .map(|member| {
                            unit.get(*member)
                                .and_then(|value| value_in(value, context))
                                .map(Value::to_string)
                                .unwrap_or_default()
                        })
                        .collect()
                })
                .collect();
            for key in unit_keys {
                match keys.get(&key) {
                    Some(other) if *other != id => {
                        let message = format!("unit key equals that of unit `{other}`");
                        self.error("model:unit-key", at.clone(), message);
                        break;
                    }
                    Some(_) => {}
                    None => {
                        keys.insert(key, id);
                    }
                }
            }
        }
    }

    fn check_documents(&mut self) {
        for (i, document) in self.list("documents").iter().enumerate() {
            let at = format!("/documents/{i}");
            let Some(path) = text(document, "path") else {
                continue;
            };
            if self.documents.insert(path, document).is_some() {
                self.error(
                    "model:document-unique",
                    format!("{at}/path"),
                    format!("`{path}` has another inventory entry"),
                );
                continue;
            }
            if path == DATASET_FILE {
                self.error(
                    "model:document-inventory",
                    format!("{at}/path"),
                    "the dataset file is not a document".into(),
                );
            }
            let mut members = Vec::new();
            for (k, member) in list(document, "members").iter().enumerate() {
                let member_at = format!("{at}/members/{k}");
                let Some(unit) = text(member, "unit") else {
                    continue;
                };
                if self
                    .expect(format!("{member_at}/unit"), unit, &[Class::Unit])
                    .is_none()
                {
                    continue;
                }
                let built = self.unit_contexts.get(unit).cloned().unwrap_or_default();
                let contexts = self.fact_contexts(
                    &member_at,
                    member,
                    built.clone(),
                    &built,
                    &format!("unit `{unit}`"),
                );
                members.push((unit, contexts));
            }
            self.members.insert(path, members);
        }

        let Some(files) = self.files else { return };
        let tracked = match files.tracked() {
            Ok(tracked) => tracked,
            Err(cause) => {
                self.error(
                    "model:document-inventory",
                    "/documents".into(),
                    format!("cannot list tracked files: {cause:#}"),
                );
                return;
            }
        };
        let tracked: BTreeSet<&str> = tracked.iter().map(String::as_str).collect();
        for path in &tracked {
            if !self.documents.contains_key(path) {
                self.error(
                    "model:document-inventory",
                    "/documents".into(),
                    format!("tracked file `{path}` has no inventory entry"),
                );
            }
        }
        for (i, document) in self.list("documents").iter().enumerate() {
            let at = format!("/documents/{i}");
            let Some(path) = text(document, "path") else {
                continue;
            };
            if !tracked.contains(path) {
                if path != DATASET_FILE {
                    self.error(
                        "model:document-inventory",
                        format!("{at}/path"),
                        format!("`{path}` is not a tracked file"),
                    );
                }
                continue;
            }
            if self.texts.contains_key(path) {
                continue;
            }
            let bytes = match files.read(path) {
                Ok(Some(bytes)) => bytes,
                Ok(None) => {
                    self.error(
                        "model:document-inventory",
                        format!("{at}/path"),
                        format!("tracked file `{path}` is missing from the working tree"),
                    );
                    self.texts.insert(path, Text::Unreadable);
                    continue;
                }
                Err(cause) => {
                    self.error(
                        "model:document-inventory",
                        format!("{at}/path"),
                        format!("cannot read `{path}`: {cause:#}"),
                    );
                    self.texts.insert(path, Text::Unreadable);
                    continue;
                }
            };
            let actual = sha256_hex(&bytes);
            let indexed = text(document, "disposition") == Some("indexed");
            if text(document, "sha256") != Some(actual.as_str()) {
                if indexed {
                    self.texts.insert(
                        path,
                        Text::Stale {
                            at: format!("{at}/sha256"),
                            actual,
                            locators: 0,
                        },
                    );
                } else {
                    self.error(
                        "model:document-digest",
                        format!("{at}/sha256"),
                        format!("the file's SHA-256 is {actual}"),
                    );
                }
                continue;
            }
            if !indexed {
                continue;
            }
            match SourceText::decode(&bytes) {
                Ok(source) => {
                    self.texts.insert(path, Text::Current(source));
                }
                Err(cause) => {
                    self.error(
                        "model:document-utf8",
                        format!("{at}/path"),
                        format!("indexed document `{path}` is not UTF-8: {cause}"),
                    );
                    self.texts.insert(path, Text::Unreadable);
                }
            }
        }
    }

    /// Checks one locator: its document is an indexed document of a unit in
    /// every context of the fact, and its text holds.
    fn check_locator(&mut self, at: String, value: &'a Value, contexts: &Contexts<'a>) {
        let Some(locator) = Locator::from_value(value) else {
            // The schema accepted it, so a coordinate is too large to be a
            // position.
            self.error(
                "model:locator-range",
                at,
                "a coordinate is not a representable position".into(),
            );
            return;
        };
        if locator.end < locator.start {
            self.error(
                "model:locator-order",
                at.clone(),
                format!("end {} precedes start {}", locator.end, locator.start),
            );
            return;
        }
        if locator.end == locator.start {
            self.warning(
                "model:locator-empty",
                at.clone(),
                "the span is empty; a site spans a token".into(),
            );
        }
        let Some((&path, document)) = self.documents.get_key_value(locator.path.as_str()) else {
            self.error(
                "model:locator-document",
                format!("{at}/path"),
                format!("`{}` is not in the document inventory", locator.path),
            );
            return;
        };
        let disposition = text(document, "disposition").unwrap_or_default();
        if disposition != "indexed" {
            self.error(
                "model:site-membership",
                format!("{at}/path"),
                format!("`{path}` is `{disposition}`, not an indexed document"),
            );
            return;
        }
        let members = self.members.get(path).cloned().unwrap_or_default();
        let outside: Contexts<'a> = contexts
            .iter()
            .copied()
            .filter(|context| !members.iter().any(|(_, member)| member.contains(context)))
            .collect();
        if !outside.is_empty() {
            self.error(
                "model:site-membership",
                format!("{at}/path"),
                format!("`{path}` is a member of no unit in {}", names(&outside)),
            );
        } else if !self.rules.augmentation && !self.site_owners.is_empty() {
            let owners = self.site_owners.clone();
            let foreign: Contexts<'a> = contexts
                .iter()
                .copied()
                .filter(|context| {
                    !members
                        .iter()
                        .any(|(unit, member)| owners.contains(unit) && member.contains(context))
                })
                .collect();
            if !foreign.is_empty() {
                let units: Vec<String> = owners.iter().map(|unit| format!("`{unit}`")).collect();
                self.error(
                    "model:site-membership",
                    format!("{at}/path"),
                    format!(
                        "`{path}` is not a member of the fact's unit {} in {}",
                        units.join(", "),
                        names(&foreign)
                    ),
                );
            }
        }
        for context in contexts {
            self.sites.insert((path, context));
        }
        match self.texts.get_mut(path) {
            Some(Text::Current(source)) => {
                let result = source.check(&locator);
                let occurrences = source.find(&locator.text);
                match result {
                    Ok(()) => {}
                    Err(LocatorError::OutOfRange(position)) => self.error(
                        "model:locator-range",
                        at,
                        format!("position {position} is not in `{path}`"),
                    ),
                    Err(problem) => {
                        let mut message = format!("stale: {problem}");
                        if !occurrences.is_empty() {
                            let shown: Vec<String> = occurrences
                                .iter()
                                .take(3)
                                .map(Position::to_string)
                                .collect();
                            let more = if occurrences.len() > 3 { ", …" } else { "" };
                            message.push_str(&format!(
                                "; the stored text occurs at {}{more}",
                                shown.join(", ")
                            ));
                        }
                        self.error("model:locator-stale", at, message);
                    }
                }
            }
            Some(Text::Stale { locators, .. }) => *locators += 1,
            Some(Text::Unreadable) | None => {}
        }
    }

    fn check_entities(&mut self) {
        // Units that compile a generated indexed document: an entity of
        // such a unit can be defined by code with no name token in it.
        let generated_units: BTreeSet<&'a str> = self
            .list("documents")
            .iter()
            .filter(|d| text(d, "origin") == Some("generated"))
            .flat_map(|d| list(d, "members").iter().filter_map(|m| text(m, "unit")))
            .collect();
        let mut identities: BTreeMap<Identity<'a>, Vec<(&'a str, Contexts<'a>)>> = BTreeMap::new();
        for (i, entity) in self.list("entities").iter().enumerate() {
            let at = format!("/entities/{i}");
            let id = text(entity, "id").unwrap_or_default();
            if let Some(unit) = text(entity, "unit") {
                self.expect(format!("{at}/unit"), unit, &[Class::Unit]);
            }
            for (k, unit) in list(entity, "units").iter().enumerate() {
                if let Some(unit) = unit.as_str() {
                    self.expect(format!("{at}/units/{k}"), unit, &[Class::Unit]);
                }
            }
            let built = self.units_contexts(entity);
            let contexts =
                self.fact_contexts(&at, entity, built.clone(), &built, "the entity's unit");

            if let Some(parent) = text(entity, "parent") {
                let resolves = self
                    .expect(format!("{at}/parent"), parent, &[Class::Entity])
                    .is_some();
                if parent == id || (resolves && self.parent_cycle(id)) {
                    self.error(
                        "model:entity-parent",
                        format!("{at}/parent"),
                        "the parent chain returns to this entity".into(),
                    );
                }
            }

            for member in [
                "language-kind",
                "declared-visibility",
                "effective-visibility",
            ] {
                if let Some(value) = entity.get(member) {
                    self.check_partition(format!("{at}/{member}"), value, &contexts);
                }
            }
            if let Some(visibility) = entity.get("effective-visibility") {
                self.check_scopes(&format!("{at}/effective-visibility"), visibility);
                self.check_whole_unit_scope(&at, entity, visibility);
                self.check_undistributed(&at, entity, visibility, &contexts);
            }

            let origin = text(entity, "origin").unwrap_or_default();
            let definitions = list(entity, "definitions");
            let in_generated_document = self
                .units_of(entity)
                .iter()
                .any(|unit| generated_units.contains(unit));
            if definitions.is_empty() && origin == "generated" && !in_generated_document {
                for member in ["anchor", "generated-by"] {
                    if entity.get(member).is_none() {
                        self.error(
                            "model:entity-origin",
                            at.clone(),
                            format!("a generated entity without definition sites has `{member}`"),
                        );
                    }
                }
            }
            self.site_owners = self.units_of(entity);
            let mut sites = BTreeSet::new();
            for (k, definition) in definitions.iter().enumerate() {
                let Some(site) = definition.get("at") else {
                    continue;
                };
                let site_at = format!("{at}/definitions/{k}/at");
                if !sites.insert(site_key(site)) {
                    self.error(
                        "model:locator-ambiguous",
                        site_at.clone(),
                        "the entity lists this definition site twice".into(),
                    );
                }
                let in_generated = text(site, "path")
                    .and_then(|path| self.documents.get(path))
                    .and_then(|document| text(document, "origin"))
                    == Some("generated");
                if in_generated && origin != "generated" {
                    self.error(
                        "model:entity-origin",
                        format!("{at}/origin"),
                        "an entity defined in a generated document has `origin: generated`".into(),
                    );
                }
                // A site may hold in fewer contexts than its entity: a
                // document compiled only in some contexts.
                let site_contexts = self.fact_contexts(
                    &format!("{at}/definitions/{k}"),
                    definition,
                    contexts.clone(),
                    &contexts,
                    "the entity",
                );
                self.check_locator(site_at, site, &site_contexts);
            }
            if let Some(anchor) = entity.get("anchor") {
                self.check_locator(format!("{at}/anchor"), anchor, &contexts);
            }
            if let Some(site) = entity.get("generated-by").and_then(|g| g.get("at")) {
                self.check_locator(format!("{at}/generated-by/at"), site, &contexts);
            }
            self.site_owners.clear();

            let mut units = self.units_of(entity);
            units.sort_unstable();
            let key = (
                units,
                text(entity, "path").unwrap_or_default(),
                text(entity, "kind").unwrap_or_default(),
            );
            let records = identities.entry(key).or_default();
            if let Some((other, shared)) = records
                .iter()
                .find_map(|(other, theirs)| overlap(theirs, &contexts).map(|s| (*other, s)))
            {
                self.error(
                    "model:entity-identity",
                    at.clone(),
                    format!(
                        "unit, path and kind equal those of entity `{other}` in {}",
                        names(&shared)
                    ),
                );
            }
            records.push((id, contexts));
        }
    }

    /// At the `unit` boundary, a scope that holds the entity's unit or its
    /// unit root covers the whole unit, so the value is written `all`.
    fn check_whole_unit_scope(&mut self, at: &str, entity: &'a Value, visibility: &'a Value) {
        let mut whole: BTreeSet<&'a str> = BTreeSet::new();
        for unit in self.units_of(entity) {
            whole.insert(unit);
            if let Some((Class::Unit, record)) = self.ids.get(unit) {
                if let Some(root) = text(record, "root") {
                    whole.insert(root);
                }
            }
        }
        let values: Vec<(String, &'a Value)> = match variant_list(visibility) {
            Some(entries) => entries
                .iter()
                .enumerate()
                .filter_map(|(k, e)| e.get("value").map(|v| (format!("/{k}/value"), v)))
                .collect(),
            None => vec![(String::new(), visibility)],
        };
        for (suffix, value) in values {
            let Some(access) = value.get("unit") else {
                continue;
            };
            let accesses: Vec<&Value> = match access.as_array() {
                Some(items) => items.iter().collect(),
                None => vec![access],
            };
            let covers = accesses.iter().any(|access| {
                text(access, "access") == Some("scoped")
                    && list(access, "scope")
                        .iter()
                        .filter_map(Value::as_str)
                        .any(|member| whole.contains(member))
            });
            if covers {
                self.error(
                    "model:effective-visibility",
                    format!("{at}/effective-visibility{suffix}/unit"),
                    "the scope covers the whole unit; the value is `all`".into(),
                );
            }
        }
    }

    /// An entity in a unit with no distribution has `external: none`.
    fn check_undistributed(
        &mut self,
        at: &str,
        entity: &'a Value,
        visibility: &'a Value,
        contexts: &Contexts<'a>,
    ) {
        let distributed = self.units_of(entity).iter().any(|unit| {
            matches!(self.ids.get(unit), Some((Class::Unit, record)) if record.get("distribution").is_some())
        });
        if distributed {
            return;
        }
        let open: Contexts<'a> = contexts
            .iter()
            .copied()
            .filter(|context| {
                value_in(visibility, context)
                    .and_then(|value| value.get("external"))
                    .is_some_and(|external| external.as_str() != Some("none"))
            })
            .collect();
        if !open.is_empty() {
            self.error(
                "model:effective-visibility",
                format!("{at}/effective-visibility"),
                format!(
                    "the entity's unit has no distribution, but `external` is not `none` in {}",
                    names(&open)
                ),
            );
        }
    }

    fn parent_cycle(&self, id: &str) -> bool {
        let mut seen = BTreeSet::new();
        let mut current = id;
        while let Some((Class::Entity, record)) = self.ids.get(current) {
            let Some(parent) = text(record, "parent") else {
                return false;
            };
            if parent == id {
                return true;
            }
            if !seen.insert(parent) {
                return false;
            }
            current = parent;
        }
        false
    }

    /// Checks `scope` and `within` members: entity ids, unit ids or
    /// inventory paths.
    fn check_scopes(&mut self, at: &str, visibility: &'a Value) {
        let values: Vec<(String, &'a Value)> = match variant_list(visibility) {
            Some(entries) => entries
                .iter()
                .enumerate()
                .filter_map(|(k, entry)| entry.get("value").map(|v| (format!("{at}/{k}/value"), v)))
                .collect(),
            None => vec![(at.to_owned(), visibility)],
        };
        for (value_at, value) in values {
            for boundary in ["unit", "project", "external"] {
                let Some(access) = value.get(boundary) else {
                    continue;
                };
                let accesses: Vec<(String, &'a Value)> = match access.as_array() {
                    Some(items) => items
                        .iter()
                        .enumerate()
                        .map(|(k, item)| (format!("{value_at}/{boundary}/{k}"), item))
                        .collect(),
                    None => vec![(format!("{value_at}/{boundary}"), access)],
                };
                for (access_at, access) in accesses {
                    for qualifier in ["scope", "within"] {
                        for (k, member) in list(access, qualifier).iter().enumerate() {
                            let Some(member) = member.as_str() else {
                                continue;
                            };
                            let resolves = matches!(
                                self.ids.get(member),
                                Some((Class::Entity | Class::Unit, _))
                            ) || self.documents.contains_key(member)
                                || member
                                    .strip_prefix("external:")
                                    .is_some_and(|key| !key.is_empty());
                            if !resolves {
                                self.error(
                                    "model:id-reference",
                                    format!("{access_at}/{qualifier}/{k}"),
                                    format!(
                                        "`{member}` names no entity, unit, inventory document \
                                         or external unit"
                                    ),
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    fn check_externals(&mut self) {
        let mut keys: BTreeMap<[&'a str; 4], &'a str> = BTreeMap::new();
        for (i, external) in self.list("externals").iter().enumerate() {
            let key = [
                text(external, "distribution").unwrap_or_default(),
                text(external, "version").unwrap_or_default(),
                text(external, "path").unwrap_or_default(),
                text(external, "kind").unwrap_or_default(),
            ];
            let id = text(external, "id").unwrap_or_default();
            if let Some(other) = keys.get(&key) {
                self.error(
                    "model:external-key",
                    format!("/externals/{i}"),
                    format!("external key equals that of `{other}`"),
                );
            } else {
                keys.insert(key, id);
            }
        }
    }

    fn check_references(&mut self) {
        let mut sites: BTreeMap<(String, &'a str, &'a str), Vec<Contexts<'a>>> = BTreeMap::new();
        for (i, reference) in self.list("references").iter().enumerate() {
            let at = format!("/references/{i}");
            let from = text(reference, "from").unwrap_or_default();
            self.expect(format!("{at}/from"), from, &[Class::Entity]);
            let built = self.built_contexts(from);
            let contexts = self.fact_contexts(
                &at,
                reference,
                built.clone(),
                &built,
                &format!("the unit of `{from}`"),
            );
            self.check_exists(format!("{at}/from"), from, &contexts);
            if let Some(target) = text(reference, "target") {
                self.check_exists(format!("{at}/target"), target, &contexts);
            }
            for (k, candidate) in list(reference, "candidates").iter().enumerate() {
                if let Some(candidate) = candidate.as_str() {
                    self.check_exists(format!("{at}/candidates/{k}"), candidate, &contexts);
                }
            }
            let target = text(reference, "target");
            match (text(reference, "disposition"), target) {
                (None, Some(target)) => {
                    self.expect(
                        format!("{at}/target"),
                        target,
                        &[Class::Entity, Class::External],
                    );
                }
                (Some("resolved"), Some(target)) => {
                    self.expect(format!("{at}/target"), target, &[Class::Entity]);
                }
                (Some("external"), Some(target)) => {
                    self.expect(format!("{at}/target"), target, &[Class::External]);
                }
                _ => {}
            }
            for (k, candidate) in list(reference, "candidates").iter().enumerate() {
                if let Some(candidate) = candidate.as_str() {
                    self.expect(
                        format!("{at}/candidates/{k}"),
                        candidate,
                        &[Class::Entity, Class::External],
                    );
                }
            }
            if let Some(site) = reference.get("at") {
                self.site_owners = self.owner_units(from);
                self.check_locator(format!("{at}/at"), site, &contexts);
                self.site_owners.clear();
                let key = (
                    site_key(site),
                    from,
                    text(reference, "role").unwrap_or_default(),
                );
                let earlier = sites.entry(key).or_default();
                if let Some(shared) = earlier.iter().find_map(|theirs| overlap(theirs, &contexts)) {
                    self.error(
                        "model:locator-ambiguous",
                        format!("{at}/at"),
                        format!(
                            "another reference has this site, `from` and role in {}",
                            names(&shared)
                        ),
                    );
                }
                earlier.push(contexts);
            }
        }
    }

    fn check_relations(&mut self) {
        let mut seen: BTreeMap<[&'a str; 4], Vec<Contexts<'a>>> = BTreeMap::new();
        for (i, relation) in self.list("relations").iter().enumerate() {
            let at = format!("/relations/{i}");
            let from = text(relation, "from").unwrap_or_default();
            self.expect(format!("{at}/from"), from, &[Class::Entity]);
            self.expect_member(&at, relation, "to", &[Class::Entity, Class::External]);
            let built = self.built_contexts(from);
            let contexts = self.fact_contexts(
                &at,
                relation,
                built.clone(),
                &built,
                &format!("the unit of `{from}`"),
            );
            self.check_exists(format!("{at}/from"), from, &contexts);
            if let Some(to) = text(relation, "to") {
                self.check_exists(format!("{at}/to"), to, &contexts);
            }
            if let Some(site) = relation.get("at") {
                self.site_owners = self.owner_units(from);
                self.check_locator(format!("{at}/at"), site, &contexts);
                self.site_owners.clear();
            }
            let key = [
                text(relation, "kind").unwrap_or_default(),
                from,
                text(relation, "to").unwrap_or_default(),
                text(relation, "name").unwrap_or_default(),
            ];
            let earlier = seen.entry(key).or_default();
            if let Some(shared) = earlier.iter().find_map(|theirs| overlap(theirs, &contexts)) {
                self.error(
                    "model:relation-duplicate",
                    at,
                    format!("the same relation is stated again in {}", names(&shared)),
                );
            }
            earlier.push(contexts);
        }
    }

    fn check_public_api(&mut self) {
        let mut seen: BTreeMap<(&'a str, &'a str), Vec<Contexts<'a>>> = BTreeMap::new();
        for (i, membership) in self.public_api_members().iter().enumerate() {
            let at = format!("/public-api/members/{i}");
            let distribution = text(membership, "distribution").unwrap_or_default();
            let resolved = self
                .expect(
                    format!("{at}/distribution"),
                    distribution,
                    &[Class::Distribution],
                )
                .is_some();
            let id = text(membership, "entity").unwrap_or_default();
            // A forwarded type is a member that is an external entity.
            let entity = match self.ids.get(id) {
                Some((Class::External, _)) => None,
                _ => self.expect(format!("{at}/entity"), id, &[Class::Entity]),
            };
            // A forwarded external type holds wherever the dataset does.
            let built = match entity {
                Some(_) => self.built_contexts(id),
                None => self.all.clone(),
            };
            let contexts = self.fact_contexts(
                &at,
                membership,
                built.clone(),
                &built,
                &format!("the unit of `{id}`"),
            );
            self.check_exists(format!("{at}/entity"), id, &contexts);
            let earlier = seen.entry((distribution, id)).or_default();
            if let Some(shared) = earlier.iter().find_map(|theirs| overlap(theirs, &contexts)) {
                self.error(
                    "model:public-api-duplicate",
                    at.clone(),
                    format!(
                        "another entry names this distribution and entity in {}",
                        names(&shared)
                    ),
                );
            }
            earlier.push(contexts.clone());
            let Some(entity) = entity.filter(|_| resolved) else {
                continue;
            };

            let access = text(membership, "access").unwrap_or_default();
            let Some(visibility) = entity.get("effective-visibility") else {
                continue;
            };
            let mut wrong: BTreeMap<&'static str, Contexts<'a>> = BTreeMap::new();
            for context in &contexts {
                let external = value_in(visibility, context).and_then(|v| v.get("external"));
                let found = external.map(external_access).unwrap_or("none");
                if found != access {
                    wrong.entry(found).or_default().insert(context);
                }
            }
            for (found, contexts) in wrong {
                self.error(
                    "model:public-api-access",
                    format!("{at}/access"),
                    format!(
                        "the entity's effective `external` access is `{found}` in {}",
                        names(&contexts)
                    ),
                );
            }
        }
    }

    /// Criteria 2 and 5 for signature records: each names a project entity, in
    /// contexts where it exists, and an external that a type text names is used.
    fn check_signature_ids(&mut self) {
        let signatures = self
            .dataset
            .get("public-api")
            .map(|public_api| list(public_api, "signatures"))
            .unwrap_or(&[]);
        for (i, signature) in signatures.iter().enumerate() {
            let at = format!("/public-api/signatures/{i}");
            let id = text(signature, "entity").unwrap_or_default();
            if self
                .expect(format!("{at}/entity"), id, &[Class::Entity])
                .is_some()
            {
                let built = self.built_contexts(id);
                let contexts = self.fact_contexts(
                    &at,
                    signature,
                    built.clone(),
                    &built,
                    &format!("the unit of `{id}`"),
                );
                self.check_exists(format!("{at}/entity"), id, &contexts);
                self.site_owners = self.owner_units(id);
                for (k, alternative) in list(signature, "alternatives").iter().enumerate() {
                    if let Some(site) = alternative.get("at") {
                        self.check_locator(format!("{at}/alternatives/{k}/at"), site, &contexts);
                    }
                }
                self.site_owners.clear();
            }
            self.use_externals_named_in(signature);
        }
    }

    /// Marks each external that a `@{id}` in a type or value slot of `value`
    /// names.
    fn use_externals_named_in(&mut self, value: &'a Value) {
        match value {
            Value::String(slot) => {
                for id in named_ids(slot) {
                    if let Some((id, (Class::External, _))) = self.ids.get_key_value(id) {
                        self.used_externals.insert(id);
                    }
                }
            }
            Value::Array(items) => items
                .iter()
                .for_each(|item| self.use_externals_named_in(item)),
            Value::Object(members) => members
                .iter()
                .filter(|(slot, _)| !SLOTS_WITHOUT_NAMES.contains(&slot.as_str()))
                .for_each(|(_, member)| self.use_externals_named_in(member)),
            _ => {}
        }
    }

    fn check_extensions(&mut self) {
        let Some(extensions) = self.dataset.get("extensions").and_then(Value::as_object) else {
            return;
        };
        for (kind, facts) in extensions {
            let Some(extension) = self.rules.extension(kind) else {
                continue;
            };
            let mut keys: BTreeMap<Vec<String>, Vec<Contexts<'a>>> = BTreeMap::new();
            for (j, fact) in facts.as_array().into_iter().flatten().enumerate() {
                let at = format!("/extensions/{kind}/{j}");
                let owner = self.extension_owner(fact, extension);
                self.site_owners = owner.map(|o| self.owner_units(o)).unwrap_or_default();
                let (built, what) = match owner {
                    Some(owner) => (self.built_contexts(owner), format!("the unit of `{owner}`")),
                    None => match self.site_contexts(fact, extension) {
                        Some((path, contexts)) => (contexts, format!("a unit compiling `{path}`")),
                        None => (self.all.clone(), "the dataset".to_string()),
                    },
                };
                let contexts = self.fact_contexts(&at, fact, built.clone(), &built, &what);
                for (member, names) in extension.members {
                    let Some(value) = fact.get(*member) else {
                        continue;
                    };
                    let member_at = format!("{at}/{member}");
                    self.check_extension_member(member_at, value, *names, &contexts);
                }
                self.site_owners.clear();
                if kind == "dart:mixin-application" {
                    self.check_relation_kind(&at, fact, "relation", "inherits");
                }
                let key: Vec<String> = extension
                    .key
                    .iter()
                    .map(|member| fact.get(*member).map(Value::to_string).unwrap_or_default())
                    .collect();
                let earlier = keys.entry(key).or_default();
                if let Some(shared) = earlier.iter().find_map(|theirs| overlap(theirs, &contexts)) {
                    self.error(
                        "model:extension-key",
                        at,
                        format!(
                            "another `{kind}` fact has key {} in {}",
                            extension.key.join(", "),
                            names(&shared)
                        ),
                    );
                }
                earlier.push(contexts);
            }
        }
    }

    /// The record whose unit owns an extension fact: the first member, in
    /// the profile's order, that names a project entity or a unit, or the
    /// `from` entity of a named reference or relation.
    fn extension_owner(
        &self,
        fact: &'a Value,
        extension: &crate::rules::Extension,
    ) -> Option<&'a str> {
        extension.members.iter().find_map(|(member, names)| {
            // A unit-or-assembly member may be an assembly name that equals
            // an unrelated id; it never decides ownership.
            if matches!(names, Names::UnitOrName | Names::Locator) {
                return None;
            }
            let id = text(fact, member)?;
            match self.ids.get(id)? {
                (Class::Entity | Class::Unit, _) => Some(id),
                (Class::Reference | Class::Relation, record) => text(record, "from"),
                _ => None,
            }
        })
    }

    /// The units of an entity, or the unit itself; empty for other ids.
    fn owner_units(&self, id: &'a str) -> Vec<&'a str> {
        match self.ids.get(id) {
            Some((Class::Entity, record)) => self.units_of(record),
            Some((Class::Unit, _)) => vec![id],
            _ => Vec::new(),
        }
    }

    /// For an extension fact with no owner record: the contexts in which a
    /// unit that compiles the document of its first site is built.
    fn site_contexts(
        &self,
        fact: &'a Value,
        extension: &crate::rules::Extension,
    ) -> Option<(&'a str, Contexts<'a>)> {
        extension.members.iter().find_map(|(member, names)| {
            if *names != Names::Locator {
                return None;
            }
            let path = text(fact.get(*member)?, "path")?;
            let members = self.members.get(path)?;
            let contexts = members
                .iter()
                .filter_map(|(unit, _)| self.unit_contexts.get(unit))
                .flat_map(|contexts| contexts.iter().copied())
                .collect();
            Some((path, contexts))
        })
    }

    fn check_extension_member(
        &mut self,
        at: String,
        value: &'a Value,
        names: Names,
        contexts: &Contexts<'a>,
    ) {
        let id = value.as_str();
        match (names, id) {
            (Names::Entity, Some(id)) => {
                if self.expect(at.clone(), id, &[Class::Entity]).is_some() {
                    self.check_exists(at, id, contexts);
                }
            }
            (Names::EntityOrExternal, Some(id)) => {
                if self
                    .expect(at.clone(), id, &[Class::Entity, Class::External])
                    .is_some()
                {
                    self.check_exists(at, id, contexts);
                }
            }
            (Names::Unit, Some(id)) => {
                self.expect(at, id, &[Class::Unit]);
            }
            // A unit id, or else an external assembly name, which may equal
            // the id of a record of another kind.
            (Names::UnitOrName, _) => {}
            (Names::Distribution, Some(id)) => {
                self.expect(at, id, &[Class::Distribution]);
            }
            (Names::Relation, Some(id)) => {
                self.expect(at, id, &[Class::Relation]);
            }
            (Names::Reference, Some(id)) => {
                self.expect(at, id, &[Class::Reference]);
            }
            (Names::Locator, _) => self.check_locator(at, value, contexts),
            (Names::Documents, _) => {
                for (k, path) in value.as_array().into_iter().flatten().enumerate() {
                    let Some(path) = path.as_str() else { continue };
                    if !self.documents.contains_key(path) {
                        self.error(
                            "model:id-reference",
                            format!("{at}/{k}"),
                            format!("`{path}` is not in the document inventory"),
                        );
                    }
                }
            }
            (Names::SortedList, _) => self.check_sorted(at, Some(value)),
            (Names::Entities, _) => {
                for (k, item) in value.as_array().into_iter().flatten().enumerate() {
                    let Some(id) = item.as_str() else { continue };
                    let item_at = format!("{at}/{k}");
                    if self.expect(item_at.clone(), id, &[Class::Entity]).is_some() {
                        self.check_exists(item_at, id, contexts);
                    }
                }
                self.check_sorted(at, Some(value));
            }
            _ => {}
        }
    }

    fn check_relation_kind(&mut self, at: &str, fact: &'a Value, member: &str, kind: &str) {
        let Some(id) = text(fact, member) else { return };
        if let Some((Class::Relation, relation)) = self.ids.get(id) {
            let found = text(relation, "kind").unwrap_or_default();
            if found != kind {
                let rule = self.profile_rule("extension-target");
                self.error(
                    &rule,
                    format!("{at}/{member}"),
                    format!("`{id}` is an `{found}` relation, not `{kind}`"),
                );
            }
        }
    }

    fn check_unused_externals(&mut self) {
        for (i, external) in self.list("externals").iter().enumerate() {
            let Some(id) = text(external, "id") else {
                continue;
            };
            if !self.used_externals.contains(id) {
                self.error(
                    "model:external-unused",
                    format!("/externals/{i}"),
                    format!("no reference, relation, extension fact or signature targets `{id}`"),
                );
            }
        }
    }

    // --------------------------------------------------------------- audit

    fn audit(&mut self) {
        self.audit_document_classes();
        self.audit_out_of_profile();
        self.audit_coverage();
        if self.rules.language == "go" {
            self.audit_go_generated();
        }
        if self.rules.language == "rust" {
            self.audit_rust_visibility_paths();
        }
    }

    fn audit_document_classes(&mut self) {
        let rule = self.profile_rule("document-class");
        let language = self.rules.language;
        for (i, document) in self.list("documents").iter().enumerate() {
            let at = format!("/documents/{i}/disposition");
            let Some(path) = text(document, "path") else {
                continue;
            };
            let disposition = text(document, "disposition").unwrap_or_default();
            if self.rules.is_source(path) {
                let allowed = matches!(disposition, "indexed" | "inactive" | "external")
                    || (disposition == "configuration" && self.rules.is_source_configuration(path));
                if !allowed {
                    self.error(
                        &rule,
                        at,
                        format!(
                            "`{path}` is {language} source: indexed, inactive or external, not `{disposition}`"
                        ),
                    );
                }
            } else if disposition == "indexed" && !self.rules.is_indexable(path) {
                self.error(
                    &rule,
                    at,
                    format!("`{path}` is not {language} source and is not indexed"),
                );
            } else if self.rules.is_configuration(path)
                && !matches!(disposition, "configuration" | "external")
            {
                self.warning(
                    &rule,
                    at,
                    format!("the profile classifies `{path}` as configuration"),
                );
            }
        }
    }

    fn audit_out_of_profile(&mut self) {
        let kinds: BTreeMap<&'a str, &'a str> = self
            .list("entities")
            .iter()
            .filter_map(|e| Some((text(e, "id")?, text(e, "kind")?)))
            .collect();
        for (i, entity) in self.list("entities").iter().enumerate() {
            let kind = text(entity, "kind").unwrap_or_default();
            let mut parent = text(entity, "parent");
            // A parameter's own callable contains it; anything above that
            // is a body.
            if matches!(kind, "parameter" | "type-parameter") {
                parent = parent
                    .and_then(|p| self.ids.get(p))
                    .and_then(|(_, record)| text(record, "parent"));
            }
            let mut seen = BTreeSet::new();
            while let Some(id) = parent {
                if !seen.insert(id) {
                    break;
                }
                if kinds.get(id) == Some(&"callable") {
                    self.error(
                        "model:out-of-profile",
                        format!("/entities/{i}"),
                        format!("a {kind} declared inside callable `{id}` is out of profile"),
                    );
                    break;
                }
                parent = self
                    .ids
                    .get(id)
                    .and_then(|(_, record)| text(record, "parent"));
            }
        }
        for (i, reference) in self.list("references").iter().enumerate() {
            let at = format!("/references/{i}");
            let mut targets: Vec<(String, &str)> = Vec::new();
            if let Some(target) = text(reference, "target") {
                targets.push((format!("{at}/target"), target));
            }
            for (k, candidate) in list(reference, "candidates").iter().enumerate() {
                if let Some(candidate) = candidate.as_str() {
                    targets.push((format!("{at}/candidates/{k}"), candidate));
                }
            }
            for (target_at, target) in targets {
                let kind = self
                    .ids
                    .get(target)
                    .and_then(|(_, record)| text(record, "kind"));
                if let Some(kind @ ("parameter" | "type-parameter")) = kind {
                    self.error(
                        "model:out-of-profile",
                        target_at,
                        format!("a reference to a {kind} is out of profile"),
                    );
                }
            }
        }
    }

    fn audit_coverage(&mut self) {
        let documents: Vec<(usize, &'a str)> = self
            .list("documents")
            .iter()
            .enumerate()
            .filter(|(_, d)| text(d, "disposition") == Some("indexed"))
            .filter_map(|(i, d)| Some((i, text(d, "path")?)))
            .collect();
        for (i, path) in documents {
            let members = self.members.get(path).cloned().unwrap_or_default();
            let compiled: Contexts<'a> = members
                .iter()
                .flat_map(|(_, contexts)| contexts.iter().copied())
                .collect();
            let empty: Contexts<'a> = compiled
                .into_iter()
                .filter(|context| !self.sites.contains(&(path, context)))
                .collect();
            if !empty.is_empty() {
                self.warning(
                    "model:coverage",
                    format!("/documents/{i}"),
                    format!("`{path}` holds no fact site in {}", names(&empty)),
                );
            }
        }
    }

    fn audit_go_generated(&mut self) {
        for (i, document) in self.list("documents").iter().enumerate() {
            let Some(path) = text(document, "path") else {
                continue;
            };
            if !path.ends_with(".go") || text(document, "disposition") != Some("indexed") {
                continue;
            }
            let Some(Text::Current(source)) = self.texts.get(path) else {
                continue;
            };
            let generated = go_generated_header(source);
            // The header makes a file generated; a generator may also write
            // a file without one.
            if generated && text(document, "origin") != Some("generated") {
                self.error(
                    "go:generated-origin",
                    format!("/documents/{i}/origin"),
                    format!("`{path}` has a generated-code header, so its origin is `generated`"),
                );
            }
        }
    }

    fn audit_rust_visibility_paths(&mut self) {
        let modules: BTreeSet<&'a str> = self
            .list("entities")
            .iter()
            .filter(|e| text(e, "kind") == Some("module"))
            .filter_map(|e| text(e, "path"))
            .collect();
        for (i, entity) in self.list("entities").iter().enumerate() {
            let Some(visibility) = entity.get("declared-visibility") else {
                continue;
            };
            let values: Vec<(String, &Value)> = match variant_list(visibility) {
                Some(entries) => entries
                    .iter()
                    .enumerate()
                    .filter_map(|(k, e)| {
                        Some((
                            format!("/entities/{i}/declared-visibility/{k}/value"),
                            e.get("value")?,
                        ))
                    })
                    .collect(),
                None => vec![(format!("/entities/{i}/declared-visibility"), visibility)],
            };
            for (at, value) in values {
                let Some(declared) = text(value, "value") else {
                    continue;
                };
                let Some(path) = declared
                    .strip_prefix("pub(in ")
                    .and_then(|rest| rest.strip_suffix(')'))
                else {
                    continue;
                };
                if !modules.contains(path) {
                    self.error(
                        "rust:visibility-path",
                        format!("{at}/value"),
                        format!("`pub(in {path})` names no module entity"),
                    );
                }
            }
        }
    }
}

fn are(contexts: &Contexts<'_>) -> &'static str {
    if contexts.len() == 1 {
        "is"
    } else {
        "are"
    }
}

/// The contexts two context sets share, when they share any.
fn overlap<'a>(left: &Contexts<'a>, right: &Contexts<'a>) -> Option<Contexts<'a>> {
    let shared: Contexts<'a> = left.intersection(right).copied().collect();
    (!shared.is_empty()).then_some(shared)
}

/// The identity of a site: path, start and end, with coordinates read as
/// positions so that `12` and `12.0` are one site.
fn site_key(site: &Value) -> String {
    let position = |member: &str| {
        site.get(member)
            .map(|value| match Position::from_value(value) {
                Some(position) => position.to_string(),
                None => value.to_string(),
            })
            .unwrap_or_default()
    };
    format!(
        "{}|{}|{}",
        site.get("path").map(Value::to_string).unwrap_or_default(),
        position("start"),
        position("end")
    )
}

/// The access an external consumer has by one boundary value: `all` when
/// any listed value grants all code access, `subtypes` when one grants
/// derived types access, otherwise `none`.
fn external_access(boundary: &Value) -> &'static str {
    let values: Vec<&Value> = match boundary.as_array() {
        Some(items) => items.iter().collect(),
        None => vec![boundary],
    };
    let access = |value: &Value| {
        value
            .as_str()
            .or_else(|| value.get("access").and_then(Value::as_str))
            .unwrap_or_default()
            .to_owned()
    };
    if values.iter().any(|v| access(v) == "all") {
        "all"
    } else if values.iter().any(|v| access(v) == "subtypes") {
        "subtypes"
    } else {
        "none"
    }
}

/// Whether a Go file has a line matching `^// Code generated .* DO NOT
/// EDIT\.$` before its package clause. Lines inside a block comment are
/// not the package clause.
fn go_generated_header(source: &SourceText) -> bool {
    const HEAD: &str = "// Code generated ";
    const TAIL: &str = " DO NOT EDIT.";
    let mut in_block = false;
    for line in 1..=source.lines() {
        let content = source.line(line).unwrap_or_default();
        if content.len() >= HEAD.len() + TAIL.len()
            && content.starts_with(HEAD)
            && content.ends_with(TAIL)
            && !in_block
        {
            return true;
        }
        let mut rest = content.as_str();
        loop {
            if in_block {
                match rest.find("*/") {
                    Some(end) => {
                        in_block = false;
                        rest = &rest[end + 2..];
                    }
                    None => break,
                }
            } else {
                let code = rest.trim_start();
                if let Some(after) = code.strip_prefix("/*") {
                    in_block = true;
                    rest = after;
                } else if code.is_empty() || code.starts_with("//") {
                    break;
                } else {
                    // The first token outside comments is the package
                    // clause in a valid Go file.
                    return false;
                }
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn go_generated_header_precedes_the_package_clause() {
        let generated = |text: &str| go_generated_header(&SourceText::new(text));
        assert!(generated(
            "// Code generated by stringer. DO NOT EDIT.\n\npackage a\n"
        ));
        assert!(generated(
            "//go:build linux\r\n// Code generated  DO NOT EDIT.\r\npackage a\r\n"
        ));
        assert!(!generated(
            "package a\n\n// Code generated by x. DO NOT EDIT.\n"
        ));
        assert!(!generated(
            "// Code generated by x. DO NOT EDIT\npackage a\n"
        ));
        assert!(!generated(
            "// Code generated by x. DO NOT EDIT. \npackage a\n"
        ));
        assert!(!generated(
            "//Code generated by x. DO NOT EDIT.\npackage a\n"
        ));
    }

    #[test]
    fn pointers_compare_array_indexes_numerically() {
        assert_eq!(compare_pointers("/a/2", "/a/10"), Ordering::Less);
        assert_eq!(compare_pointers("/a", "/a/0"), Ordering::Less);
        assert_eq!(compare_pointers("/b", "/a/0"), Ordering::Greater);
    }

    #[test]
    fn external_access_takes_the_widest_listed_value() {
        assert_eq!(external_access(&json!("all")), "all");
        assert_eq!(
            external_access(&json!(["none", {"access": "subtypes", "within": ["x"]}])),
            "subtypes"
        );
        assert_eq!(
            external_access(&json!({"access": "scoped", "scope": ["x"]})),
            "none"
        );
        assert_eq!(external_access(&json!(["subtypes", "all"])), "all");
    }
}

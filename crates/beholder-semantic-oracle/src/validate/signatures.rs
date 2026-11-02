//! Signature checks: model validation criterion 8.
//!
//! Each check names one rule of the model's Signatures section, and reads the
//! slots and source declarations that section states. `model:signature-slots`
//! is the table of slots by `language-kind`; `model:signature-canonical` calls
//! the language's canonicalizer; the others compare slots with entity,
//! reference and relation facts. The `rust:reachable-unnameable` recomputation
//! reads the same slots.

use std::collections::{BTreeMap, BTreeSet};

use serde_json::Value;

use super::{list, names, overlap, text, Checker, Class, Contexts};
use crate::{
    canonical::{
        auto_trait_key, canonical_auto_trait, canonicalize, canonicalize_bounds, named_ids,
        predeclared, rust_opaque_types, value_form_issue, NameResolver,
    },
    signature::{base_key, slots, Slot, SlotClass, Source},
};

const SLOTS: &str = "model:signature-slots";
const CANONICAL: &str = "model:signature-canonical";
const PARAMETERS: &str = "model:signature-parameters";
const REFERENCES: &str = "model:signature-references";
const INFERENCE: &str = "model:signature-inference";
const BASES: &str = "model:signature-bases";

/// Names a canonical form removes from a type text, whose tokens keep their
/// `references` facts.
const REMOVED: [&str; 13] = [
    "Array",
    "ReadonlyArray",
    "Optional",
    "Union",
    "List",
    "Dict",
    "Set",
    "FrozenSet",
    "Tuple",
    "Type",
    "Nullable",
    "Required",
    "NotRequired",
];

/// A signature's slots that a member's table row has no name for: its keys, and
/// the record members that are not slots.
const NOT_SLOTS: [&str; 3] = ["entity", "contexts", "alternatives"];

struct Ids<'a, 'c>(&'c Checker<'a>);

impl NameResolver for Ids<'_, '_> {
    fn standard_path<'r>(&'r self, id: &'r str) -> Option<&'r str> {
        match self.0.ids.get(id) {
            Some((Class::External, record)) => text(record, "path"),
            _ => None,
        }
    }

    fn language_kind(&self, id: &str) -> Option<&str> {
        self.0
            .ids
            .get(id)
            .and_then(|(_, record)| text(record, "language-kind"))
    }
}

/// One record, with what the checks read about its member.
struct Record<'a> {
    at: String,
    record: &'a Value,
    id: &'a str,
    entity: &'a Value,
    contexts: Contexts<'a>,
}

impl<'a> Checker<'a> {
    fn signature_records(&self) -> &'a [Value] {
        self.dataset
            .get("public-api")
            .map(|public_api| list(public_api, "signatures"))
            .unwrap_or(&[])
    }

    fn language_kinds(&self, entity: &'a Value) -> Vec<&'a str> {
        match entity.get("language-kind") {
            Some(Value::String(kind)) => vec![kind.as_str()],
            Some(Value::Array(entries)) => entries
                .iter()
                .filter_map(|entry| text(entry, "value"))
                .collect(),
            _ => Vec::new(),
        }
    }

    fn member_contexts(&self) -> BTreeMap<&'a str, Contexts<'a>> {
        let mut members: BTreeMap<&'a str, Contexts<'a>> = BTreeMap::new();
        for member in self.public_api_members() {
            let Some(id) = text(member, "entity") else {
                continue;
            };
            let contexts = self
                .declared(member)
                .unwrap_or_else(|| match self.ids.get(id) {
                    Some((Class::Entity, _)) => self.built_contexts(id),
                    _ => self.all.clone(),
                });
            members.entry(id).or_default().extend(contexts);
        }
        members
    }

    pub(super) fn check_signatures(&mut self) {
        let members = self.member_contexts();
        let mut records = Vec::new();
        for (index, record) in self.signature_records().iter().enumerate() {
            let Some(id) = text(record, "entity") else {
                continue;
            };
            let Some((Class::Entity, entity)) = self.ids.get(id).copied() else {
                continue;
            };
            let contexts = self
                .declared(record)
                .unwrap_or_else(|| self.built_contexts(id));
            records.push(Record {
                at: format!("/public-api/signatures/{index}"),
                record,
                id,
                entity,
                contexts,
            });
        }
        self.check_signature_slots(&records, &members);
        for record in &records {
            self.check_canonical(record);
            self.check_signature_references(record);
            self.check_inference(record);
        }
        self.check_signature_parameters(&records, &members);
        self.check_signature_bases(&records, &members);
        if self.rules.language == "rust" {
            self.check_reachable_unnameable(&records, &members);
        }
    }

    // ------------------------------------------------------ signature-slots

    fn check_signature_slots(
        &mut self,
        records: &[Record<'a>],
        members: &BTreeMap<&'a str, Contexts<'a>>,
    ) {
        let mut seen: BTreeMap<&'a str, Vec<Contexts<'a>>> = BTreeMap::new();
        let mut covered: BTreeMap<&'a str, Contexts<'a>> = BTreeMap::new();
        for record in records {
            let member = members.get(record.id);
            let outside: Contexts<'a> = match member {
                Some(contexts) => record.contexts.difference(contexts).copied().collect(),
                None => record.contexts.clone(),
            };
            if !outside.is_empty() {
                self.error(
                    SLOTS,
                    format!("{}/entity", record.at),
                    format!(
                        "`{}` is no public API member in {}",
                        record.id,
                        names(&outside)
                    ),
                );
            }
            let earlier = seen.entry(record.id).or_default();
            if let Some(shared) = earlier
                .iter()
                .find_map(|theirs| overlap(theirs, &record.contexts))
            {
                let message = format!(
                    "another record states `{}` in {}; a member has one record per distinct set \
                     of contexts",
                    record.id,
                    names(&shared)
                );
                self.error(SLOTS, record.at.clone(), message);
            }
            earlier.push(record.contexts.clone());
            covered
                .entry(record.id)
                .or_default()
                .extend(record.contexts.iter().copied());
            self.check_record_slots(record);
            if self.rules.language == "rust" {
                self.check_auto_trait_places(record);
            }
        }
        // A member of a kind whose row requires a slot has a record that holds it.
        for (id, contexts) in members {
            let Some((Class::Entity, entity)) = self.ids.get(id).copied() else {
                continue;
            };
            let holding: Contexts<'a> = covered.get(id).cloned().unwrap_or_default();
            let uncovered: Contexts<'a> = contexts.difference(&holding).copied().collect();
            if uncovered.is_empty() {
                continue;
            }
            for kind in self.language_kinds(entity) {
                let Some(row) = self.rules.slots_of(kind) else {
                    continue;
                };
                if !row.required.is_empty() {
                    let message = format!(
                        "`{id}` is a `{kind}`, which has {}; no record states it in {}",
                        row.required.join(", "),
                        names(&uncovered)
                    );
                    self.error(SLOTS, "/public-api/signatures".into(), message);
                }
            }
        }
    }

    fn check_record_slots(&mut self, record: &Record<'a>) {
        let kinds = self.language_kinds(record.entity);
        let rows: Vec<_> = kinds
            .iter()
            .filter_map(|kind| self.rules.slots_of(kind))
            .collect();
        let bodies: Vec<(String, &'a Value)> = match record.record.get("alternatives") {
            Some(Value::Array(alternatives)) => alternatives
                .iter()
                .enumerate()
                .map(|(i, alternative)| (format!("{}/alternatives/{i}", record.at), alternative))
                .collect(),
            _ => vec![(record.at.clone(), record.record)],
        };
        for (at, body) in bodies {
            let Some(members) = body.as_object() else {
                continue;
            };
            for slot in members.keys() {
                if NOT_SLOTS.contains(&slot.as_str()) || slot == "at" {
                    continue;
                }
                if !rows.iter().any(|row| row.slots.contains(&slot.as_str())) {
                    let message = format!(
                        "a `{}` has no `{slot}` slot in {}",
                        kinds.join("`/`"),
                        self.rules.language
                    );
                    self.error(SLOTS, format!("{at}/{slot}"), message);
                }
            }
            for row in &rows {
                if body.get("overloads").is_some() {
                    continue;
                }
                for slot in row.required {
                    if body.get(*slot).is_none() {
                        let message = format!("a `{}` always has `{slot}`", kinds.join("`/`"));
                        self.error(SLOTS, at.clone(), message);
                    }
                }
            }
        }
    }

    /// Rust: where `auto-traits` can occur, and when an element takes an ordinal.
    fn check_auto_trait_places(&mut self, record: &Record<'a>) {
        let kind = text(record.entity, "kind").unwrap_or_default();
        let parent_is_trait = text(record.entity, "parent")
            .and_then(|parent| self.ids.get(parent))
            .is_some_and(|(_, parent)| self.language_kinds(parent).contains(&"trait"));
        if kind == "callable" && record.record.get("auto-traits").is_some() {
            let modifiers = list(record.record, "modifiers");
            let provided = modifiers.iter().any(|m| m.as_str() == Some("provided"));
            if parent_is_trait && !provided {
                self.error(
                    SLOTS,
                    format!("{}/auto-traits", record.at),
                    "a trait method with no default body has no hidden type, so no `auto-traits`"
                        .to_owned(),
                );
            }
            let is_async = modifiers.iter().any(|m| m.as_str() == Some("async"));
            let returns = text(record.record, "returns").unwrap_or_default();
            self.check_ordinals(
                &format!("{}/auto-traits", record.at),
                list(record.record, "auto-traits"),
                is_async,
                returns,
            );
        }
        if kind == "type" {
            self.check_ordinals(
                &format!("{}/auto-traits", record.at),
                list(record.record, "auto-traits"),
                false,
                "impl",
            );
        }
        for (key, entries) in [
            ("bases", list(record.record, "bases")),
            ("implementations", list(record.record, "implementations")),
        ] {
            for (i, entry) in entries.iter().enumerate() {
                for (k, binding) in list(entry, "bindings").iter().enumerate() {
                    if let Some(returns) = text(binding, "returns") {
                        self.check_ordinals(
                            &format!("{}/{key}/{i}/bindings/{k}/auto-traits", record.at),
                            list(binding, "auto-traits"),
                            false,
                            returns,
                        );
                    }
                }
            }
        }
    }

    /// A return whose whole type is its only opaque type has plain elements; any
    /// other has an ordinal below the number of its occurrences.
    fn check_ordinals(&mut self, at: &str, elements: &'a [Value], is_async: bool, returns: &str) {
        if elements.is_empty() {
            return;
        }
        let (impls, whole) = rust_opaque_types(returns);
        let occurrences = impls + usize::from(is_async);
        if occurrences == 0 {
            self.error(
                SLOTS,
                at.to_owned(),
                format!("the return `{returns}` holds no opaque type"),
            );
            return;
        }
        let plain = occurrences == 1 && (is_async || whole);
        for (i, element) in elements.iter().enumerate() {
            let element = element.as_str().unwrap_or_default();
            let ordinal = element
                .strip_prefix('#')
                .and_then(|rest| rest.split(' ').next())
                .and_then(|digits| digits.parse::<usize>().ok());
            let problem = match (plain, ordinal) {
                (true, Some(_)) => Some("a sole whole opaque return has plain elements".to_owned()),
                (false, None) => Some(format!(
                    "this return has {occurrences} opaque occurrence(s) inside it, and an element \
                     names one by `#<n> `"
                )),
                (false, Some(n)) if n >= occurrences => {
                    Some(format!("`#{n}` is past its {occurrences} occurrence(s)"))
                }
                _ => None,
            };
            if let Some(problem) = problem {
                self.error(SLOTS, format!("{at}/{i}"), problem);
            }
        }
    }

    // -------------------------------------------------- signature-canonical

    /// Type parameters a text can name bare: the record's, and those of the
    /// member and the declarations around it.
    fn parameters_in_scope(&self, record: &Record<'a>) -> BTreeSet<String> {
        let mut scope = BTreeSet::new();
        for slot in slots(record.record) {
            if slot.path.ends_with(".name") && slot.path.contains("type-parameters[") {
                scope.insert(slot.text.trim_start_matches('*').to_owned());
            }
        }
        let mut owner = Some(record.id);
        let mut guard = BTreeSet::new();
        while let Some(id) = owner {
            if !guard.insert(id) {
                break;
            }
            for entity in self.list("entities") {
                if text(entity, "parent") == Some(id)
                    && text(entity, "kind") == Some("type-parameter")
                {
                    if let Some(name) = text(entity, "name") {
                        scope.insert(name.to_owned());
                    }
                }
            }
            owner = self
                .ids
                .get(id)
                .and_then(|(_, record)| text(record, "parent"));
        }
        // A member of a generic implementation or a Python module-level `TypeVar`
        // is in scope through its declaration.
        scope
    }

    fn check_canonical(&mut self, record: &Record<'a>) {
        let language = self.rules.language;
        let scope = self.parameters_in_scope(record);
        let mut findings: Vec<(String, String)> = Vec::new();
        for slot in slots(record.record) {
            let at = format!("{}{}", record.at, slot.pointer);
            match slot.class {
                SlotClass::DeclaredType | SlotClass::InferredType => {
                    let resolver = Ids(self);
                    let found = if slot.path.ends_with("bounds") {
                        canonicalize_bounds(language, &slot.text, &resolver)
                    } else {
                        canonicalize(language, &slot.text, &resolver)
                    };
                    if found.text != slot.text {
                        findings.push((
                            at.clone(),
                            format!(
                                "`{}` is not canonical; its one spelling is `{}`",
                                slot.text, found.text
                            ),
                        ));
                    }
                    for issue in found.issues {
                        findings.push((at.clone(), format!("`{}`: {issue}", slot.text)));
                    }
                    for name in found.bare {
                        if !scope.contains(&name) {
                            findings.push((
                                at.clone(),
                                format!(
                                    "`{name}` in `{}` is neither predeclared nor a type \
                                     parameter; write an entity as `@{{id}}`",
                                    slot.text
                                ),
                            ));
                        }
                    }
                    for id in found.ids {
                        if !matches!(
                            self.ids.get(id.as_str()),
                            Some((Class::Entity | Class::External, _))
                        ) {
                            findings
                                .push((at.clone(), format!("`@{{{id}}}` resolves to no entity")));
                        }
                    }
                }
                SlotClass::Value => {
                    if let Some(issue) = value_form_issue(language, &slot.text, slot.is_default) {
                        findings.push((at.clone(), issue));
                    }
                    for id in named_ids(language, &slot.text) {
                        if !matches!(
                            self.ids.get(id.as_str()),
                            Some((Class::Entity | Class::External, _))
                        ) {
                            findings
                                .push((at.clone(), format!("`@{{{id}}}` resolves to no entity")));
                        }
                    }
                }
                SlotClass::AutoTrait => {
                    let resolver = Ids(self);
                    let canonical = canonical_auto_trait(&slot.text, &resolver);
                    if canonical != slot.text {
                        findings.push((
                            at.clone(),
                            format!(
                                "`{}` is not canonical; its one spelling is `{canonical}`",
                                slot.text
                            ),
                        ));
                    }
                }
                SlotClass::Shape => {
                    if (slot.path.ends_with(".tag") || slot.path == "tag")
                        && serde_json::from_str::<String>(&slot.text).is_err()
                    {
                        findings
                            .push((at.clone(), format!("`{}` is not a JSON string", slot.text)));
                    }
                }
            }
        }
        self.sorted_lists(record.record, &record.at, &mut findings);
        for (at, message) in findings {
            self.error(CANONICAL, at, message);
        }
    }

    /// The lists the model states are sorted.
    fn sorted_lists(&self, body: &Value, at: &str, findings: &mut Vec<(String, String)>) {
        let plain = |key: &str| -> Vec<String> {
            list(body, key)
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_owned)
                .collect()
        };
        let keyed = |key: &str, by: &dyn Fn(&Value) -> String| -> Vec<String> {
            list(body, key).iter().map(by).collect()
        };
        let lists = [
            ("modifiers", plain("modifiers")),
            ("auto-traits", plain("auto-traits")),
            ("bases", keyed("bases", &base_key)),
            ("implementations", keyed("implementations", &base_key)),
            (
                "bindings",
                keyed("bindings", &|binding| {
                    text(binding, "name").unwrap_or_default().to_owned()
                }),
            ),
            (
                "index-signatures",
                keyed("index-signatures", &|entry| {
                    list(entry, "parameters")
                        .first()
                        .and_then(|key| key.get("type"))
                        .map_or_else(String::new, |ty| {
                            ty.as_str().map_or_else(|| ty.to_string(), str::to_owned)
                        })
                }),
            ),
        ];
        for (key, items) in lists {
            if items.windows(2).any(|pair| pair[0] > pair[1]) {
                findings.push((format!("{at}/{key}"), format!("`{key}` is sorted")));
            }
            let keys: BTreeSet<&str> = items.iter().map(|item| auto_trait_key(item)).collect();
            if key == "auto-traits" && keys.len() != items.len() {
                findings.push((format!("{at}/{key}"), "one element per trait".to_owned()));
            }
        }
        // Every nested record has these lists too: an accessor, a binding, a group
        // entry, a parameter.
        let Some(members) = body.as_object() else {
            return;
        };
        for (key, member) in members {
            match member {
                Value::Object(_) if key != "at" => {
                    self.sorted_lists(member, &format!("{at}/{key}"), findings);
                }
                Value::Array(entries) => {
                    for (i, entry) in entries.iter().enumerate() {
                        if entry.is_object() {
                            self.sorted_lists(entry, &format!("{at}/{key}/{i}"), findings);
                        }
                    }
                }
                _ => {}
            }
        }
    }

    // -------------------------------------------------- signature-parameters

    fn parameter_entities(&self, owner: &str, contexts: &Contexts<'a>) -> Vec<&'a Value> {
        let listed = self.rules.listed_parameters;
        let mut found: Vec<(usize, &'a Value)> = self
            .list("entities")
            .iter()
            .enumerate()
            .filter(|(_, entity)| {
                text(entity, "parent") == Some(owner)
                    && text(entity, "kind") == Some("parameter")
                    && text(entity, "language-kind").is_some_and(|kind| listed.contains(&kind))
            })
            .filter(|(_, entity)| match text(entity, "id") {
                Some(id) => {
                    let exists = self.entity_contexts(id);
                    contexts.is_empty() || !exists.is_disjoint(contexts)
                }
                None => false,
            })
            .collect();
        // Sites give source order. A synthetic parameter has none, so its place
        // in `entities` does.
        found.sort_by_key(|(place, entity)| {
            let site = list(entity, "definitions")
                .first()
                .and_then(|d| d.get("at"));
            (site.map(site_key), *place)
        });
        found.into_iter().map(|(_, entity)| entity).collect()
    }

    fn check_signature_parameters(
        &mut self,
        records: &[Record<'a>],
        members: &BTreeMap<&'a str, Contexts<'a>>,
    ) {
        let mut stated: BTreeSet<&'a str> = BTreeSet::new();
        for record in records {
            stated.insert(record.id);
            let bodies: Vec<(String, &'a Value, Option<&'a Value>)> =
                match record.record.get("alternatives") {
                    Some(Value::Array(alternatives)) => alternatives
                        .iter()
                        .enumerate()
                        .map(|(i, alternative)| {
                            (
                                format!("{}/alternatives/{i}", record.at),
                                alternative,
                                alternative.get("at"),
                            )
                        })
                        .collect(),
                    _ => vec![(record.at.clone(), record.record, None)],
                };
            for (at, body, site) in bodies {
                if let Some(overloads) = body.get("overloads").and_then(Value::as_array) {
                    let sites = definition_sites(record.entity);
                    for (n, overload) in overloads.iter().enumerate() {
                        let range = (sites.get(n).copied(), sites.get(n + 1).copied());
                        self.compare_parameters(
                            &format!("{at}/overloads/{n}"),
                            record.id,
                            overload,
                            &record.contexts,
                            Some(range),
                        );
                    }
                    continue;
                }
                let range = site.map(|site| {
                    let sites = definition_sites(record.entity);
                    let key = site_key(site);
                    let next = sites.iter().copied().find(|other| *other > key);
                    (Some(key), next)
                });
                self.compare_parameters(&at, record.id, body, &record.contexts, range);
                self.check_group_parameters(&at, record, body);
            }
        }
        // A callable with parameter entities has a record that names them.
        for (id, contexts) in members {
            if stated.contains(id) {
                continue;
            }
            let Some((Class::Entity, entity)) = self.ids.get(id).copied() else {
                continue;
            };
            if text(entity, "kind") != Some("callable") {
                continue;
            }
            let unlisted = self.parameter_entities(id, contexts);
            if let Some(first) = unlisted.first().and_then(|entity| text(entity, "name")) {
                self.error(
                    PARAMETERS,
                    "/public-api/signatures".into(),
                    format!("`{id}` has parameter `{first}`, and no record lists it"),
                );
            }
        }
    }

    fn check_group_parameters(&mut self, at: &str, record: &Record<'a>, body: &'a Value) {
        for (group, kind) in [
            ("constructors", "constructor"),
            ("call-signatures", "call-signature"),
            ("construct-signatures", "construct-signature"),
            ("index-signatures", "index-signature"),
        ] {
            let children: Vec<&'a str> = self
                .list("entities")
                .iter()
                .filter(|entity| {
                    text(entity, "parent") == Some(record.id)
                        && self.language_kinds(entity).contains(&kind)
                })
                .filter_map(|entity| text(entity, "id"))
                .collect();
            for (n, entry) in list(body, group).iter().enumerate() {
                if let Some(child) = children.get(n) {
                    self.compare_parameters(
                        &format!("{at}/{group}/{n}"),
                        child,
                        entry,
                        &record.contexts,
                        None,
                    );
                }
            }
        }
    }

    fn compare_parameters(
        &mut self,
        at: &str,
        owner: &str,
        body: &'a Value,
        contexts: &Contexts<'a>,
        range: Option<(Option<SiteKey<'a>>, Option<SiteKey<'a>>)>,
    ) {
        let written: Vec<&str> = list(body, "parameters")
            .iter()
            .filter_map(|parameter| text(parameter, "name"))
            .collect();
        let origin = self
            .ids
            .get(owner)
            .and_then(|(_, entity)| text(entity, "origin"))
            .unwrap_or("source");
        let found = self.parameter_entities(owner, contexts);
        let in_range = |entity: &&'a Value| match range {
            None => true,
            Some((start, end)) => list(entity, "definitions").iter().any(|definition| {
                definition.get("at").is_some_and(|site| {
                    let key = site_key(site);
                    start.is_none_or(|start| key >= start) && end.is_none_or(|end| key < end)
                })
            }),
        };
        let expected: Vec<&str> = found
            .iter()
            .copied()
            .filter(in_range)
            .filter_map(|entity| text(entity, "name"))
            .collect();
        // A generated callable's parameters can have no entities.
        if expected.is_empty() && origin != "source" {
            return;
        }
        if written != expected {
            self.error(
                PARAMETERS,
                format!("{at}/parameters"),
                format!(
                    "the parameter list is {written:?}; the `parameter` entities of its source \
                     declaration are {expected:?}"
                ),
            );
        }
    }

    // -------------------------------------------------- signature-references

    fn is_inside(&self, from: &str, root: &str) -> bool {
        let mut current = Some(from);
        let mut guard = BTreeSet::new();
        while let Some(id) = current {
            if id == root {
                return true;
            }
            if !guard.insert(id) {
                return false;
            }
            current = self.ids.get(id).and_then(|(_, r)| text(r, "parent"));
        }
        false
    }

    /// The site range `[nth site, next site)` of a member with several signature
    /// sites, and whether a reference's site lies in it.
    fn in_site_range(&self, reference: &Value, sites: &[SiteKey<'a>], start: SiteKey<'a>) -> bool {
        let Some(site) = reference.get("at") else {
            return false;
        };
        let key = site_key(site);
        let end = sites.iter().copied().find(|other| *other > start);
        key >= start && end.is_none_or(|end| key < end)
    }

    /// The entities of a type's signature group, in source order. An entity can
    /// declare several signatures of a group by its sites.
    fn group_children(&self, owner: &str, group: &str) -> Vec<&'a Value> {
        let kind = match group {
            "constructors" => "constructor",
            "call-signatures" => "call-signature",
            "construct-signatures" => "construct-signature",
            _ => "index-signature",
        };
        let mut children: Vec<&'a Value> = self
            .list("entities")
            .iter()
            .filter(|entity| {
                text(entity, "parent") == Some(owner) && self.language_kinds(entity).contains(&kind)
            })
            .collect();
        children.sort_by_key(|entity| definition_sites(entity).first().copied());
        children
    }

    /// The `trait-impl` blocks an implementation entry can be: those whose own
    /// `type` references name each entity its `type` and `self-type` name, and for
    /// a type's `bases`, the type itself.
    fn implementation_blocks(
        &self,
        member: &str,
        is_base: bool,
        entry: &Value,
        references: &[&'a Value],
    ) -> Vec<&'a str> {
        let language = self.rules.language;
        let mut needed: Vec<String> = Vec::new();
        for key in ["type", "self-type"] {
            if let Some(written) = text(entry, key) {
                needed.extend(named_ids(language, written));
            }
        }
        if is_base {
            needed.push(member.to_owned());
        }
        self.list("entities")
            .iter()
            .filter(|entity| self.language_kinds(entity).contains(&"trait-impl"))
            .filter_map(|entity| text(entity, "id"))
            .filter(|block| {
                needed.iter().all(|id| {
                    references.iter().any(|reference| {
                        text(reference, "target") == Some(id.as_str())
                            && text(reference, "from")
                                .is_some_and(|from| self.is_inside(from, block))
                    })
                })
            })
            .collect()
    }

    /// Whether a `type` reference lies in the declaration that states a slot:
    /// the model's Source declarations table.
    fn reference_states_slot(
        &self,
        reference: &Value,
        record: &Record<'a>,
        slot: &Slot<'_>,
        references: &[&'a Value],
    ) -> bool {
        let Some(from) = text(reference, "from") else {
            return false;
        };
        let inside_member = self.is_inside(from, record.id);
        if let Some(alternative) = slot.alternative {
            let site = list(record.record, "alternatives")
                .get(alternative)
                .and_then(|entry| entry.get("at"));
            return inside_member
                && site.is_some_and(|site| {
                    self.in_site_range(reference, &definition_sites(record.entity), site_key(site))
                });
        }
        match slot.source {
            Source::Member => inside_member,
            Source::Overload(n) => {
                let sites = definition_sites(record.entity);
                inside_member
                    && sites
                        .get(n)
                        .is_some_and(|site| self.in_site_range(reference, &sites, *site))
            }
            Source::Group(group, n) => {
                let children = self.group_children(record.id, group);
                match children.as_slice() {
                    // A synthetic constructor's declaration is the owning class.
                    [] => inside_member,
                    [only] if definition_sites(only).len() > 1 => {
                        let sites = definition_sites(only);
                        text(only, "id").is_some_and(|child| self.is_inside(from, child))
                            && sites
                                .get(n)
                                .is_some_and(|site| self.in_site_range(reference, &sites, *site))
                    }
                    children => children
                        .get(n)
                        .and_then(|child| text(child, "id"))
                        .is_some_and(|child| self.is_inside(from, child)),
                }
            }
            Source::Impl => {
                text(record.entity, "parent").is_some_and(|block| self.is_inside(from, block))
            }
            Source::Base | Source::Implementation => {
                let is_trait = self.language_kinds(record.entity).contains(&"trait");
                let is_base = slot.source == Source::Base;
                // Only a Rust implementation has a block; another language's base
                // is in the member's own declaration, as is a Rust supertrait.
                if self.rules.language != "rust" || (is_base && is_trait) {
                    return inside_member;
                }
                let Some(entry) = entry_of(record.record, &slot.pointer) else {
                    return false;
                };
                self.implementation_blocks(record.id, is_base, entry, references)
                    .iter()
                    .any(|block| self.is_inside(from, block))
            }
        }
    }

    /// The module-level type parameter a `type-parameters` entry names, such as
    /// a Python `TypeVar`, whose declaration states the entry.
    fn entry_declaration(&self, record: &Record<'a>, slot: &Slot<'_>) -> Option<&'a str> {
        let (index, _) = slot
            .path
            .strip_prefix("type-parameters[")?
            .split_once(']')?;
        let index: usize = index.parse().ok()?;
        let name = list(record.record, "type-parameters")
            .get(index)
            .and_then(|entry| text(entry, "name"))?;
        let unit = text(record.entity, "unit");
        self.list("entities")
            .iter()
            .find(|entity| {
                text(entity, "kind") == Some("type-parameter")
                    && text(entity, "name") == Some(name)
                    && text(entity, "language-kind") == Some("typevar")
                    && text(entity, "unit") == unit
            })
            .and_then(|entity| text(entity, "id"))
    }

    fn check_signature_references(&mut self, record: &Record<'a>) {
        let language = self.rules.language;
        let references: Vec<&'a Value> = self
            .list("references")
            .iter()
            .filter(|reference| text(reference, "role") == Some("type"))
            .filter(|reference| match self.declared(reference) {
                Some(contexts) => !contexts.is_disjoint(&record.contexts),
                None => true,
            })
            .collect();
        let structural = self.structural_bases(record.id);
        let mut problems = Vec::new();
        for slot in slots(record.record) {
            if slot.class != SlotClass::DeclaredType || record_is_synthetic(record.entity) {
                continue;
            }
            // A structural base has no source token, so its entry has no source
            // declaration to read.
            if slot.source == Source::Base {
                let head = entry_of(record.record, &slot.pointer)
                    .and_then(|entry| text(entry, "type"))
                    .map(|written| named_ids(language, written))
                    .and_then(|ids| ids.into_iter().next());
                if head.is_some_and(|id| structural.contains(id.as_str())) {
                    continue;
                }
            }
            let resolver = Ids(self);
            let ids = canonicalize(language, &slot.text, &resolver).ids;
            let declaration = self.entry_declaration(record, &slot);
            for id in ids {
                let stated = references.iter().any(|reference| {
                    text(reference, "target") == Some(id.as_str())
                        && (declaration.is_some_and(|declared| {
                            text(reference, "from")
                                .is_some_and(|from| self.is_inside(from, declared))
                        }) || self.reference_states_slot(reference, record, &slot, &references))
                });
                if !stated {
                    problems.push((
                        format!("{}{}", record.at, slot.pointer),
                        format!(
                            "`@{{{id}}}` has no `type` reference from the slot's source \
                             declaration"
                        ),
                    ));
                }
            }
        }
        // Each `type` reference inside a parameter has its `@{id}`.
        for reference in &references {
            let Some(from) = text(reference, "from") else {
                continue;
            };
            let Some((_, parameter)) = self.ids.get(from).copied() else {
                continue;
            };
            if text(parameter, "kind") != Some("parameter")
                || text(parameter, "parent") != Some(record.id)
            {
                continue;
            }
            let Some(name) = text(parameter, "name") else {
                continue;
            };
            let slot = list(record.record, "parameters")
                .iter()
                .find(|written| text(written, "name") == Some(name))
                .and_then(|written| written.get("type"));
            let Some(ty) = slot.and_then(Value::as_str) else {
                continue;
            };
            let Some(target) = text(reference, "target") else {
                continue;
            };
            if self.reference_token_is_removed(target) {
                continue;
            }
            let resolver = Ids(self);
            if !canonicalize(language, ty, &resolver)
                .ids
                .iter()
                .any(|id| id == target)
            {
                problems.push((
                    format!("{}/parameters", record.at),
                    format!(
                        "the `type` reference to `{target}` inside parameter `{name}` has no \
                         `@{{id}}` in `{ty}`"
                    ),
                ));
            }
        }
        for (at, message) in problems {
            self.error(REFERENCES, at, message);
        }
    }

    /// Whether a canonical form writes this target's token bare or removes it.
    fn reference_token_is_removed(&self, target: &str) -> bool {
        let path = self
            .ids
            .get(target)
            .and_then(|(_, record)| text(record, "path"))
            .unwrap_or(target);
        let last = path.rsplit(['.', ':', '#', '/']).next().unwrap_or(path);
        let lower = last.to_ascii_lowercase();
        predeclared(self.rules.language).contains(&last)
            || predeclared(self.rules.language).contains(&lower.as_str())
            || REMOVED.contains(&last)
            || (self.rules.language == "csharp" && path.starts_with("System."))
    }

    /// Targets of a member's relations that no type token establishes: a structural
    /// relation has no `at`, and a derive's `at` is the derive's name.
    fn structural_bases(&self, id: &str) -> BTreeSet<&'a str> {
        let type_sites: BTreeSet<SiteKey<'a>> = self
            .list("references")
            .iter()
            .filter(|reference| text(reference, "role") == Some("type"))
            .filter_map(|reference| reference.get("at"))
            .map(site_key)
            .collect();
        self.list("relations")
            .iter()
            .filter(|relation| {
                matches!(text(relation, "kind"), Some("inherits" | "implements"))
                    && text(relation, "from") == Some(id)
                    && relation
                        .get("at")
                        .is_none_or(|at| !type_sites.contains(&site_key(at)))
            })
            .filter_map(|relation| text(relation, "to"))
            .collect()
    }

    // --------------------------------------------------- signature-inference

    fn check_inference(&mut self, record: &Record<'a>) {
        let origin = text(record.entity, "origin").unwrap_or("source");
        let kinds = self.language_kinds(record.entity);
        let declaration_file = list(record.entity, "definitions")
            .first()
            .and_then(|definition| definition.get("at"))
            .and_then(|at| text(at, "path"))
            .unwrap_or_default();
        let declaration_only = declaration_file.ends_with(".d.ts")
            || declaration_file.ends_with(".d.mts")
            || declaration_file.ends_with(".d.cts");
        let allowed = kinds
            .iter()
            .any(|kind| self.rules.inferred_on_source.contains(kind))
            && !declaration_only;
        let mut problems = Vec::new();
        for slot in slots(record.record) {
            let at = format!("{}{}", record.at, slot.pointer);
            let is_key =
                slot.path.starts_with("bases[") || slot.path.starts_with("implementations[");
            match (origin, slot.inferred) {
                ("source", true) if !allowed => problems.push((
                    at,
                    format!(
                        "a source `{}` in {} states this slot's type; {{inferred}} is for what \
                         the profile's inference rule gives",
                        kinds.join("`/`"),
                        self.rules.language
                    ),
                )),
                ("synthetic", false)
                    if !is_key
                        && matches!(slot.class, SlotClass::DeclaredType | SlotClass::Value) =>
                {
                    problems.push((
                        at,
                        "each type and value slot of a synthetic member is {inferred}".to_owned(),
                    ));
                }
                _ => {}
            }
        }
        for (at, message) in problems {
            self.error(INFERENCE, at, message);
        }
    }

    // ------------------------------------------------------- signature-bases

    fn relation_targets(&self, id: &str, contexts: &Contexts<'a>) -> BTreeSet<&'a str> {
        self.list("relations")
            .iter()
            .filter(|relation| {
                matches!(text(relation, "kind"), Some("inherits" | "implements"))
                    && text(relation, "from") == Some(id)
            })
            .filter(|relation| match self.declared(relation) {
                Some(theirs) => contexts.is_empty() || !theirs.is_disjoint(contexts),
                None => true,
            })
            .filter_map(|relation| text(relation, "to"))
            .collect()
    }

    /// The head of a base's type text: an id, or a bare name a target's path ends in.
    fn base_head(&self, ty: &str, targets: &BTreeSet<&'a str>) -> Option<String> {
        if let Some(id) = named_ids(self.rules.language, ty).into_iter().next() {
            return Some(id);
        }
        let head: String = ty
            .chars()
            .take_while(|c| c.is_alphanumeric() || *c == '_')
            .collect();
        targets
            .iter()
            .find(|target| {
                let path = self
                    .ids
                    .get(**target)
                    .and_then(|(_, record)| text(record, "path"))
                    .unwrap_or(target);
                path.rsplit(['.', ':', '#', '/']).next() == Some(head.as_str())
            })
            .map(|target| (*target).to_owned())
            .or(Some(head))
    }

    fn check_signature_bases(
        &mut self,
        records: &[Record<'a>],
        members: &BTreeMap<&'a str, Contexts<'a>>,
    ) {
        let mut stated: BTreeSet<&'a str> = BTreeSet::new();
        for record in records {
            stated.insert(record.id);
            let targets = self.relation_targets(record.id, &record.contexts);
            let bases: BTreeSet<String> = list(record.record, "bases")
                .iter()
                .filter_map(|base| text(base, "type"))
                .filter_map(|ty| self.base_head(ty, &targets))
                .collect();
            let targets: BTreeSet<String> = targets.iter().map(|t| (*t).to_owned()).collect();
            for missing in targets.difference(&bases) {
                self.error(
                    BASES,
                    format!("{}/bases", record.at),
                    format!("the relation to `{missing}` has no `bases` entry"),
                );
            }
            for extra in bases.difference(&targets) {
                self.error(
                    BASES,
                    format!("{}/bases", record.at),
                    format!("`{extra}` is in `bases`, and no relation targets it"),
                );
            }
        }
        for (id, contexts) in members {
            if stated.contains(id) {
                continue;
            }
            let Some((Class::Entity, _)) = self.ids.get(id) else {
                continue;
            };
            if let Some(target) = self.relation_targets(id, contexts).first() {
                self.error(
                    BASES,
                    "/public-api/signatures".into(),
                    format!(
                        "`{id}` has a relation to `{target}`, and no record states its `bases`"
                    ),
                );
            }
        }
    }

    // ----------------------------------------------- rust:reachable-unnameable

    fn check_reachable_unnameable(
        &mut self,
        records: &[Record<'a>],
        members: &BTreeMap<&'a str, Contexts<'a>>,
    ) {
        let rule = "rust:reachable-unnameable";
        let language = self.rules.language;
        let distributions = self.member_distributions();
        // (target, distribution) -> the members whose signatures expose it.
        let mut exposed: BTreeMap<(String, &'a str), BTreeSet<&'a str>> = BTreeMap::new();
        for record in records {
            let Some(distribution) = distributions.get(record.id).copied() else {
                continue;
            };
            for slot in slots(record.record) {
                if !matches!(
                    slot.class,
                    SlotClass::DeclaredType | SlotClass::InferredType
                ) {
                    continue;
                }
                for id in named_ids(language, &slot.text) {
                    let Some((Class::Entity, target)) = self.ids.get(id.as_str()).copied() else {
                        continue;
                    };
                    let nominal = matches!(text(target, "kind"), Some("type" | "type-alias"))
                        && text(target, "parent")
                            .and_then(|parent| self.ids.get(parent))
                            .is_none_or(|(_, parent)| text(parent, "kind") == Some("module"));
                    let named = members
                        .get(id.as_str())
                        .is_some_and(|contexts| !contexts.is_disjoint(&record.contexts));
                    if nominal && !named {
                        exposed
                            .entry((id, distribution))
                            .or_default()
                            .insert(record.id);
                    }
                }
            }
        }
        let facts = self
            .dataset
            .get("extensions")
            .map(|extensions| list(extensions, rule))
            .unwrap_or(&[]);
        let mut seen: BTreeSet<(String, String)> = BTreeSet::new();
        for (i, fact) in facts.iter().enumerate() {
            let (Some(entity), Some(distribution)) =
                (text(fact, "entity"), text(fact, "distribution"))
            else {
                continue;
            };
            seen.insert((entity.to_owned(), distribution.to_owned()));
            let expected = exposed.get(&(entity.to_owned(), distribution));
            let want: Vec<&str> = expected
                .map(|via| via.iter().copied().collect())
                .unwrap_or_default();
            let have: Vec<&str> = list(fact, "via").iter().filter_map(Value::as_str).collect();
            if want != have {
                self.error(
                    rule,
                    format!("/extensions/{rule}/{i}/via"),
                    format!(
                        "`via` is {have:?}; the signatures that expose `{entity}` are {want:?}"
                    ),
                );
            }
        }
        for ((entity, distribution), via) in exposed {
            if !seen.contains(&(entity.clone(), distribution.to_owned())) {
                let via: Vec<&str> = via.into_iter().collect();
                self.error(
                    rule,
                    format!("/extensions/{rule}"),
                    format!(
                        "the signatures of {via:?} expose `{entity}`, which has no consumer path \
                         in `{distribution}`; no fact states it"
                    ),
                );
            }
        }
    }

    fn member_distributions(&self) -> BTreeMap<&'a str, &'a str> {
        self.public_api_members()
            .iter()
            .filter_map(|member| Some((text(member, "entity")?, text(member, "distribution")?)))
            .collect()
    }
}

fn record_is_synthetic(entity: &Value) -> bool {
    text(entity, "origin") == Some("synthetic")
}

/// The `bases` or `implementations` entry a slot's pointer lies in.
fn entry_of<'r>(record: &'r Value, pointer: &str) -> Option<&'r Value> {
    let mut parts = pointer.trim_start_matches('/').split('/');
    let key = parts.next()?;
    let index: usize = parts.next()?.parse().ok()?;
    list(record, key).get(index)
}

type SiteKey<'a> = (&'a str, u64, u64);

fn site_key(site: &Value) -> SiteKey<'_> {
    let position = |key: &str, index: usize| {
        site.get(key)
            .and_then(|position| position.get(index))
            .and_then(Value::as_u64)
            .unwrap_or(0)
    };
    (
        text(site, "path").unwrap_or_default(),
        position("start", 0),
        position("start", 1),
    )
}

fn definition_sites(entity: &Value) -> Vec<SiteKey<'_>> {
    let mut sites: Vec<SiteKey<'_>> = list(entity, "definitions")
        .iter()
        .filter_map(|definition| definition.get("at"))
        .map(site_key)
        .collect();
    sites.sort_unstable();
    sites
}

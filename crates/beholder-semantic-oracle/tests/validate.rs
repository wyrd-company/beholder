// ---
// relationships:
//   references: semantic-oracle-model
// ---
//! Validation and audit rules against a small generic fixture project.
//!
//! `tests/fixtures/tally` is a one-file Rust library with a complete
//! dataset. Every test changes one thing in the dataset or the project
//! files and asserts the exact findings: each rule fires at its location and
//! nothing else fires.

use std::path::Path;

use beholder_semantic_oracle::{
    audit, parse_yaml,
    project::{sha256_hex, MemoryProject},
    validate, Report, Severity,
};
use serde_json::{json, Value};

const FILES: [&str; 3] = ["Cargo.toml", "README.md", "src/lib.rs"];

fn fixture_dir() -> &'static Path {
    Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/tally"))
}

fn dataset() -> Value {
    let text = std::fs::read_to_string(fixture_dir().join("semantic-oracle.yml")).unwrap();
    parse_yaml(&text).unwrap()
}

fn files() -> MemoryProject {
    FILES.iter().fold(MemoryProject::new(), |project, path| {
        project.with(path, std::fs::read(fixture_dir().join(path)).unwrap())
    })
}

/// Sets the member at `pointer`, adding it to its object when absent.
fn set(dataset: &mut Value, pointer: &str, value: Value) {
    let (parent, key) = pointer.rsplit_once('/').unwrap();
    let parent = dataset
        .pointer_mut(parent)
        .unwrap_or_else(|| panic!("no {parent}"));
    match parent {
        Value::Object(members) => {
            members.insert(key.to_owned(), value);
        }
        Value::Array(items) => items[key.parse::<usize>().unwrap()] = value,
        _ => panic!("{pointer} is inside a scalar"),
    }
}

fn push(dataset: &mut Value, pointer: &str, value: Value) {
    dataset
        .pointer_mut(pointer)
        .and_then(Value::as_array_mut)
        .unwrap_or_else(|| panic!("no list {pointer}"))
        .push(value);
}

/// `(severity, rule, at)` of every finding.
fn found(report: &Report) -> Vec<(&'static str, String, String)> {
    report
        .findings
        .iter()
        .map(|f| {
            let severity = match f.severity {
                Severity::Error => "error",
                Severity::Warning => "warning",
            };
            (severity, f.rule.clone(), f.at.clone())
        })
        .collect()
}

fn expect(report: &Report, expected: &[(&'static str, &str, &str)]) {
    let expected: Vec<(&str, String, String)> = expected
        .iter()
        .map(|(s, r, a)| (*s, r.to_string(), a.to_string()))
        .collect();
    assert_eq!(found(report), expected, "{:#?}", report.findings);
}

fn validate_with(dataset: &Value, files: &MemoryProject) -> Report {
    validate(dataset, Some(files))
}

#[test]
fn the_fixture_passes_validation_and_audit() {
    let dataset = dataset();
    expect(&validate_with(&dataset, &files()), &[]);
    expect(&audit(&dataset, Some(&files())), &[]);
    expect(&validate(&dataset, None), &[]);
}

#[test]
fn findings_are_deterministic() {
    let mut dataset = dataset();
    set(&mut dataset, "/references/1/target", json!("nowhere"));
    set(
        &mut dataset,
        "/entities/1/definitions/0/at/text",
        json!("Count"),
    );
    set(&mut dataset, "/documents/0/sha256", json!("0".repeat(64)));
    let first = audit(&dataset, Some(&files()));
    let second = audit(&dataset, Some(&files()));
    assert_eq!(first, second);
    let ats: Vec<&str> = first.findings.iter().map(|f| f.at.as_str()).collect();
    assert_eq!(
        ats,
        [
            "/documents/0/sha256",
            "/entities/1/definitions/0/at",
            "/references/1/target"
        ]
    );
}

// ------------------------------------------------------------ versions

#[test]
fn another_model_version_is_one_finding() {
    let mut dataset = dataset();
    set(&mut dataset, "/model", json!("semantic-oracle/2"));
    expect(
        &validate_with(&dataset, &files()),
        &[("error", "model:version", "/model")],
    );
}

#[test]
fn an_unknown_profile_version_is_one_finding() {
    let mut dataset = dataset();
    set(&mut dataset, "/profile", json!("rust/2"));
    expect(
        &validate_with(&dataset, &files()),
        &[("error", "model:profile-version", "/profile")],
    );
}

#[test]
fn schema_errors_stop_later_checks() {
    let mut dataset = dataset();
    dataset.as_object_mut().unwrap().remove("relations");
    set(&mut dataset, "/references/0/target", json!("nowhere"));
    expect(
        &validate_with(&dataset, &files()),
        &[("error", "schema:required", "")],
    );
}

// ------------------------------------------------------------------ ids

#[test]
fn ids_are_unique_across_the_dataset() {
    let mut dataset = dataset();
    set(&mut dataset, "/distributions/0/id", json!("tally-lib"));
    set(&mut dataset, "/units/0/distribution", json!("tally-lib"));
    expect(
        &validate_with(&dataset, &files()),
        &[
            ("error", "model:id-unique", "/distributions/0/id"),
            ("error", "model:id-reference", "/public-api/0/distribution"),
            ("error", "model:id-reference", "/public-api/1/distribution"),
            ("error", "model:id-reference", "/units/0/distribution"),
        ],
    );
}

#[test]
fn id_references_resolve_to_the_right_kind_of_record() {
    let mut dataset = dataset();
    set(&mut dataset, "/references/0/target", json!("tally-lib"));
    set(&mut dataset, "/references/1/from", json!("missing"));
    expect(
        &validate_with(&dataset, &files()),
        &[
            ("error", "model:id-reference", "/references/0/target"),
            ("error", "model:id-reference", "/references/1/from"),
        ],
    );
}

#[test]
fn a_disposition_names_the_kind_of_its_target() {
    let mut dataset = dataset();
    set(&mut dataset, "/references/0/disposition", json!("external"));
    expect(
        &validate_with(&dataset, &files()),
        &[("error", "model:id-reference", "/references/0/target")],
    );
}

#[test]
fn every_external_is_a_target() {
    let mut dataset = dataset();
    push(
        &mut dataset,
        "/externals",
        json!({"id": "ext.core.fmt.Debug", "kind": "type", "language-kind": "trait",
               "distribution": "core", "path": "core::fmt::Debug"}),
    );
    expect(
        &validate_with(&dataset, &files()),
        &[("error", "model:external-unused", "/externals/0")],
    );
}

#[test]
fn external_keys_are_unique() {
    let mut dataset = dataset();
    for id in ["ext.a", "ext.b"] {
        push(
            &mut dataset,
            "/externals",
            json!({"id": id, "kind": "type", "language-kind": "trait",
                   "distribution": "core", "path": "core::fmt::Debug"}),
        );
        push(
            &mut dataset,
            "/relations",
            json!({"kind": "implements", "from": "tally.Counter", "to": id}),
        );
    }
    expect(
        &validate_with(&dataset, &files()),
        &[("error", "model:external-key", "/externals/1")],
    );
}

// ------------------------------------------------------------ documents

#[test]
fn every_tracked_file_has_one_inventory_entry() {
    let mut dataset = dataset();
    let files = files().with("LICENSE", "text\n");
    push(
        &mut dataset,
        "/documents",
        json!({"path": "docs/gone.md", "sha256": "0".repeat(64),
               "disposition": "out-of-profile", "reason": "documentation"}),
    );
    expect(
        &validate_with(&dataset, &files),
        &[
            ("error", "model:document-inventory", "/documents"),
            ("error", "model:document-inventory", "/documents/3/path"),
        ],
    );
}

#[test]
fn the_dataset_file_is_not_a_document() {
    let mut dataset = dataset();
    push(
        &mut dataset,
        "/documents",
        json!({"path": "semantic-oracle.yml", "sha256": "0".repeat(64),
               "disposition": "out-of-profile", "reason": "the dataset"}),
    );
    let files = files().with("semantic-oracle.yml", "model: x\n");
    expect(
        &validate_with(&dataset, &files),
        &[("error", "model:document-inventory", "/documents/3/path")],
    );
}

#[test]
fn a_path_has_one_inventory_entry() {
    let mut dataset = dataset();
    let copy = dataset["documents"][1].clone();
    push(&mut dataset, "/documents", copy);
    expect(
        &validate_with(&dataset, &files()),
        &[("error", "model:document-unique", "/documents/3/path")],
    );
}

#[test]
fn a_digest_mismatch_outside_indexed_documents_is_reported() {
    let files = files().with("README.md", "# tally, changed\n");
    let report = validate_with(&dataset(), &files);
    expect(
        &report,
        &[("error", "model:document-digest", "/documents/1/sha256")],
    );
    assert!(report.findings[0]
        .message
        .contains(&sha256_hex(b"# tally, changed\n")));
}

#[test]
fn a_changed_indexed_document_makes_its_locators_stale() {
    let files = files().with(
        "src/lib.rs",
        "pub struct Counter;\n\n\npub fn make() -> Counter {\n    Counter\n}\n",
    );
    let report = validate_with(&dataset(), &files);
    expect(
        &report,
        &[("error", "model:locator-stale", "/documents/2/sha256")],
    );
    assert!(
        report.findings[0].message.contains("4 locator(s)"),
        "{}",
        report.findings[0].message
    );
}

#[test]
fn an_indexed_document_is_utf8() {
    let mut dataset = dataset();
    let bytes = b"pub struct Counter;\xff\n".to_vec();
    set(
        &mut dataset,
        "/documents/2/sha256",
        json!(sha256_hex(&bytes)),
    );
    let files = files().with("src/lib.rs", bytes);
    let report = validate_with(&dataset, &files);
    expect(
        &report,
        &[("error", "model:document-utf8", "/documents/2/path")],
    );
}

// ------------------------------------------------------------- locators

#[test]
fn a_locator_whose_text_differs_is_stale_and_names_where_the_text_is() {
    let mut dataset = dataset();
    set(
        &mut dataset,
        "/entities/2/definitions/0/at/start",
        json!([3, 7]),
    );
    set(
        &mut dataset,
        "/entities/2/definitions/0/at/end",
        json!([3, 11]),
    );
    let report = validate_with(&dataset, &files());
    expect(
        &report,
        &[(
            "error",
            "model:locator-stale",
            "/entities/2/definitions/0/at",
        )],
    );
    assert_eq!(
        report.findings[0].message,
        "stale: the text at these coordinates is \" mak\"; the stored text occurs at [3, 8]"
    );
}

#[test]
fn a_locator_lies_in_its_document() {
    let mut dataset = dataset();
    set(&mut dataset, "/references/1/at/end", json!([4, 40]));
    expect(
        &validate_with(&dataset, &files()),
        &[("error", "model:locator-range", "/references/1/at")],
    );
}

#[test]
fn a_locator_ends_after_it_starts() {
    let mut dataset = dataset();
    set(&mut dataset, "/references/1/at/end", json!([4, 1]));
    expect(
        &validate_with(&dataset, &files()),
        &[("error", "model:locator-order", "/references/1/at")],
    );
}

#[test]
fn a_locator_names_an_indexed_inventory_document() {
    let mut dataset = dataset();
    set(&mut dataset, "/references/0/at/path", json!("src/other.rs"));
    set(&mut dataset, "/references/1/at/path", json!("README.md"));
    expect(
        &validate_with(&dataset, &files()),
        &[
            ("error", "model:locator-document", "/references/0/at/path"),
            ("error", "model:site-membership", "/references/1/at/path"),
        ],
    );
}

#[test]
fn a_site_is_in_a_unit_member_in_every_context_of_its_fact() {
    let mut dataset = dataset();
    let mut test = dataset["contexts"][0].clone();
    test["id"] = json!("test");
    test["parameters"]["mode"] = json!("test");
    push(&mut dataset, "/contexts", test);
    set(
        &mut dataset,
        "/documents/2/members/0/contexts",
        json!(["default"]),
    );
    expect(
        &validate_with(&dataset, &files()),
        &[
            (
                "error",
                "model:site-membership",
                "/entities/1/definitions/0/at/path",
            ),
            (
                "error",
                "model:site-membership",
                "/entities/2/definitions/0/at/path",
            ),
            ("error", "model:site-membership", "/references/0/at/path"),
            ("error", "model:site-membership", "/references/1/at/path"),
        ],
    );
}

#[test]
fn a_reference_site_has_one_fact_per_from_and_role() {
    let mut dataset = dataset();
    let copy = dataset["references"][1].clone();
    push(&mut dataset, "/references", copy);
    let mut other_role = dataset["references"][1].clone();
    other_role["role"] = json!("read");
    push(&mut dataset, "/references", other_role);
    expect(
        &validate_with(&dataset, &files()),
        &[("error", "model:locator-ambiguous", "/references/2/at")],
    );
}

#[test]
fn an_entity_lists_a_definition_site_once() {
    let mut dataset = dataset();
    let site = dataset["entities"][1]["definitions"][0].clone();
    push(&mut dataset, "/entities/1/definitions", site);
    expect(
        &validate_with(&dataset, &files()),
        &[(
            "error",
            "model:locator-ambiguous",
            "/entities/1/definitions/1/at",
        )],
    );
}

#[test]
fn an_empty_span_is_a_warning() {
    let mut dataset = dataset();
    set(&mut dataset, "/references/1/at/end", json!([4, 5]));
    set(&mut dataset, "/references/1/at/text", json!(""));
    expect(
        &validate_with(&dataset, &files()),
        &[("warning", "model:locator-empty", "/references/1/at")],
    );
}

// ------------------------------------------------------------- contexts

#[test]
fn fact_contexts_are_declared_and_in_scope() {
    let mut dataset = dataset();
    set(&mut dataset, "/references/0/contexts", json!(["nightly"]));
    set(&mut dataset, "/units/0/contexts", json!(["default"]));
    let mut other = dataset["contexts"][0].clone();
    other["id"] = json!("release");
    other["parameters"]["profile"] = json!("release");
    push(&mut dataset, "/contexts", other);
    set(&mut dataset, "/references/1/contexts", json!(["release"]));
    expect(
        &validate_with(&dataset, &files()),
        &[
            (
                "error",
                "model:context-declared",
                "/references/0/contexts/0",
            ),
            ("error", "model:context-scope", "/references/1/contexts"),
        ],
    );
}

#[test]
fn contexts_differ_in_a_parameter() {
    let mut dataset = dataset();
    let mut copy = dataset["contexts"][0].clone();
    copy["id"] = json!("again");
    push(&mut dataset, "/contexts", copy);
    expect(
        &validate_with(&dataset, &files()),
        &[("error", "model:context-distinct", "/contexts/1/parameters")],
    );
}

#[test]
fn context_variant_lists_partition_the_record_contexts() {
    let mut dataset = dataset();
    let mut other = dataset["contexts"][0].clone();
    other["id"] = json!("release");
    other["parameters"]["profile"] = json!("release");
    push(&mut dataset, "/contexts", other);
    set(
        &mut dataset,
        "/entities/1/declared-visibility",
        json!([{"contexts": ["default"], "value": {"value": "pub", "explicit": true}}]),
    );
    set(
        &mut dataset,
        "/units/0/features",
        json!([{"contexts": ["default", "release"], "value": []},
               {"contexts": ["release"], "value": []}]),
    );
    expect(
        &validate_with(&dataset, &files()),
        &[
            (
                "error",
                "model:context-partition",
                "/entities/1/declared-visibility",
            ),
            (
                "error",
                "model:context-partition",
                "/units/0/features/1/contexts",
            ),
        ],
    );
}

#[test]
fn sorted_profile_lists_are_sorted() {
    let mut dataset = dataset();
    set(
        &mut dataset,
        "/contexts/0/parameters/cfg",
        json!(["zeta", "alpha"]),
    );
    set(&mut dataset, "/units/0/features", json!(["std", "alloc"]));
    expect(
        &validate_with(&dataset, &files()),
        &[
            ("error", "rust:sorted", "/contexts/0/parameters/cfg"),
            ("error", "rust:sorted", "/units/0/features"),
        ],
    );
}

// ------------------------------------------------------ units, entities

#[test]
fn unit_keys_are_unique_in_a_context() {
    let mut dataset = dataset();
    let mut copy = dataset["units"][0].clone();
    copy["id"] = json!("tally-lib-again");
    copy.as_object_mut().unwrap().remove("root");
    push(&mut dataset, "/units", copy);
    expect(
        &validate_with(&dataset, &files()),
        &[("error", "model:unit-key", "/units/1")],
    );
}

#[test]
fn a_unit_root_is_a_module_of_the_unit() {
    let mut dataset = dataset();
    set(&mut dataset, "/units/0/root", json!("tally.Counter"));
    expect(
        &validate_with(&dataset, &files()),
        &[("error", "model:unit-root", "/units/0/root")],
    );
}

#[test]
fn unit_path_and_kind_identify_one_entity_in_a_context() {
    let mut dataset = dataset();
    set(&mut dataset, "/entities/2/path", json!("tally::Counter"));
    set(&mut dataset, "/entities/2/kind", json!("type"));
    set(&mut dataset, "/entities/2/language-kind", json!("struct"));
    expect(
        &validate_with(&dataset, &files()),
        &[("error", "model:entity-identity", "/entities/2")],
    );
}

#[test]
fn the_parent_chain_ends() {
    let mut dataset = dataset();
    set(&mut dataset, "/entities/0/parent", json!("tally.Counter"));
    expect(
        &validate_with(&dataset, &files()),
        &[
            ("error", "model:entity-parent", "/entities/0/parent"),
            ("error", "model:entity-parent", "/entities/1/parent"),
        ],
    );
}

#[test]
fn a_generated_entity_without_sites_has_an_anchor_and_generator() {
    let mut dataset = dataset();
    set(&mut dataset, "/entities/2/origin", json!("generated"));
    set(&mut dataset, "/entities/2/definitions", json!([]));
    expect(
        &validate_with(&dataset, &files()),
        &[
            ("error", "model:entity-origin", "/entities/2"),
            ("error", "model:entity-origin", "/entities/2"),
        ],
    );
}

#[test]
fn an_entity_in_a_generated_document_is_generated() {
    let mut dataset = dataset();
    set(&mut dataset, "/documents/2/origin", json!("generated"));
    expect(
        &validate_with(&dataset, &files()),
        &[
            ("error", "model:entity-origin", "/entities/1/origin"),
            ("error", "model:entity-origin", "/entities/2/origin"),
        ],
    );
}

#[test]
fn scope_members_name_entities_units_or_documents() {
    let mut dataset = dataset();
    set(
        &mut dataset,
        "/entities/2/effective-visibility",
        json!({"unit": {"access": "scoped", "scope": ["tally.crate", "src/lib.rs", "elsewhere"]},
               "project": "none", "external": "none"}),
    );
    dataset["public-api"].as_array_mut().unwrap().pop();
    expect(
        &validate_with(&dataset, &files()),
        &[(
            "error",
            "model:id-reference",
            "/entities/2/effective-visibility/unit/scope/2",
        )],
    );
}

// --------------------------------------------- relations and public API

#[test]
fn a_relation_is_stated_once_per_context() {
    let mut dataset = dataset();
    let relation = json!({"kind": "aliases", "from": "tally.make", "to": "tally.Counter"});
    push(&mut dataset, "/relations", relation.clone());
    push(&mut dataset, "/relations", relation);
    expect(
        &validate_with(&dataset, &files()),
        &[("error", "model:relation-duplicate", "/relations/1")],
    );
}

#[test]
fn a_public_api_member_is_externally_accessible_in_its_distribution() {
    let mut dataset = dataset();
    set(
        &mut dataset,
        "/entities/2/effective-visibility/external",
        json!("none"),
    );
    set(&mut dataset, "/public-api/0/access", json!("subtypes"));
    push(
        &mut dataset,
        "/distributions",
        json!({"id": "other", "kind": "crate", "name": "other", "version": "1.0.0"}),
    );
    let mut copy = dataset["public-api"][0].clone();
    copy["distribution"] = json!("other");
    copy["access"] = json!("all");
    push(&mut dataset, "/public-api", copy);
    expect(
        &validate_with(&dataset, &files()),
        &[
            ("error", "model:public-api-access", "/public-api/0/access"),
            ("error", "model:public-api-access", "/public-api/1/access"),
            (
                "error",
                "model:public-api-distribution",
                "/public-api/2/distribution",
            ),
        ],
    );
}

#[test]
fn a_public_api_member_is_stated_once_per_context() {
    let mut dataset = dataset();
    let copy = dataset["public-api"][0].clone();
    push(&mut dataset, "/public-api", copy);
    expect(
        &validate_with(&dataset, &files()),
        &[("error", "model:public-api-duplicate", "/public-api/2")],
    );
}

// ----------------------------------------------------------- extensions

#[test]
fn extension_facts_resolve_and_have_unique_keys() {
    let mut dataset = dataset();
    let fact = json!({"entity": "tally.make", "macro": "ext.nowhere",
                      "invocation": {"path": "src/lib.rs", "start": [3, 8], "end": [3, 12], "text": "make"}});
    set(
        &mut dataset,
        "/extensions",
        json!({"rust:macro-expansion": [fact.clone(), fact],
               "rust:reachable-unnameable": [{"entity": "tally.Counter", "distribution": "tally-lib", "via": "tally.make"}]}),
    );
    expect(
        &validate_with(&dataset, &files()),
        &[
            (
                "error",
                "model:id-reference",
                "/extensions/rust:macro-expansion/0/macro",
            ),
            (
                "error",
                "model:extension-key",
                "/extensions/rust:macro-expansion/1",
            ),
            (
                "error",
                "model:id-reference",
                "/extensions/rust:macro-expansion/1/macro",
            ),
            (
                "error",
                "model:id-reference",
                "/extensions/rust:reachable-unnameable/0/distribution",
            ),
        ],
    );
}

#[test]
fn an_extension_target_counts_as_a_use_of_an_external() {
    let mut dataset = dataset();
    push(
        &mut dataset,
        "/externals",
        json!({"id": "ext.core.concat", "kind": "macro", "language-kind": "macro-rules",
               "distribution": "core", "path": "core::concat"}),
    );
    set(
        &mut dataset,
        "/extensions",
        json!({"rust:macro-expansion": [{"entity": "tally.make", "macro": "ext.core.concat",
               "invocation": {"path": "src/lib.rs", "start": [3, 8], "end": [3, 13], "text": "make("}}]}),
    );
    expect(&validate_with(&dataset, &files()), &[]);
}

// ---------------------------------------------------------------- audit

#[test]
fn the_audit_checks_document_classes() {
    let mut dataset = dataset();
    set(
        &mut dataset,
        "/documents/0/disposition",
        json!("out-of-profile"),
    );
    set(&mut dataset, "/documents/0/reason", json!("not read"));
    set(&mut dataset, "/documents/1/disposition", json!("indexed"));
    set(&mut dataset, "/documents/1/origin", json!("source"));
    set(
        &mut dataset,
        "/documents/1/members",
        json!([{"unit": "tally-lib"}]),
    );
    dataset["documents"][1]
        .as_object_mut()
        .unwrap()
        .remove("reason");
    let files = files().with("build/helper.rs", "fn main() {}\n");
    push(
        &mut dataset,
        "/documents",
        json!({"path": "build/helper.rs", "sha256": sha256_hex(b"fn main() {}\n"),
               "disposition": "out-of-profile", "reason": "unused"}),
    );
    expect(
        &audit(&dataset, Some(&files)),
        &[
            ("warning", "rust:document-class", "/documents/0/disposition"),
            ("warning", "model:coverage", "/documents/1"),
            ("error", "rust:document-class", "/documents/1/disposition"),
            ("error", "rust:document-class", "/documents/3/disposition"),
        ],
    );
}

#[test]
fn the_audit_rejects_out_of_profile_facts() {
    let mut dataset = dataset();
    push(
        &mut dataset,
        "/entities",
        json!({"id": "tally.make.local", "kind": "constant", "language-kind": "const",
               "name": "LOCAL", "path": "tally::make::LOCAL", "unit": "tally-lib",
               "parent": "tally.make", "origin": "source", "definitions": [],
               "declared-visibility": {"value": "private", "explicit": false},
               "effective-visibility": {"unit": {"access": "scoped", "scope": ["tally.make"]},
                                        "project": "none", "external": "none"}}),
    );
    push(
        &mut dataset,
        "/entities",
        json!({"id": "tally.make.T", "kind": "type-parameter", "language-kind": "type-param",
               "name": "T", "path": "tally::make::<T>", "unit": "tally-lib",
               "parent": "tally.make", "origin": "source", "definitions": []}),
    );
    set(&mut dataset, "/references/0/target", json!("tally.make.T"));
    expect(
        &audit(&dataset, Some(&files())),
        &[
            ("error", "model:out-of-profile", "/entities/3"),
            ("error", "model:out-of-profile", "/references/0/target"),
        ],
    );
}

#[test]
fn the_audit_checks_rust_restricted_visibility_paths() {
    let mut dataset = dataset();
    set(
        &mut dataset,
        "/entities/2/declared-visibility",
        json!({"value": "pub(in tally::detail)", "explicit": true}),
    );
    expect(
        &audit(&dataset, Some(&files())),
        &[(
            "error",
            "rust:visibility-path",
            "/entities/2/declared-visibility/value",
        )],
    );
    set(
        &mut dataset,
        "/entities/2/declared-visibility",
        json!({"value": "pub(in tally)", "explicit": true}),
    );
    expect(&audit(&dataset, Some(&files())), &[]);
}

#[test]
fn validation_alone_does_not_audit() {
    let mut dataset = dataset();
    set(
        &mut dataset,
        "/documents/0/disposition",
        json!("out-of-profile"),
    );
    set(&mut dataset, "/documents/0/reason", json!("not read"));
    expect(&validate_with(&dataset, &files()), &[]);
}

// ------------------------------------------------- review round 1 cases

/// The fixture with a second context, `release`, in which the unit is
/// built.
fn two_contexts() -> Value {
    let mut dataset = dataset();
    let mut release = dataset["contexts"][0].clone();
    release["id"] = json!("release");
    release["parameters"]["profile"] = json!("release");
    push(&mut dataset, "/contexts", release);
    dataset
}

#[test]
fn a_missing_or_malformed_profile_is_a_schema_error() {
    for profile in [None, Some(json!(7))] {
        let mut dataset = dataset();
        match profile.clone() {
            None => {
                dataset.as_object_mut().unwrap().remove("profile");
            }
            Some(value) => set(&mut dataset, "/profile", value),
        }
        let report = validate_with(&dataset, &files());
        assert!(!report.passes(), "{profile:?}");
        assert!(
            report
                .findings
                .iter()
                .all(|f| f.rule.starts_with("schema:")),
            "{:#?}",
            report.findings
        );
    }
    assert!(!validate(&json!({}), None).passes());
}

#[test]
fn a_unit_key_names_one_unit_across_contexts() {
    let mut dataset = two_contexts();
    set(&mut dataset, "/units/0/contexts", json!(["default"]));
    let mut copy = dataset["units"][0].clone();
    copy["id"] = json!("tally-lib-release");
    copy["contexts"] = json!(["release"]);
    copy.as_object_mut().unwrap().remove("root");
    push(&mut dataset, "/units", copy);
    expect(
        &validate_with(&dataset, &files()),
        &[("error", "model:unit-key", "/units/1")],
    );
}

#[test]
fn an_entity_without_a_distribution_has_no_external_access() {
    let mut dataset = dataset();
    dataset["units"][0]
        .as_object_mut()
        .unwrap()
        .remove("distribution");
    set(&mut dataset, "/public-api", json!([]));
    expect(
        &validate_with(&dataset, &files()),
        &[
            (
                "error",
                "model:effective-visibility",
                "/entities/0/effective-visibility",
            ),
            (
                "error",
                "model:effective-visibility",
                "/entities/1/effective-visibility",
            ),
            (
                "error",
                "model:effective-visibility",
                "/entities/2/effective-visibility",
            ),
        ],
    );
}

#[test]
fn an_omitted_fact_context_list_means_every_context_of_the_unit() {
    let mut dataset = two_contexts();
    set(&mut dataset, "/entities/2/contexts", json!(["default"]));
    set(&mut dataset, "/public-api/1/contexts", json!(["default"]));
    expect(
        &validate_with(&dataset, &files()),
        &[
            ("error", "model:context-scope", "/references/0/from"),
            ("error", "model:context-scope", "/references/1/from"),
        ],
    );
    set(&mut dataset, "/references/0/contexts", json!(["default"]));
    set(&mut dataset, "/references/1/contexts", json!(["default"]));
    expect(&validate_with(&dataset, &files()), &[]);
}

#[test]
fn a_target_exists_in_every_context_of_its_fact() {
    let mut dataset = two_contexts();
    set(&mut dataset, "/entities/1/contexts", json!(["default"]));
    set(&mut dataset, "/public-api/0/contexts", json!(["default"]));
    push(
        &mut dataset,
        "/relations",
        json!({"kind": "aliases", "from": "tally.make", "to": "tally.Counter"}),
    );
    expect(
        &validate_with(&dataset, &files()),
        &[
            ("error", "model:context-scope", "/references/0/target"),
            ("error", "model:context-scope", "/references/1/target"),
            ("error", "model:context-scope", "/relations/0/to"),
        ],
    );
}

#[test]
fn an_unknown_distribution_is_one_finding() {
    let mut dataset = dataset();
    set(&mut dataset, "/public-api/0/distribution", json!("missing"));
    expect(
        &validate_with(&dataset, &files()),
        &[("error", "model:id-reference", "/public-api/0/distribution")],
    );
}

#[test]
fn an_extension_fact_holds_in_the_contexts_of_its_owner_unit() {
    let mut dataset = two_contexts();
    set(&mut dataset, "/units/0/contexts", json!(["default"]));
    set(
        &mut dataset,
        "/extensions",
        json!({"rust:macro-expansion": [{"entity": "tally.Counter", "macro": "tally.make",
               "invocation": {"path": "src/lib.rs", "start": [1, 12], "end": [1, 19], "text": "Counter"}}]}),
    );
    expect(&validate_with(&dataset, &files()), &[]);
    set(
        &mut dataset,
        "/extensions/rust:macro-expansion/0/contexts",
        json!(["release"]),
    );
    expect(
        &validate_with(&dataset, &files()),
        &[(
            "error",
            "model:context-scope",
            "/extensions/rust:macro-expansion/0/contexts",
        )],
    );
}

#[test]
fn integral_float_coordinates_are_checked_like_integers() {
    let mut dataset = dataset();
    set(
        &mut dataset,
        "/entities/1/definitions/0/at/start",
        json!([1.0, 12.0]),
    );
    set(
        &mut dataset,
        "/entities/1/definitions/0/at/end",
        json!([1.0, 19.0]),
    );
    expect(&validate_with(&dataset, &files()), &[]);
    set(
        &mut dataset,
        "/entities/1/definitions/0/at/text",
        json!("Count"),
    );
    expect(
        &validate_with(&dataset, &files()),
        &[(
            "error",
            "model:locator-stale",
            "/entities/1/definitions/0/at",
        )],
    );
    set(
        &mut dataset,
        "/entities/1/definitions/0/at/end",
        json!([1, 1e30]),
    );
    expect(
        &validate_with(&dataset, &files()),
        &[(
            "error",
            "model:locator-range",
            "/entities/1/definitions/0/at",
        )],
    );
}

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
    audit, audit_with_reports, parse_yaml,
    project::{sha256_hex, MemoryProject},
    validate, Finding, Report, Severity,
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

/// The rule of public API recomputation. A test of another rule ignores
/// its findings, which restate the mutation's effect on public API; the
/// recomputation tests check them with [`expect_recomputed`].
const RECOMPUTED: &str = "model:public-api-membership-recomputed";

fn found_where(
    report: &Report,
    keep: impl Fn(&str) -> bool,
) -> Vec<(&'static str, String, String)> {
    report
        .findings
        .iter()
        .filter(|f| keep(&f.rule))
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
    // A mutation aimed at another rule can also disturb a signature check, which
    // has its own tests below; a test that names a signature rule sees them all.
    let names_signature_rule = expected.iter().any(|(_, rule, _)| is_signature_rule(rule));
    let found = found_where(report, |rule| {
        rule != RECOMPUTED && (names_signature_rule || !is_signature_rule(rule))
    });
    assert_eq!(found, expected, "{:#?}", report.findings);
}

fn is_signature_rule(rule: &str) -> bool {
    rule.starts_with("model:signature-") || rule == "rust:reachable-unnameable"
}

/// Expects exactly these public API recomputation findings, as
/// `(at, message fragment)`.
fn expect_recomputed(report: &Report, expected: &[(&str, &str)]) {
    let found: Vec<&Finding> = report
        .findings
        .iter()
        .filter(|f| f.rule == RECOMPUTED)
        .collect();
    assert_eq!(found.len(), expected.len(), "{:#?}", report.findings);
    for (finding, (at, fragment)) in found.iter().zip(expected) {
        assert_eq!(finding.at, *at, "{:#?}", report.findings);
        assert!(
            finding.message.contains(fragment),
            "`{}` lacks `{fragment}`",
            finding.message
        );
    }
}

fn validate_with(dataset: &Value, files: &MemoryProject) -> Report {
    validate(dataset, Some(files))
}

#[test]
fn the_fixture_passes_validation_and_audit() {
    let dataset = dataset();
    for report in [
        validate_with(&dataset, &files()),
        audit(&dataset, Some(&files())),
        validate(&dataset, None),
    ] {
        assert_eq!(found_where(&report, |_| true), [], "{:#?}", report.findings);
    }
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
            (
                "error",
                "model:id-reference",
                "/public-api/members/0/distribution",
            ),
            (
                "error",
                "model:id-reference",
                "/public-api/members/1/distribution",
            ),
            (
                "error",
                "model:id-reference",
                "/public-api/members/2/distribution",
            ),
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
        json!({"unit": {"access": "scoped", "scope": ["tally.Counter", "src/lib.rs", "elsewhere"]},
               "project": "none", "external": "none"}),
    );
    dataset["public-api"]["members"]
        .as_array_mut()
        .unwrap()
        .remove(1);
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
fn a_public_api_member_is_externally_accessible() {
    let mut dataset = dataset();
    set(
        &mut dataset,
        "/entities/2/effective-visibility/external",
        json!("none"),
    );
    set(
        &mut dataset,
        "/public-api/members/0/access",
        json!("subtypes"),
    );
    expect(
        &validate_with(&dataset, &files()),
        &[
            (
                "error",
                "model:public-api-access",
                "/public-api/members/0/access",
            ),
            (
                "error",
                "model:public-api-access",
                "/public-api/members/1/access",
            ),
        ],
    );
}

#[test]
fn a_member_of_another_distribution_is_one_the_rule_reaches() {
    let mut dataset = dataset();
    push(
        &mut dataset,
        "/distributions",
        json!({"id": "other", "kind": "crate", "name": "other", "version": "1.0.0"}),
    );
    let mut copy = dataset["public-api"]["members"][0].clone();
    copy["distribution"] = json!("other");
    push(&mut dataset, "/public-api/members", copy);
    expect_recomputed(
        &validate_with(&dataset, &files()),
        &[(
            "/public-api/members/3",
            "gives `tally.Counter` no consumer path in `other`",
        )],
    );
}

#[test]
fn a_public_api_member_is_stated_once_per_context() {
    let mut dataset = dataset();
    let copy = dataset["public-api"]["members"][0].clone();
    push(&mut dataset, "/public-api/members", copy);
    expect(
        &validate_with(&dataset, &files()),
        &[(
            "error",
            "model:public-api-duplicate",
            "/public-api/members/3",
        )],
    );
}

#[test]
fn a_signature_names_a_project_entity_in_its_contexts() {
    let mut dataset = dataset();
    set(
        &mut dataset,
        "/public-api/signatures/0/entity",
        json!("tally.absent"),
    );
    expect(
        &validate_with(&dataset, &files()),
        &[(
            "error",
            "model:id-reference",
            "/public-api/signatures/0/entity",
        )],
    );

    let mut dataset = self::dataset();
    set(
        &mut dataset,
        "/public-api/signatures/0/contexts",
        json!(["other"]),
    );
    expect(
        &validate_with(&dataset, &files()),
        &[(
            "error",
            "model:context-declared",
            "/public-api/signatures/0/contexts/0",
        )],
    );
}

#[test]
fn an_external_that_only_a_signature_names_is_used() {
    let mut dataset = dataset();
    let external = json!({"id": "ext.core.Marker", "kind": "type", "language-kind": "trait",
                          "distribution": "core", "path": "core::Marker"});
    push(&mut dataset, "/externals", external);
    expect(
        &validate_with(&dataset, &files()),
        &[("error", "model:external-unused", "/externals/0")],
    );
    set(
        &mut dataset,
        "/public-api/signatures/0/returns",
        json!({"inferred": "impl @{ext.core.Marker}"}),
    );
    let report = validate_with(&dataset, &files());
    assert!(
        !report
            .findings
            .iter()
            .any(|f| f.rule == "model:external-unused"),
        "{:#?}",
        report.findings
    );
}

#[test]
fn a_string_literal_names_no_external() {
    let mut dataset = dataset();
    let external = json!({"id": "ext.core.Marker", "kind": "type", "language-kind": "trait",
                          "distribution": "core", "path": "core::Marker"});
    push(&mut dataset, "/externals", external);
    for slot in [
        json!({"entity": "tally.make", "returns": "Literal<\"@{ext.core.Marker}\">"}),
        json!({"entity": "tally.make", "returns": "Literal<\"\\\"@{ext.core.Marker}\">"}),
        json!({"entity": "tally.make", "value": "\"@{ext.core.Marker}\""}),
        json!({"entity": "tally.make", "abi": "@{ext.core.Marker}"}),
        // A lifetime is no char literal, and it does not open a string either.
        json!({"entity": "tally.make", "returns": "&'a Tag<\"@{ext.core.Marker}\">"}),
    ] {
        set(&mut dataset, "/public-api/signatures/0", slot);
        expect(
            &validate_with(&dataset, &files()),
            &[("error", "model:external-unused", "/externals/0")],
        );
    }
}

/// A char literal's quote opens no string, so a name after it counts.
#[test]
fn a_char_literal_does_not_hide_a_later_external() {
    let mut dataset = dataset();
    let external = json!({"id": "ext.core.Marker", "kind": "type", "language-kind": "trait",
                          "distribution": "core", "path": "core::Marker"});
    push(&mut dataset, "/externals", external);
    for returns in [
        "@{tally.Counter}<'\"', @{ext.core.Marker}>",
        "@{tally.Counter}<'\\\"', @{ext.core.Marker}>",
        "@{tally.Counter}<'\\u{22}', @{ext.core.Marker}>",
        "@{tally.Counter}<'a', @{ext.core.Marker}>",
        "&'a @{tally.Counter}<@{ext.core.Marker}>",
    ] {
        set(
            &mut dataset,
            "/public-api/signatures/0/returns",
            json!(returns),
        );
        let report = validate_with(&dataset, &files());
        assert!(
            !report
                .findings
                .iter()
                .any(|f| f.rule == "model:external-unused"),
            "{returns}: {:#?}",
            report.findings
        );
    }
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
               "rust:reachable-unnameable": [{"entity": "tally.Counter", "distribution": "tally-lib", "via": ["tally.make"]}]}),
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
    set(&mut dataset, "/public-api/members", json!([]));
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
    set(
        &mut dataset,
        "/public-api/members/1/contexts",
        json!(["default"]),
    );
    set(
        &mut dataset,
        "/public-api/signatures/0/contexts",
        json!(["default"]),
    );
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
    set(
        &mut dataset,
        "/public-api/members/0/contexts",
        json!(["default"]),
    );
    set(
        &mut dataset,
        "/public-api/signatures/1/contexts",
        json!(["default"]),
    );
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
    set(
        &mut dataset,
        "/public-api/members/0/distribution",
        json!("missing"),
    );
    expect(
        &validate_with(&dataset, &files()),
        &[(
            "error",
            "model:id-reference",
            "/public-api/members/0/distribution",
        )],
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

#[test]
fn an_extension_target_exists_in_every_context_of_its_fact() {
    let mut dataset = two_contexts();
    set(&mut dataset, "/entities/2/contexts", json!(["default"]));
    set(
        &mut dataset,
        "/public-api/members/1/contexts",
        json!(["default"]),
    );
    set(
        &mut dataset,
        "/public-api/signatures/0/contexts",
        json!(["default"]),
    );
    set(&mut dataset, "/references/0/contexts", json!(["default"]));
    set(&mut dataset, "/references/1/contexts", json!(["default"]));
    set(
        &mut dataset,
        "/extensions",
        json!({"rust:macro-expansion": [{"entity": "tally.Counter", "macro": "tally.make",
               "invocation": {"path": "src/lib.rs", "start": [1, 12], "end": [1, 19], "text": "Counter"}}]}),
    );
    expect(
        &validate_with(&dataset, &files()),
        &[(
            "error",
            "model:context-scope",
            "/extensions/rust:macro-expansion/0/macro",
        )],
    );
}

#[test]
fn an_assembly_name_equal_to_an_id_does_not_own_an_extension_fact() {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../docs/specifications/semantic-oracle-profile-csharp.example.yml"
    );
    let mut dataset = parse_yaml(&std::fs::read_to_string(path).unwrap()).unwrap();
    // The aliased assembly's name equals the id of an entity in a unit
    // built only in `debug`; the alias site's unit is built in both.
    set(
        &mut dataset,
        "/extensions/csharp:extern-alias/0/unit",
        json!("Geometry.Tests.ShapeTests"),
    );
    set(
        &mut dataset,
        "/extensions/csharp:extern-alias/0/contexts",
        json!(["release"]),
    );
    expect(&validate(&dataset, None), &[]);
}

#[test]
fn a_site_is_in_a_document_of_its_fact_unit() {
    let mut dataset = dataset();
    let mut other = dataset["units"][0].clone();
    other["id"] = json!("tally-bin");
    other["kind"] = json!("bin");
    other["target-name"] = json!("tally-bin");
    other.as_object_mut().unwrap().remove("distribution");
    other.as_object_mut().unwrap().remove("root");
    push(&mut dataset, "/units", other);
    set(
        &mut dataset,
        "/documents/2/members",
        json!([{"unit": "tally-bin"}]),
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
fn an_unowned_extension_fact_holds_where_its_site_unit_is_built() {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../docs/specifications/semantic-oracle-profile-csharp.example.yml"
    );
    let mut dataset = parse_yaml(&std::fs::read_to_string(path).unwrap()).unwrap();
    // The alias site's document is compiled only in `debug`, while its unit
    // is built in both contexts.
    let documents = dataset["documents"].as_array_mut().unwrap();
    let program = documents
        .iter_mut()
        .find(|d| d["path"] == "src/Atlas/Program.cs")
        .unwrap();
    program["members"][0]["contexts"] = json!(["debug"]);
    let report = validate(&dataset, None);
    assert!(
        report
            .findings
            .iter()
            .any(|f| f.at == "/extensions/csharp:extern-alias/0/at/path"
                && f.rule == "model:site-membership"),
        "{:#?}",
        report.findings
    );
}

#[test]
fn equal_coordinates_are_one_site_whatever_their_number_form() {
    let mut dataset = dataset();
    let mut copy = dataset["references"][1].clone();
    copy["at"]["start"] = json!([4.0, 5.0]);
    push(&mut dataset, "/references", copy);
    expect(
        &validate_with(&dataset, &files()),
        &[("error", "model:locator-ambiguous", "/references/2/at")],
    );
}

// ------------------------------------------------- public API recomputation

#[test]
fn a_member_the_profile_rule_finds_has_an_entry() {
    let mut dataset = dataset();
    dataset["public-api"]["members"]
        .as_array_mut()
        .unwrap()
        .remove(1);
    expect_recomputed(
        &validate_with(&dataset, &files()),
        &[(
            "/public-api/members",
            "makes `tally.make` a member of `tally` with paths [`tally::make`]",
        )],
    );
}

#[test]
fn an_entry_lists_exactly_the_consumer_paths() {
    let mut dataset = dataset();
    set(
        &mut dataset,
        "/public-api/members/0/paths",
        json!(["tally::Counter", "tally::Count"]),
    );
    expect_recomputed(
        &validate_with(&dataset, &files()),
        &[(
            "/public-api/members/0/paths",
            "not consumer paths [`tally::Count`]",
        )],
    );
}

#[test]
fn only_a_pub_re_export_adds_a_consumer_path() {
    let mut dataset = dataset();
    let re_export = json!({
        "kind": "exports", "from": "tally.crate", "to": "tally.Counter",
        "name": "Tally", "visibility": "pub(crate)",
    });
    push(&mut dataset, "/relations", re_export.clone());
    expect_recomputed(&validate_with(&dataset, &files()), &[]);
    set(&mut dataset, "/relations/0/visibility", json!("pub"));
    expect_recomputed(
        &validate_with(&dataset, &files()),
        &[("/public-api/members/0/paths", "missing [`tally::Tally`]")],
    );
}

#[test]
fn a_private_item_has_no_consumer_path() {
    let mut dataset = dataset();
    set(
        &mut dataset,
        "/entities/2/declared-visibility",
        json!({"value": "private", "explicit": false}),
    );
    expect_recomputed(
        &validate_with(&dataset, &files()),
        &[
            ("/entities/2/effective-visibility", "`external` is `none`"),
            ("/entities/2/effective-visibility", "`project` is `none`"),
            (
                "/public-api/members/1",
                "gives `tally.make` no consumer path",
            ),
        ],
    );
}

#[test]
fn a_definition_site_holds_in_contexts_of_its_entity() {
    let mut dataset = dataset();
    set(
        &mut dataset,
        "/entities/1/definitions/0/contexts",
        json!(["default"]),
    );
    expect(&validate_with(&dataset, &files()), &[]);
    set(
        &mut dataset,
        "/entities/1/definitions/0/contexts",
        json!(["release"]),
    );
    expect(
        &validate_with(&dataset, &files()),
        &[(
            "error",
            "model:context-declared",
            "/entities/1/definitions/0/contexts/0",
        )],
    );
}

#[test]
fn a_scope_that_covers_the_unit_is_written_all() {
    let mut dataset = dataset();
    set(
        &mut dataset,
        "/entities/2/effective-visibility/unit",
        json!({"access": "scoped", "scope": ["tally.crate"]}),
    );
    expect(
        &validate_with(&dataset, &files()),
        &[(
            "error",
            "model:effective-visibility",
            "/entities/2/effective-visibility/unit",
        )],
    );
}

// ------------------------------------------------------------ signatures

/// The findings of one signature rule.
fn rule_findings(dataset: &Value, rule: &str) -> Vec<String> {
    validate_with(dataset, &files())
        .findings
        .into_iter()
        .filter(|f| f.rule == rule)
        .map(|f| format!("{} {}", f.at, f.message))
        .collect()
}

/// Asserts a rule finds nothing in the valid dataset, and one finding naming
/// `expect` after the mutation, so that a check that passes everything fails.
fn fails_rule(rule: &str, expect: &str, mutate: impl FnOnce(&mut Value)) {
    assert_eq!(
        rule_findings(&dataset(), rule),
        Vec::<String>::new(),
        "{rule}: the control"
    );
    let mut changed = dataset();
    mutate(&mut changed);
    let found = rule_findings(&changed, rule);
    assert!(
        found.iter().any(|message| message.contains(expect)),
        "{rule}: expected `{expect}` in {found:#?}"
    );
}

#[test]
fn signature_slots_rejects_a_slot_the_kind_does_not_have() {
    fails_rule("model:signature-slots", "has no `value` slot", |d| {
        set(d, "/public-api/signatures/0/value", json!("1"));
    });
}

#[test]
fn signature_slots_rejects_a_record_for_a_non_member() {
    fails_rule("model:signature-slots", "is no public API member", |d| {
        d["public-api"]["members"].as_array_mut().unwrap().remove(1);
    });
}

#[test]
fn signature_slots_rejects_two_records_in_one_context() {
    fails_rule("model:signature-slots", "another record states", |d| {
        let copy = d["public-api"]["signatures"][0].clone();
        push(d, "/public-api/signatures", copy);
    });
}

#[test]
fn signature_slots_requires_what_the_kind_always_has() {
    // A Rust field always has `type`.
    fails_rule("model:signature-slots", "which has type", |d| {
        let field = json!({"id": "tally.Counter.n", "kind": "field", "language-kind": "field",
            "name": "n", "path": "tally::Counter::n", "unit": "tally-lib", "parent": "tally.Counter",
            "origin": "source", "definitions": [],
            "declared-visibility": {"value": "pub", "explicit": true},
            "effective-visibility": {"unit": "all", "project": "all", "external": "all"}});
        push(d, "/entities", field);
        push(
            d,
            "/public-api/members",
            json!({"distribution": "tally", "entity": "tally.Counter.n",
                   "paths": ["tally::Counter::n"], "access": "all"}),
        );
    });
}

/// A record that exists and lacks the slot, not a member with no record: a constant
/// can hold `value` alone by the schema, and always has `type`.
#[test]
fn signature_slots_requires_a_record_to_hold_what_the_kind_always_has() {
    let constant = |d: &mut Value| {
        let entity = json!({"id": "tally.LIMIT", "kind": "constant", "language-kind": "const",
            "name": "LIMIT", "path": "tally::LIMIT", "unit": "tally-lib", "parent": "tally.crate",
            "origin": "source", "definitions": [],
            "declared-visibility": {"value": "pub", "explicit": true},
            "effective-visibility": {"unit": "all", "project": "all", "external": "all"}});
        push(d, "/entities", entity);
        push(
            d,
            "/public-api/members",
            json!({"distribution": "tally", "entity": "tally.LIMIT",
                   "paths": ["tally::LIMIT"], "access": "all"}),
        );
    };
    let mut control = dataset();
    constant(&mut control);
    push(
        &mut control,
        "/public-api/signatures",
        json!({"entity": "tally.LIMIT", "type": "u8", "value": "1"}),
    );
    assert_eq!(
        rule_findings(&control, "model:signature-slots"),
        Vec::<String>::new()
    );
    let mut lacking = dataset();
    constant(&mut lacking);
    push(
        &mut lacking,
        "/public-api/signatures",
        json!({"entity": "tally.LIMIT", "value": "1"}),
    );
    let found = rule_findings(&lacking, "model:signature-slots");
    assert!(
        found.iter().any(|m| m.contains("always has `type`")),
        "{found:#?}"
    );
}

#[test]
fn signature_slots_checks_auto_trait_ordinals() {
    let pair = json!("(impl @{tally.Counter}, impl @{tally.Counter})");
    fails_rule("model:signature-slots", "holds no opaque type", |d| {
        set(d, "/public-api/signatures/0/auto-traits", json!(["Send"]));
    });
    fails_rule("model:signature-slots", "has plain elements", |d| {
        set(
            d,
            "/public-api/signatures/0/returns",
            json!("impl @{tally.Counter}"),
        );
        set(
            d,
            "/public-api/signatures/0/auto-traits",
            json!(["#0 Send"]),
        );
    });
    fails_rule("model:signature-slots", "names one by `#<n> `", |d| {
        set(d, "/public-api/signatures/0/returns", pair.clone());
        set(d, "/public-api/signatures/0/auto-traits", json!(["Send"]));
    });
    fails_rule("model:signature-slots", "past its 2 occurrence", |d| {
        set(d, "/public-api/signatures/0/returns", pair.clone());
        set(
            d,
            "/public-api/signatures/0/auto-traits",
            json!(["#2 Send"]),
        );
    });
    // A type's elements are plain.
    fails_rule("model:signature-slots", "has plain elements", |d| {
        set(
            d,
            "/public-api/signatures/1/auto-traits",
            json!(["#0 Send"]),
        );
    });
}

#[test]
fn signature_canonical_rejects_a_second_spelling() {
    fails_rule("model:signature-canonical", "its one spelling is", |d| {
        set(
            d,
            "/public-api/signatures/0/returns",
            json!("( @{tally.Counter} )"),
        );
    });
}

#[test]
fn signature_canonical_rejects_an_id_that_resolves_to_nothing() {
    fails_rule("model:signature-canonical", "resolves to no entity", |d| {
        set(
            d,
            "/public-api/signatures/0/returns",
            json!("@{tally.Absent}"),
        );
    });
}

#[test]
fn signature_canonical_rejects_a_bare_entity_name() {
    fails_rule(
        "model:signature-canonical",
        "neither predeclared nor a type parameter",
        |d| {
            set(d, "/public-api/signatures/0/returns", json!("Counter"));
        },
    );
}

#[test]
fn signature_canonical_rejects_an_unresolved_name() {
    fails_rule("model:signature-canonical", "observation form", |d| {
        set(d, "/public-api/signatures/0/returns", json!("@?{Counter}"));
    });
}

#[test]
fn signature_canonical_rejects_a_value_outside_the_forms() {
    fails_rule("model:signature-canonical", "is not a value form", |d| {
        set(
            d,
            "/public-api/signatures/0",
            json!({"entity": "tally.make", "value": "1 + 1"}),
        );
    });
}

#[test]
fn signature_canonical_rejects_an_unsorted_set() {
    fails_rule(
        "model:signature-canonical",
        "`auto-traits` is sorted",
        |d| {
            set(
                d,
                "/public-api/signatures/1/auto-traits",
                json!(["Send", "RefUnwindSafe"]),
            );
        },
    );
    fails_rule("model:signature-canonical", "its one spelling is", |d| {
        set(
            d,
            "/public-api/signatures/1/auto-traits",
            json!(["Send when U: Send, T: Send"]),
        );
    });
}

#[test]
fn signature_canonical_rejects_an_unsorted_set_below_the_record() {
    fails_rule(
        "model:signature-canonical",
        "/bindings/0/auto-traits",
        |d| {
            set(
                d,
                "/public-api/signatures/1/bases",
                json!([{"type": "@{tally.make}", "bindings": [
                {"name": "m", "auto-traits": ["Send", "RefUnwindSafe"]}]}]),
            );
        },
    );
    fails_rule("model:signature-canonical", "/bases/0/bindings", |d| {
        set(
            d,
            "/public-api/signatures/1/bases",
            json!([{"type": "@{tally.make}", "bindings": [
                {"name": "z", "type": "u8"}, {"name": "a", "type": "u8"}]}]),
        );
    });
}

/// A C# `bounds` slot sorts by the language's rule, not by its text.
#[test]
fn signature_canonical_reads_a_bounds_slot_by_its_rule() {
    fails_rule("model:signature-canonical", "its one spelling is", |d| {
        set(
            d,
            "/public-api/signatures/0/type-parameters",
            json!([{"name": "T", "kind": "type", "bounds": "@{tally.make} + @{tally.Counter}"}]),
        );
    });
}

#[test]
fn signature_parameters_rejects_a_list_that_differs_from_the_entities() {
    fails_rule("model:signature-parameters", "the parameter list is", |d| {
        set(
            d,
            "/public-api/signatures/0/parameters",
            json!([{"name": "other", "type": "u8"}]),
        );
    });
}

#[test]
fn signature_parameters_rejects_an_entity_no_list_names() {
    fails_rule("model:signature-parameters", "the parameter list is", |d| {
        let parameter = json!({"id": "tally.make.n", "kind": "parameter", "language-kind": "param",
            "name": "n", "path": "tally::make::(n)", "unit": "tally-lib", "parent": "tally.make",
            "origin": "source",
            "definitions": [{"at": {"path": "src/lib.rs", "start": [3, 8], "end": [3, 12], "text": "make"}}]});
        push(d, "/entities", parameter);
    });
}

#[test]
fn signature_references_rejects_an_id_with_no_reference() {
    fails_rule(
        "model:signature-references",
        "has no `type` reference",
        |d| {
            d["references"].as_array_mut().unwrap().remove(0);
        },
    );
}

#[test]
fn signature_references_rejects_a_reference_from_outside_the_declaration() {
    fails_rule(
        "model:signature-references",
        "has no `type` reference",
        |d| {
            set(d, "/references/0/from", json!("tally.crate"));
        },
    );
}

#[test]
fn signature_inference_rejects_inferred_on_a_rust_source_entity() {
    fails_rule("model:signature-inference", "{inferred} is for what", |d| {
        set(
            d,
            "/public-api/signatures/0/returns",
            json!({"inferred": "@{tally.Counter}"}),
        );
    });
}

#[test]
fn signature_bases_rejects_a_base_with_no_relation_and_a_relation_with_no_base() {
    fails_rule("model:signature-bases", "no relation targets it", |d| {
        set(
            d,
            "/public-api/signatures/1/bases",
            json!([{"type": "@{tally.make}"}]),
        );
    });
    fails_rule("model:signature-bases", "has no `bases` entry", |d| {
        push(
            d,
            "/relations",
            json!({"kind": "implements", "from": "tally.make", "to": "tally.Counter"}),
        );
    });
    fails_rule(
        "model:signature-bases",
        "no record states its `bases`",
        |d| {
            d["public-api"]["signatures"]
                .as_array_mut()
                .unwrap()
                .remove(1);
            push(
                d,
                "/relations",
                json!({"kind": "implements", "from": "tally.Counter", "to": "tally.make"}),
            );
        },
    );
}

#[test]
fn reachable_unnameable_via_is_recomputed() {
    let rule = "rust:reachable-unnameable";
    // `make` exposes `Counter`, which has no consumer path once it is no member.
    fails_rule(rule, "no fact states it", |d| {
        let members = d["public-api"]["members"].as_array_mut().unwrap();
        members.retain(|m| m["entity"] != "tally.Counter");
        d["public-api"]["signatures"]
            .as_array_mut()
            .unwrap()
            .remove(1);
    });
    let fact =
        json!({"entity": "tally.Counter", "distribution": "tally", "via": ["tally.Counter"]});
    fails_rule(rule, "`via` is", |d| {
        set(
            d,
            "/extensions",
            json!({"rust:reachable-unnameable": [fact]}),
        );
    });
}

// ------------------------------------------------------- compiler reports

fn report(slots: Value, covers: Value) -> Value {
    json!({
        "format": "semantic-oracle-compiler-report/1",
        "language": "rust",
        "context": "default",
        "tool": {"name": "rustdoc", "version": "test"},
        "release": "1.99.1",
        "covers": covers,
        "members": [{"path": "tally::make", "slots": slots, "unstated": []}],
    })
}

const COMPILER: &str = "profile:signature-compiler";

fn compiler_findings(report: &Report) -> Vec<String> {
    report
        .findings
        .iter()
        .filter(|f| f.rule == COMPILER)
        .map(|f| f.message.clone())
        .collect()
}

#[test]
fn a_slot_with_no_report_is_unchecked_and_has_not_passed() {
    let audited = audit_with_reports(&dataset(), Some(&files()), &[]);
    assert_eq!(audited.agreed, 0);
    assert!(!audited.unchecked.is_empty());
    assert!(audited.unchecked.iter().any(|u| u.slot == "returns"));
    assert_eq!(audited.summary()["unchecked"], audited.unchecked.len());
}

#[test]
fn a_report_that_agrees_counts_the_slot_as_agreed() {
    let reported = report(json!({"returns": "@{tally::Counter}"}), json!(["returns"]));
    let audited = audit_with_reports(&dataset(), Some(&files()), &[reported]);
    assert_eq!(compiler_findings(&audited), Vec::<String>::new());
    assert_eq!(audited.agreed, 1);
    assert!(!audited.unchecked.iter().any(|u| u.slot == "returns"));
}

#[test]
fn a_report_compares_after_canonical_spelling() {
    let reported = report(
        json!({"returns": "( @{tally::Counter} )"}),
        json!(["returns"]),
    );
    let audited = audit_with_reports(&dataset(), Some(&files()), &[reported]);
    assert_eq!(compiler_findings(&audited), Vec::<String>::new());
    assert_eq!(audited.agreed, 1);
}

#[test]
fn a_disagreement_names_the_rule() {
    let reported = report(json!({"returns": "u32"}), json!(["returns"]));
    let audited = audit_with_reports(&dataset(), Some(&files()), &[reported]);
    let found = compiler_findings(&audited);
    assert_eq!(found.len(), 1, "{found:#?}");
    assert!(found[0].contains("states `u32`"), "{found:#?}");
    assert_eq!(audited.agreed, 0);
}

#[test]
fn a_report_must_run_under_the_contexts_release() {
    let agrees = json!({"returns": "@{tally::Counter}"});
    let audited_with = |release: Option<&str>| {
        let mut reported = report(agrees.clone(), json!(["returns"]));
        match release {
            Some(release) => reported["release"] = json!(release),
            None => {
                reported.as_object_mut().unwrap().remove("release");
            }
        }
        audit_with_reports(&dataset(), Some(&files()), &[reported])
    };
    // The context declares `release: '1.99'`; a later patch release is the same.
    for release in ["1.99", "1.99.1"] {
        let audited = audited_with(Some(release));
        assert!(compiler_findings(&audited).is_empty(), "{release}");
        assert_eq!(audited.agreed, 1, "{release}");
    }
    // Another minor release, and a prefix that is not a dotted component.
    for release in ["1.98.0", "1.9", "1.990"] {
        let audited = audited_with(Some(release));
        let found = compiler_findings(&audited);
        assert!(
            found.iter().any(|f| f.contains("ran under release")),
            "{release}: {found:#?}"
        );
        assert_eq!(audited.agreed, 0, "{release}");
    }
    let found = compiler_findings(&audited_with(None));
    assert!(
        found.iter().any(|f| f.contains("states no `release`")),
        "{found:#?}"
    );
}

#[test]
fn a_covered_slot_the_report_lacks_is_a_disagreement() {
    let reported = report(json!({}), json!(["returns"]));
    let found = compiler_findings(&audit_with_reports(&dataset(), Some(&files()), &[reported]));
    assert!(found[0].contains("states no such slot"), "{found:#?}");
}

#[test]
fn a_slot_the_report_states_and_the_record_lacks_is_a_disagreement() {
    let reported = report(
        json!({"returns": "@{tally::Counter}", "abi": "C"}),
        json!(["returns", "abi"]),
    );
    let found = compiler_findings(&audit_with_reports(&dataset(), Some(&files()), &[reported]));
    assert!(
        found.iter().any(|m| m.contains("the record lacks it")),
        "{found:#?}"
    );
}

#[test]
fn a_slot_a_report_does_not_cover_stays_unchecked() {
    let reported = report(json!({}), json!(["parameters[].type"]));
    let audited = audit_with_reports(&dataset(), Some(&files()), &[reported]);
    assert_eq!(compiler_findings(&audited), Vec::<String>::new());
    assert_eq!(audited.agreed, 0);
    assert!(audited.unchecked.iter().any(|u| u.slot == "returns"));
}

#[test]
fn a_member_a_report_marks_unstated_stays_unchecked() {
    let mut reported = report(json!({}), json!(["returns"]));
    reported["members"][0]["unstated"] = json!(["returns"]);
    let audited = audit_with_reports(&dataset(), Some(&files()), &[reported]);
    assert_eq!(compiler_findings(&audited), Vec::<String>::new());
    assert!(audited.unchecked.iter().any(|u| u.slot == "returns"));
}

#[test]
fn a_report_of_another_language_or_context_is_rejected() {
    let mut other_language = report(json!({}), json!([]));
    other_language["language"] = json!("go");
    let found = compiler_findings(&audit_with_reports(
        &dataset(),
        Some(&files()),
        &[other_language],
    ));
    assert!(found[0].contains("is for `go`"), "{found:#?}");
    let mut other_context = report(json!({}), json!([]));
    other_context["context"] = json!("absent");
    let found = compiler_findings(&audit_with_reports(
        &dataset(),
        Some(&files()),
        &[other_context],
    ));
    assert!(found[0].contains("is not declared"), "{found:#?}");
}

#[test]
fn every_context_needs_a_report_for_a_slot_to_agree() {
    let dataset = two_contexts();
    let reported = report(json!({"returns": "@{tally::Counter}"}), json!(["returns"]));
    let audited = audit_with_reports(&dataset, Some(&files()), &[reported]);
    assert_eq!(audited.agreed, 0);
    let unchecked = audited
        .unchecked
        .iter()
        .find(|u| u.slot == "returns")
        .unwrap();
    // Per slot and context: agreed where a report states it, and unchecked where none
    // does, with both in the output and no pass summed.
    assert_eq!(unchecked.agreed_in, ["default"]);
    assert_eq!(unchecked.unchecked_in, ["release"]);
    assert_eq!(audited.contexts["default"].agreed, 1);
    assert!(audited.contexts["release"].unchecked >= 1);
    assert_eq!(audited.contexts["release"].agreed, 0);
    assert_eq!(audited.summary()["agreed"], 0);
    assert_eq!(audited.summary()["contexts"]["default"]["agreed"], 1);
    assert_eq!(unchecked.to_json()["agreed-in"], json!(["default"]));
    assert_eq!(unchecked.to_json()["unchecked-in"], json!(["release"]));
    assert!(
        unchecked.reason.contains("no compiler report for context"),
        "{unchecked:?}"
    );
}

/// One path can name several kinds, each with its own member in a report.
#[test]
fn a_report_reads_a_member_by_its_kind() {
    let mut reported = report(json!({"returns": "@{tally::Counter}"}), json!(["returns"]));
    reported["members"][0]["kind"] = json!("callable");
    let audited = audit_with_reports(&dataset(), Some(&files()), &[reported.clone()]);
    assert_eq!(audited.agreed, 1);
    // A member of another kind at this path says nothing about this one.
    reported["members"][0]["kind"] = json!("module");
    reported["members"][0]["slots"] = json!({"returns": "u32"});
    let audited = audit_with_reports(&dataset(), Some(&files()), &[reported]);
    assert_eq!(compiler_findings(&audited), Vec::<String>::new());
    assert_eq!(audited.agreed, 0);
}

/// A report lists what it could not read, and that does not count as agreement.
#[test]
fn a_report_that_could_not_read_a_module_warns() {
    let mut reported = report(json!({}), json!([]));
    reported["failed"] = json!(["pkg.broken: ImportError"]);
    let audited = audit_with_reports(&dataset(), Some(&files()), &[reported]);
    assert!(audited
        .findings
        .iter()
        .any(|f| f.rule == COMPILER && f.message.contains("pkg.broken")));
    assert_eq!(audited.agreed, 0);
}

/// Two units can hold one path, such as two versions of one crate.
#[test]
fn a_report_of_another_unit_says_nothing_about_this_one() {
    let mut reported = report(json!({"returns": "u32"}), json!(["returns"]));
    reported["members"][0]["unit"] = json!({"package-version": "9.9.9"});
    let audited = audit_with_reports(&dataset(), Some(&files()), &[reported.clone()]);
    assert_eq!(compiler_findings(&audited), Vec::<String>::new());
    assert!(audited.unchecked.iter().any(|u| u.slot == "returns"));
    let version = dataset()["units"][0]["package-version"].clone();
    reported["members"][0]["unit"] = json!({"package-version": version});
    let found = compiler_findings(&audit_with_reports(&dataset(), Some(&files()), &[reported]));
    assert!(
        found.iter().any(|m| m.contains("states `u32`")),
        "{found:#?}"
    );
}

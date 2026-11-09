// ---
// relationships:
//   references: semantic-oracle-model
// ---
//! Checks the semantic-oracle schema assets against their examples.
//!
//! Every `<specification>.example.yml` in `docs/specifications` is a valid
//! dataset and must validate with no error. Every
//! `<specification>.invalid-examples.yml` lists cases; each case applies a
//! JSON Patch to a valid example and names the one reason the result fails:
//! the instance location of the defect and the schema keyword that rejects
//! it, as the validator names keywords (`falseSchema` for a `false` schema).
//! A case that rejects a member names it in `member`, and `also` lists any
//! further error the same defect causes, with its own `member` where its
//! keyword rejects members. A case passes only when the
//! distinct `(location, keyword)` pairs of the errors are exactly the named
//! one and those in `also`, and each such error is about exactly its
//! `member`: one name, or a list.

use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
};

use beholder_semantic_oracle::{
    dataset_validator, model_schema, parse_yaml, profile_validator, Profile, MODEL_SCHEMA_ID,
    MODEL_VERSION, PROFILES,
};
use jsonschema::{error::ValidationErrorKind, Validator};
use serde_json::{json, Value};

fn specifications() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/specifications")
}

fn load(name: &str) -> Value {
    let path = specifications().join(name);
    let text = fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    parse_yaml(&text).unwrap_or_else(|e| panic!("parse {}: {e}", path.display()))
}

/// Asset file names in `docs/specifications` that end with `suffix`.
fn assets(suffix: &str) -> Vec<String> {
    let mut names: Vec<String> = fs::read_dir(specifications())
        .expect("read docs/specifications")
        .map(|entry| {
            entry
                .expect("read entry")
                .file_name()
                .into_string()
                .expect("UTF-8 name")
        })
        .filter(|name| name.starts_with("semantic-oracle-") && name.ends_with(suffix))
        .collect();
    names.sort();
    names
}

/// `(instance location, schema keyword, message)` of every error.
fn errors(validator: &Validator, instance: &Value) -> Vec<(String, String, String)> {
    validator
        .iter_errors(instance)
        .map(|error| {
            (
                error.instance_path().to_string(),
                error.kind().keyword().to_string(),
                error.to_string(),
            )
        })
        .collect()
}

/// Keywords whose error is about one member of an object.
const MEMBER_KEYWORDS: [&str; 3] = ["required", "additionalProperties", "unevaluatedProperties"];

/// The members the errors at `at` with `keyword` are about.
fn members(validator: &Validator, instance: &Value, at: &str, keyword: &str) -> BTreeSet<String> {
    let mut members = BTreeSet::new();
    for error in validator.iter_errors(instance) {
        if error.instance_path().to_string() != at || error.kind().keyword() != keyword {
            continue;
        }
        match error.kind() {
            ValidationErrorKind::Required { property } => {
                members.insert(property.as_str().unwrap_or_default().to_string());
            }
            ValidationErrorKind::AdditionalProperties { unexpected }
            | ValidationErrorKind::UnevaluatedProperties { unexpected } => {
                members.extend(unexpected.iter().cloned());
            }
            _ => {}
        }
    }
    members
}

fn describe(errors: &[(String, String, String)]) -> String {
    errors
        .iter()
        .map(|(at, keyword, message)| format!("  {at} [{keyword}] {message}"))
        .collect::<Vec<_>>()
        .join("\n")
}

/// The profile a valid example belongs to, from its file name.
fn profile_of(example: &str) -> Profile {
    PROFILES
        .into_iter()
        .find(|profile| example.starts_with(&format!("{}.", profile.specification)))
        .unwrap_or_else(|| panic!("{example} names no profile specification"))
}

#[test]
fn every_profile_has_a_valid_example() {
    let examples = assets(".example.yml");
    for profile in PROFILES {
        let name = format!("{}.example.yml", profile.specification);
        assert!(examples.contains(&name), "missing {name}");
    }
}

#[test]
fn valid_examples_validate() {
    let any = dataset_validator().expect("dataset validator");
    let mut failures = Vec::new();
    for example in assets(".example.yml") {
        let dataset = load(&example);
        let profile = profile_of(&example);
        let own = profile_validator(&profile).expect("profile validator");
        for (label, validator) in [("dataset schema", &any), (profile.version, &own)] {
            let found = errors(validator, &dataset);
            if !found.is_empty() {
                failures.push(format!("{example} under {label}:\n{}", describe(&found)));
            }
        }
        for other in PROFILES.into_iter().filter(|other| *other != profile) {
            let validator = profile_validator(&other).expect("profile validator");
            assert!(
                !validator.is_valid(&dataset),
                "{example} validates under {}",
                other.version
            );
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn invalid_examples_fail_for_their_named_reason() {
    let validator = dataset_validator().expect("dataset validator");
    let mut failures = Vec::new();
    let mut count = 0;
    for file in assets(".invalid-examples.yml") {
        let document = load(&file);
        let cases = document["cases"]
            .as_array()
            .unwrap_or_else(|| panic!("{file}: no cases"));
        assert!(!cases.is_empty(), "{file}: no cases");
        let mut names = BTreeSet::new();
        for case in cases {
            count += 1;
            let name = case["name"].as_str().expect("case name");
            assert!(names.insert(name), "{file}: duplicate case {name}");
            assert!(case["reason"].is_string(), "{file} {name}: no reason");
            let fails_at = case["fails-at"].as_str().expect("fails-at");
            let keyword = case["keyword"].as_str().expect("keyword");
            let mut dataset = load(case["base"].as_str().expect("base"));
            let patch: json_patch::Patch =
                serde_json::from_value(case["patch"].clone()).expect("patch");
            json_patch::patch(&mut dataset, &patch)
                .unwrap_or_else(|e| panic!("{file} {name}: patch does not apply: {e}"));

            let found = errors(&validator, &dataset);
            let label = format!("{file} {name}");
            if found.is_empty() {
                failures.push(format!("{label}: validates"));
                continue;
            }
            // The named error and each `also` error: location, keyword and
            // the members the error must be about.
            let mut named = vec![(fails_at.to_string(), keyword.to_string(), &case["member"])];
            for also in case["also"].as_array().into_iter().flatten() {
                let at = also["at"].as_str().expect("also at");
                let keyword = also["keyword"].as_str().expect("also keyword");
                named.push((at.to_string(), keyword.to_string(), &also["member"]));
            }
            let expected: BTreeSet<(String, String)> = named
                .iter()
                .map(|(at, keyword, _)| (at.clone(), keyword.clone()))
                .collect();
            let actual: BTreeSet<(String, String)> = found
                .iter()
                .map(|(at, keyword, _)| (at.clone(), keyword.clone()))
                .collect();
            if actual != expected {
                failures.push(format!(
                    "{label}: expected {expected:?}, found:\n{}",
                    describe(&found)
                ));
            }
            for (at, keyword, member) in &named {
                let listed: Option<BTreeSet<String>> = match member {
                    Value::Null => None,
                    Value::String(member) => Some(BTreeSet::from([member.clone()])),
                    Value::Array(items) => Some(
                        items
                            .iter()
                            .map(|item| item.as_str().expect("member name").to_string())
                            .collect(),
                    ),
                    other => panic!("{label}: member {other}"),
                };
                if MEMBER_KEYWORDS.contains(&keyword.as_str()) {
                    let found = members(&validator, &dataset, at, keyword);
                    match listed {
                        None => failures.push(format!(
                            "{label}: the `{keyword}` error at {at} names no member"
                        )),
                        Some(listed) if listed != found => failures.push(format!(
                            "{label}: the `{keyword}` error at {at} is about {found:?}, not {listed:?}"
                        )),
                        Some(_) => {}
                    }
                } else if listed.is_some() {
                    failures.push(format!("{label}: `member` with keyword `{keyword}`"));
                }
            }
        }
    }
    assert!(count > 0, "no invalid examples");
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn every_extension_kind_has_a_valid_example() {
    for profile in PROFILES {
        let schema = profile.schema().expect("profile schema");
        let kinds = schema["$defs"]["extensions"]["properties"]
            .as_object()
            .unwrap_or_else(|| panic!("{}: no extension kinds", profile.version));
        let example = load(&format!("{}.example.yml", profile.specification));
        for kind in kinds.keys() {
            let facts = example["extensions"][kind].as_array();
            assert!(
                facts.is_some_and(|facts| !facts.is_empty()),
                "{}: no example fact for {kind}",
                profile.version
            );
        }
    }
}

#[test]
fn every_disposition_has_a_valid_example() {
    let mut seen = BTreeSet::new();
    for example in assets(".example.yml") {
        let dataset = load(&example);
        for reference in dataset["references"].as_array().into_iter().flatten() {
            let disposition = reference["disposition"].as_str().unwrap_or("default");
            seen.insert(disposition.to_string());
        }
    }
    for disposition in [
        "default",
        "resolved",
        "external",
        "candidates",
        "nonexistent",
        "indeterminate",
    ] {
        assert!(
            seen.contains(disposition),
            "no example reference with disposition {disposition}"
        );
    }
}

#[test]
fn schema_ids_and_version_constants_agree() {
    let model = model_schema().expect("model schema");
    assert_eq!(model["$id"], MODEL_SCHEMA_ID);
    assert_eq!(
        model["$defs"]["dataset"]["properties"]["model"]["const"],
        MODEL_VERSION
    );

    let listed: BTreeSet<&str> = model["properties"]["profile"]["enum"]
        .as_array()
        .expect("profile enum")
        .iter()
        .map(|value| value.as_str().expect("profile version"))
        .collect();
    let defined: BTreeSet<&str> = PROFILES.iter().map(|profile| profile.version).collect();
    assert_eq!(listed, defined, "model schema profile list");

    let dispatched: BTreeSet<(String, String)> = model["allOf"]
        .as_array()
        .expect("profile dispatch")
        .iter()
        .map(|rule| {
            (
                rule["if"]["properties"]["profile"]["const"]
                    .as_str()
                    .expect("profile")
                    .to_string(),
                rule["then"]["$ref"]
                    .as_str()
                    .expect("profile schema")
                    .to_string(),
            )
        })
        .collect();
    let expected: BTreeSet<(String, String)> = PROFILES
        .iter()
        .map(|profile| (profile.version.to_string(), profile.schema_id.to_string()))
        .collect();
    assert_eq!(dispatched, expected, "model schema profile dispatch");

    for profile in PROFILES {
        let schema = profile.schema().expect("profile schema");
        assert_eq!(schema["$id"], profile.schema_id);
        assert_eq!(
            profile.schema_id,
            format!("{MODEL_SCHEMA_ID}/{}", profile.version),
            "profile schema ids extend the model schema id"
        );
        assert_eq!(schema["properties"]["profile"]["const"], profile.version);
        assert_eq!(
            schema["properties"]["project"]["properties"]["language"]["const"],
            profile.language
        );
        assert_eq!(Profile::by_version(profile.version), Some(profile));
    }
}

/// The examples have no project files, so only rules that need none apply.
/// The examples are excerpts, so coverage warnings are expected and not
/// checked.
#[test]
fn valid_examples_pass_validation_and_audit_without_project_files() {
    let mut unexpected = Vec::new();
    for example in assets(".example.yml") {
        let dataset = load(&example);
        let report = beholder_semantic_oracle::audit(&dataset, None);
        for finding in report
            .findings
            .iter()
            .filter(|finding| finding.severity == beholder_semantic_oracle::Severity::Error)
        {
            unexpected.push(format!(
                "{example}: {} {} {}: {}",
                finding.severity, finding.rule, finding.at, finding.message
            ));
        }
    }
    assert!(unexpected.is_empty(), "{}", unexpected.join("\n"));
}

/// Every profile's public API rule is recomputed: an example without one of
/// its `public-api` entries reports the member the rule still finds.
#[test]
fn every_profile_recomputes_public_api() {
    for example in assets(".example.yml") {
        let mut dataset = load(&example);
        let entries = dataset["public-api"]["members"].as_array_mut().unwrap();
        let removed = entries.pop().expect("the example states public API");
        let report = beholder_semantic_oracle::validate(&dataset, None);
        let recomputed: Vec<_> = report
            .findings
            .iter()
            .filter(|f| f.rule == "model:public-api-membership-recomputed")
            .collect();
        assert_eq!(recomputed.len(), 1, "{example}: {recomputed:#?}");
        let entity = removed["entity"].as_str().unwrap();
        assert_eq!(recomputed[0].at, "/public-api/members", "{example}");
        assert!(
            recomputed[0].message.contains(&format!("`{entity}`")),
            "{example}: {}",
            recomputed[0].message
        );
    }
}

/// The public API recomputation findings of a dataset, as messages.
fn recomputed(dataset: &Value) -> Vec<String> {
    beholder_semantic_oracle::validate(dataset, None)
        .findings
        .into_iter()
        .filter(|f| f.rule == "model:public-api-membership-recomputed")
        .map(|f| format!("{} {}", f.at, f.message))
        .collect()
}

fn push(dataset: &mut Value, pointer: &str, value: Value) {
    dataset
        .pointer_mut(pointer)
        .and_then(Value::as_array_mut)
        .unwrap_or_else(|| panic!("no list {pointer}"))
        .push(value);
}

/// A project entity with the members recomputation reads.
fn entity(
    id: &str,
    kind: &str,
    language_kind: &str,
    path: &str,
    parent: &str,
    unit: &str,
) -> Value {
    json!({
        "id": id, "kind": kind, "language-kind": language_kind,
        "name": path.rsplit(['.', ':']).next().unwrap(),
        "path": path, "unit": unit, "parent": parent, "origin": "source",
        "definitions": [],
        "declared-visibility": {"value": "exported", "explicit": true},
        "effective-visibility": {"unit": "all", "project": "all", "external": "all"},
    })
}

#[test]
fn a_re_export_cycle_adds_no_consumer_path() {
    let mut rust = load("semantic-oracle-profile-rust.example.yml");
    for (from, name) in [("geometry.crate", "again"), ("geometry.shapes", "up")] {
        push(
            &mut rust,
            "/relations",
            json!({"kind": "exports", "from": from, "to": "geometry.crate", "name": name, "visibility": "pub"}),
        );
    }
    assert_eq!(recomputed(&rust), Vec::<String>::new());

    let mut typescript = load("semantic-oracle-profile-typescript.example.yml");
    push(
        &mut typescript,
        "/relations",
        json!({"kind": "exports", "from": "index", "to": "index", "name": "again"}),
    );
    let findings = recomputed(&typescript);
    assert!(
        findings.iter().all(|f| !f.contains("#again")),
        "{findings:#?}"
    );
}

#[test]
fn rust_inherent_items_belong_to_the_unit_of_their_type() {
    let mut dataset = load("semantic-oracle-profile-rust.example.yml");
    let mut block = entity(
        "atlas.impl-Circle",
        "extension",
        "impl",
        "atlas::impl geometry::shapes::Circle#1",
        "atlas.crate",
        "atlas-bin",
    );
    block.as_object_mut().unwrap().remove("name");
    block["declared-visibility"] = json!({"value": "private", "explicit": false});
    block["effective-visibility"] = json!({"unit": {"access": "scoped", "scope": ["atlas.crate"]}, "project": "none", "external": "none"});
    push(&mut dataset, "/entities", block);
    let mut item = entity(
        "atlas.Circle.extra",
        "callable",
        "method",
        "geometry::shapes::Circle::extra",
        "atlas.impl-Circle",
        "atlas-bin",
    );
    item["declared-visibility"] = json!({"value": "pub", "explicit": true});
    item["effective-visibility"] = json!({"unit": "all", "project": "none", "external": "none"});
    push(&mut dataset, "/entities", item);
    let findings = recomputed(&dataset);
    assert!(
        findings.iter().all(|f| !f.contains("atlas.Circle.extra")),
        "{findings:#?}"
    );
}

#[test]
fn go_promotion_through_a_diamond_is_ambiguous() {
    let mut dataset = load("semantic-oracle-profile-go.example.yml");
    let unit = "geometry-pkg";
    let package = "geometry";
    let prefix = "example.com/geometry";
    for name in ["Left", "Right", "Outer"] {
        push(
            &mut dataset,
            "/entities",
            entity(
                &format!("geometry.{name}"),
                "type",
                "struct",
                &format!("{prefix}.{name}"),
                package,
                unit,
            ),
        );
    }
    // Left and Right embed Base; Outer embeds Left and Right; Base embeds
    // Outer, closing a cycle.
    let embeds = [
        ("Left", "Base"),
        ("Right", "Base"),
        ("Outer", "Left"),
        ("Outer", "Right"),
        ("Base", "Outer"),
    ];
    for (outer, inner) in embeds {
        let field = format!("geometry.{outer}.{inner}");
        push(
            &mut dataset,
            "/entities",
            entity(
                &field,
                "field",
                "embedded",
                &format!("{prefix}.{outer}.{inner}"),
                &format!("geometry.{outer}"),
                unit,
            ),
        );
        push(
            &mut dataset,
            "/extensions/go:embeds",
            json!({"struct": format!("geometry.{outer}"), "field": field, "type": format!("geometry.{inner}")}),
        );
    }
    let findings = recomputed(&dataset).join("\n");
    assert!(
        findings.contains("`example.com/geometry.Left.Label`"),
        "{findings}"
    );
    assert!(
        findings.contains("`example.com/geometry.Right.Label`"),
        "{findings}"
    );
    assert!(!findings.contains("Outer.Label"), "{findings}");
}

#[test]
fn a_python_submodule_named_by_all_follows_stub_selection() {
    for stub_first in [false, true] {
        let mut dataset = load("semantic-oracle-profile-python.example.yml");
        if stub_first {
            let entities = dataset["entities"].as_array_mut().unwrap();
            let at = entities
                .iter()
                .position(|e| e["id"] == "stub.atlas.shapes")
                .unwrap();
            let stub = entities.remove(at);
            entities.insert(0, stub);
        }
        let all = dataset["extensions"]
            .as_object_mut()
            .unwrap()
            .entry("python:all-member")
            .or_insert_with(|| json!([]));
        all.as_array_mut()
            .unwrap()
            .push(json!({"module": "atlas", "name": "shapes"}));
        let findings = recomputed(&dataset);
        assert!(
            findings.iter().all(|f| !f.contains("`atlas.shapes`")),
            "stub first: {stub_first}: {findings:#?}"
        );
    }
}

#[test]
fn a_python_class_level_import_is_a_class_member() {
    let mut dataset = load("semantic-oracle-profile-python.example.yml");
    for (to, name) in [("atlas.fast", "helper"), ("ext.importlib", "loader")] {
        push(
            &mut dataset,
            "/relations",
            json!({"kind": "exports", "from": "atlas.shapes.Circle", "to": to, "name": name, "visibility": "import"}),
        );
    }
    let findings = recomputed(&dataset).join("\n");
    assert!(
        findings.contains("missing [`atlas.shapes.Circle.helper`]"),
        "{findings}"
    );
    assert!(
        findings.contains("makes `ext.importlib` a member of `atlas-dist` with paths [`atlas.shapes.Circle.loader`]"),
        "{findings}"
    );
}

#[test]
fn a_typescript_static_member_has_a_static_segment() {
    let mut dataset = load("semantic-oracle-profile-typescript.example.yml");
    let mut member = entity(
        "shapes.Circle.static-unit",
        "field",
        "static-field",
        "\"src/shapes.ts\"#Circle.static:unit",
        "shapes.Circle",
        "shapes-ts",
    );
    member["name"] = json!("unit");
    member["declared-visibility"] = json!({"value": "public", "explicit": false});
    push(&mut dataset, "/entities", member);
    let findings = recomputed(&dataset).join("\n");
    assert!(
        findings.contains("[`import:@acme/geometry#Circle.static:unit`, `require:@acme/geometry#Circle.static:unit`]"),
        "{findings}"
    );
}

#[test]
fn a_rust_type_alias_names_the_inherent_items_of_its_type() {
    let mut dataset = load("semantic-oracle-profile-rust.example.yml");
    let mut block = entity(
        "geometry.shapes.impl-Circle",
        "extension",
        "impl",
        "geometry::shapes::impl geometry::shapes::Circle#1",
        "geometry.shapes",
        "geometry-lib",
    );
    block.as_object_mut().unwrap().remove("name");
    block["declared-visibility"] = json!({"value": "private", "explicit": false});
    block["effective-visibility"] = json!({
        "unit": {"access": "scoped", "scope": ["geometry.shapes"]},
        "project": "none", "external": "none"});
    push(&mut dataset, "/entities", block);
    let mut item = entity(
        "geometry.shapes.Circle.grow",
        "callable",
        "method",
        "geometry::shapes::Circle::grow",
        "geometry.shapes.impl-Circle",
        "geometry-lib",
    );
    item["declared-visibility"] = json!({"value": "pub", "explicit": true});
    push(&mut dataset, "/entities", item);
    let findings = recomputed(&dataset).join("\n");
    assert!(
        findings.contains("paths [`geometry::Scale::grow`, `geometry::shapes::Circle::grow`]"),
        "{findings}"
    );
}

#[test]
fn ambient_module_entries_of_one_distribution_have_distinct_keys() {
    let mut dataset = load("semantic-oracle-profile-typescript.example.yml");
    for specifier in ["alpha", "beta"] {
        push(
            &mut dataset,
            "/extensions/typescript:entry",
            json!({"distribution": "geometry", "mode": "import", "subpath": ".",
                   "specifier": specifier, "module": "shapes"}),
        );
    }
    let keys: Vec<_> = beholder_semantic_oracle::validate(&dataset, None)
        .findings
        .into_iter()
        .filter(|f| f.rule == "model:extension-key")
        .collect();
    assert!(keys.is_empty(), "{keys:#?}");
}

/// A unit with its root entity, which has no public names.
fn empty_container(dataset: &mut Value, unit_id: &str, unit: Value, root: Value) {
    let mut record = unit;
    record["id"] = json!(unit_id);
    push(dataset, "/units", record);
    push(dataset, "/entities", root);
}

#[test]
fn a_go_package_without_exported_names_is_a_member() {
    let mut dataset = load("semantic-oracle-profile-go.example.yml");
    let mut root = entity(
        "empty",
        "module",
        "package",
        "example.com/geometry/empty",
        "x",
        "empty-pkg",
    );
    root.as_object_mut().unwrap().remove("parent");
    empty_container(
        &mut dataset,
        "empty-pkg",
        json!({"kind": "package", "name": "example.com/geometry/empty",
               "distribution": "geometry-module", "root": "empty", "package-name": "empty"}),
        root,
    );
    let findings = recomputed(&dataset).join("\n");
    assert!(
        findings.contains(
            "makes `empty` a member of `geometry-module` with paths [`example.com/geometry/empty`]"
        ),
        "{findings}"
    );
}

#[test]
fn an_internal_go_package_is_not_a_member() {
    let dataset = load("semantic-oracle-profile-go.example.yml");
    let entries = dataset["public-api"]["members"].as_array().unwrap();
    assert!(entries.iter().all(|entry| entry["entity"] != "calc"));
    assert_eq!(recomputed(&dataset), Vec::<String>::new());
}

#[test]
fn a_dart_library_without_public_names_is_a_member() {
    let mut dataset = load("semantic-oracle-profile-dart.example.yml");
    let mut root = entity(
        "empty-library",
        "module",
        "library",
        "package:geometry/empty.dart",
        "x",
        "empty-lib",
    );
    root["declared-visibility"] = json!({"value": "public", "explicit": true});
    root.as_object_mut().unwrap().remove("parent");
    empty_container(
        &mut dataset,
        "empty-lib",
        json!({"kind": "library", "name": "package:geometry/empty.dart",
               "distribution": "geometry", "root": "empty-library", "language-version": "3.12"}),
        root,
    );
    let findings = recomputed(&dataset).join("\n");
    assert!(
        findings.contains("makes `empty-library` a member of `geometry` with paths [`package:geometry/empty.dart`]"),
        "{findings}"
    );
}

#[test]
fn a_dart_unnamed_constructor_is_a_member_as_new() {
    let mut dataset = load("semantic-oracle-profile-dart.example.yml");
    let members = dataset["public-api"]["members"].as_array_mut().unwrap();
    members.retain(|m| m["entity"] != "shapes.Circle.new");
    let findings = recomputed(&dataset).join("\n");
    assert!(
        findings.contains(
            "makes `shapes.Circle.new` a member of `geometry` with paths [`package:geometry/geometry.dart::Circle.new`]"
        ),
        "{findings}"
    );
}

#[test]
fn a_dart_deferred_prefix_load_library_is_not_a_member() {
    let mut dataset = load("semantic-oracle-profile-dart.example.yml");
    let mut load_library = entity(
        "geometry.loadLibrary",
        "callable",
        "function",
        "package:geometry/geometry.dart::loadLibrary",
        "geometry-library",
        "geometry-lib",
    );
    load_library["origin"] = json!("synthetic");
    load_library["declared-visibility"] = json!({"value": "public", "explicit": false});
    push(&mut dataset, "/entities", load_library);
    let findings = recomputed(&dataset).join("\n");
    assert!(!findings.contains("geometry.loadLibrary"), "{findings}");
}

#[test]
fn a_typescript_entry_module_without_exports_is_a_member() {
    let mut dataset = load("semantic-oracle-profile-typescript.example.yml");
    let mut root = entity(
        "empty",
        "module",
        "module",
        "\"src/empty.ts\"",
        "x",
        "empty-ts",
    );
    root.as_object_mut().unwrap().remove("parent");
    empty_container(
        &mut dataset,
        "empty-ts",
        json!({"kind": "module", "name": "src/empty.ts", "distribution": "geometry", "root": "empty"}),
        root,
    );
    for mode in ["import", "require"] {
        push(
            &mut dataset,
            "/extensions/typescript:entry",
            json!({"distribution": "geometry", "mode": mode, "subpath": "./empty", "module": "empty"}),
        );
    }
    let findings = recomputed(&dataset).join("\n");
    assert!(
        findings.contains(
            "makes `empty` a member of `geometry` with paths [`import:@acme/geometry/empty`, `require:@acme/geometry/empty`]"
        ),
        "{findings}"
    );
}

#[test]
fn a_csharp_namespace_is_a_member_through_a_public_type() {
    let mut dataset = load("semantic-oracle-profile-csharp.example.yml");
    let namespace = |id: &str, path: &str, parent: &str| {
        let mut record = entity(id, "namespace", "namespace", path, parent, "geometry-lib");
        record.as_object_mut().unwrap().remove("unit");
        record["units"] = json!(["geometry-lib"]);
        record["declared-visibility"] = json!({"value": "public", "explicit": false});
        record
    };
    push(
        &mut dataset,
        "/entities",
        namespace("Geometry.Outer", "Geometry.Outer", "Geometry"),
    );
    push(
        &mut dataset,
        "/entities",
        namespace(
            "Geometry.Outer.Inner",
            "Geometry.Outer.Inner",
            "Geometry.Outer",
        ),
    );
    push(
        &mut dataset,
        "/entities",
        namespace("Geometry.Hidden", "Geometry.Hidden", "Geometry"),
    );
    let mut public_type = entity(
        "Geometry.Outer.Inner.Visible",
        "type",
        "class",
        "Geometry.Outer.Inner.Visible",
        "Geometry.Outer.Inner",
        "geometry-lib",
    );
    public_type["declared-visibility"] = json!({"value": "public", "explicit": true});
    push(&mut dataset, "/entities", public_type);
    let mut internal_type = entity(
        "Geometry.Hidden.Secret",
        "type",
        "class",
        "Geometry.Hidden.Secret",
        "Geometry.Hidden",
        "geometry-lib",
    );
    internal_type["declared-visibility"] = json!({"value": "internal", "explicit": true});
    internal_type["effective-visibility"] =
        json!({"unit": "all", "project": "none", "external": "none"});
    push(&mut dataset, "/entities", internal_type);
    let findings = recomputed(&dataset).join("\n");
    assert!(
        findings.contains("makes `Geometry.Outer` a member"),
        "{findings}"
    );
    assert!(
        findings.contains("makes `Geometry.Outer.Inner` a member"),
        "{findings}"
    );
    assert!(!findings.contains("Geometry.Hidden"), "{findings}");
}

#[test]
fn a_forwarded_project_type_brings_its_namespaces() {
    let mut dataset = load("semantic-oracle-profile-csharp.example.yml");
    let mut distribution = dataset["distributions"][0].clone();
    distribution["id"] = json!("other");
    distribution["name"] = json!("Other");
    push(&mut dataset, "/distributions", distribution);
    let mut unit = dataset["units"][0].clone();
    unit["id"] = json!("other-lib");
    unit["name"] = json!("Other");
    unit["distribution"] = json!("other");
    unit["project"] = json!("src/Other/Other.csproj");
    push(&mut dataset, "/units", unit);
    let mut namespace = entity(
        "Other.Ns",
        "namespace",
        "namespace",
        "Other.Ns",
        "x",
        "other-lib",
    );
    namespace.as_object_mut().unwrap().remove("parent");
    namespace.as_object_mut().unwrap().remove("unit");
    namespace["units"] = json!(["other-lib"]);
    namespace["declared-visibility"] = json!({"value": "public", "explicit": false});
    push(&mut dataset, "/entities", namespace);
    let mut thing = entity(
        "Other.Ns.Thing",
        "type",
        "class",
        "Other.Ns.Thing",
        "Other.Ns",
        "other-lib",
    );
    thing["declared-visibility"] = json!({"value": "public", "explicit": true});
    push(&mut dataset, "/entities", thing);
    let site = json!({"path": "src/Geometry/Shapes.cs", "start": [1, 11], "end": [1, 19], "text": "Geometry"});
    push(
        &mut dataset,
        "/extensions/csharp:type-forward",
        json!({"unit": "geometry-lib", "type": "Other.Ns.Thing", "at": site}),
    );
    let findings = recomputed(&dataset).join("\n");
    assert!(
        findings.contains("makes `Other.Ns` a member of `geometry`"),
        "{findings}"
    );
    assert!(
        findings.contains("makes `Other.Ns.Thing` a member of `geometry`"),
        "{findings}"
    );
    // The forwarded external type `Legacy.Point` adds no namespace.
    let example = load("semantic-oracle-profile-csharp.example.yml");
    assert_eq!(recomputed(&example), Vec::<String>::new());
}

/// Every example states signatures for its public API members: a record names
/// a member in the contexts it lists, and a member of a kind with slots has a
/// record unless it has nothing to state.
#[test]
fn every_example_states_signatures_for_its_members() {
    const SLOT_KINDS: [&str; 8] = [
        "callable",
        "type",
        "type-alias",
        "field",
        "property",
        "variable",
        "constant",
        "enum-member",
    ];
    // Plain declarations of these members state nothing beyond their names.
    const NOTHING_TO_STATE: [&str; 9] = [
        "atlas.portable.run",
        "stub.atlas.shapes.Shape",
        "Geometry.IShape",
        "Geometry.Generated.Metrics",
        "Geometry.Header",
        "shapes.Kind",
        "Geometry.Shape..ctor",
        "Geometry.Visitor",
        "ext.Legacy.Point",
    ];
    for example in assets(".example.yml") {
        let dataset = load(&example);
        let members = dataset["public-api"]["members"].as_array().unwrap();
        let signatures = dataset["public-api"]["signatures"].as_array().unwrap();
        assert!(!signatures.is_empty(), "{example}");
        let member_ids: BTreeSet<&str> = members
            .iter()
            .map(|member| member["entity"].as_str().unwrap())
            .collect();
        let stated: BTreeSet<&str> = signatures
            .iter()
            .map(|signature| signature["entity"].as_str().unwrap())
            .collect();
        for id in &stated {
            assert!(member_ids.contains(id), "{example}: `{id}` is no member");
        }
        for entity in dataset["entities"].as_array().unwrap() {
            let id = entity["id"].as_str().unwrap();
            let kind = entity["kind"].as_str().unwrap();
            if member_ids.contains(id)
                && SLOT_KINDS.contains(&kind)
                && !NOTHING_TO_STATE.contains(&id)
            {
                assert!(stated.contains(id), "{example}: `{id}` has no record");
            }
        }
    }
}

/// Each signature form that the examples' members do not use is schema-valid
/// for its profile.
#[test]
fn every_signature_form_is_schema_valid_for_its_profile() {
    let at = json!({"path": "x", "start": [1, 1], "end": [1, 2], "text": "x"});
    let forms = [
        (
            "rust",
            json!({"entity": "x", "impl": {
                "type-parameters": [{"name": "T", "kind": "type", "bounds": "@{C}"}],
                "self-type": "@{X}<T>", "where": "@{V}<T>: @{C}"},
                "parameters": [{"name": "self", "mode": "self-mut-ref"}, {"name": "n", "type": "u8"}],
                "modifiers": ["async", "const", "unsafe"], "returns": "T"}),
        ),
        // `extern "C" fn` and a C variadic tail.
        (
            "rust",
            json!({"entity": "x", "abi": "C", "modifiers": ["unsafe"],
                "parameters": [{"name": "count", "type": "i32"}, {"mode": "c-variadic"}]}),
        ),
        // An implementation that no type's `bases` holds: a blanket one, and one
        // for an external self type.
        (
            "rust",
            json!({"entity": "x", "implementations": [
                {"type": "@{Tr}", "self-type": "T",
                 "type-parameters": [{"name": "T", "kind": "type", "bounds": "@{D}"}],
                 "bindings": [{"name": "Item", "type": "T"}]},
                {"type": "@{Tr}", "self-type": "@{V}<T>", "where": "@{V}<T>: @{C}"}]}),
        ),
        (
            "rust",
            json!({"entity": "x", "type-parameters": [
                    {"name": "'a", "kind": "lifetime"},
                    {"name": "N", "kind": "const", "type": "usize", "default": "10"}],
                "modifiers": ["auto"],
                "bases": [{"type": "@{T}<u8>", "self-type": "@{X}<U>",
                    "type-parameters": [{"name": "U", "kind": "type"}], "where": "@{V}<U>: @{C}",
                    "bindings": [
                        {"name": "Item", "type": "u8"},
                        {"name": "Out", "type-parameters": [{"name": "'b", "kind": "lifetime"}], "where": "U: 'b", "type": "&'b U"},
                        {"name": "SIZE", "value": "4"}]}]}),
        ),
        (
            "rust",
            json!({"entity": "x", "bounds": "@{C}", "type": "u8"}),
        ),
        (
            "typescript",
            json!({"entity": "x", "overloads": [
                {"type-parameters": [{"name": "T", "kind": "type", "variance": "in out", "modifiers": ["const"], "default": "string"}],
                 "parameters": [{"name": "v", "type": "T"}], "returns": "T"},
                {"parameters": [{"type": "string", "mode": "rest"}], "returns": "string"},
                {}]}),
        ),
        (
            "typescript",
            json!({"entity": "x", "constructors": [{"access": "protected"}, {"parameters": [{"name": "a", "type": {"inferred": "number"}}]}],
                "call-signatures": [{"returns": "void"}],
                "construct-signatures": [{"returns": "@{X}"}],
                "index-signatures": [{"parameters": [{"name": "k", "type": "string"}], "returns": "number", "modifiers": ["readonly"]}],
                "get": {"type": "string", "access": "protected"}, "set": {"type": "string"},
                "modifiers": ["abstract", "optional", "static"]}),
        ),
        ("typescript", json!({"entity": "x", "value": "\"a\""})),
        // A default before a required parameter emits no `?`.
        (
            "typescript",
            json!({"entity": "x", "parameters": [
                {"name": "first", "type": {"inferred": "number | undefined"}},
                {"name": "second", "type": "string"}]}),
        ),
        // A legacy `TypeVar` with constraints, a variance and a default.
        (
            "python",
            json!({"entity": "x", "type-parameters": [
                {"name": "T", "kind": "type", "bounds": "(int, str)", "variance": "in", "default": "int"}]}),
        ),
        (
            "python",
            json!({"entity": "x", "alternatives": [
                {"at": at, "parameters": [{"name": "a", "type": "int", "default": "present", "mode": "keyword-only"}], "returns": "int"},
                {"at": at, "overloads": [{"returns": "int"}, {"returns": "str"}]},
                {"at": at}]}),
        ),
        (
            "python",
            json!({"entity": "x", "parameters": [
                    {"name": "a", "mode": "positional-only"},
                    {"name": "args", "mode": "var-positional"},
                    {"name": "kwargs", "mode": "var-keyword"}],
                "type-parameters": [{"name": "T", "kind": "type", "bounds": "int", "default": "bool"}],
                "modifiers": ["abstract", "classvar", "final", "not-required"],
                "get": {"type": "int"}, "set": {"type": "int"}, "delete": "present", "value": "1"}),
        ),
        (
            "go",
            json!({"entity": "x", "type-parameters": [{"name": "T", "kind": "type", "bounds": "~int | ~string"}],
                "receiver": "@{X}[T]", "parameters": [{"type": "int"}], "returns": "(int, error)",
                "tag": "\"json\"", "value": "(5/2 + 3i)", "type": {"inferred": "int"}}),
        ),
        (
            "csharp",
            json!({"entity": "x", "type-parameters": [{"name": "T", "kind": "type", "variance": "out", "bounds": "class, new()"}],
                "parameters": [
                    {"name": "a", "mode": "ref readonly", "type": "string~"},
                    {"name": "b", "mode": "this ref", "type": "(int x, string? y)"},
                    {"name": "c", "mode": "params", "type": "int[]", "default": "null"},
                    {"name": "d", "mode": "scoped ref", "type": "@{S}"},
                    {"name": "e", "mode": "scoped", "type": "@{S}"}],
                "return-mode": "ref readonly", "returns": "int",
                "add": {"type": "@{H}?"}, "remove": {"type": "@{H}?"},
                "get": {"type": "int", "access": "protected internal", "modifiers": ["readonly"]},
                "set": {"type": "int", "access": "private"}, "init": {"type": "int"},
                "modifiers": ["override", "required", "sealed"]}),
        ),
        (
            "dart",
            json!({"entity": "x", "type-parameters": [{"name": "T", "kind": "type", "bounds": "num"}],
                "parameters": [
                    {"name": "a", "mode": "optional", "default": "null"},
                    {"name": "b", "mode": "required-named", "type": {"inferred": "int"}, "default": {"inferred": "1"}}],
                "underlying": "int", "get": {"type": "int"}, "set": {"type": "int", "modifiers": ["covariant"]},
                "receiver": "@{List}<T>", "bounds": "@{Engine}",
                "modifiers": ["abstract", "base", "const", "covariant", "external", "interface", "late", "mixin", "sealed", "static"]}),
        ),
    ];
    let validator = dataset_validator().expect("build the validator");
    for (language, form) in forms {
        let mut dataset = load(&format!("semantic-oracle-profile-{language}.example.yml"));
        dataset["public-api"]["signatures"]
            .as_array_mut()
            .unwrap()
            .push(form);
        let found = errors(&validator, &dataset);
        assert!(found.is_empty(), "{language}:\n{}", describe(&found));
    }
}

/// YAML 1.2 reads `N` as a string, as a Rust const parameter's name needs.
#[test]
fn a_bare_name_that_older_yaml_reads_as_a_boolean_is_a_string() {
    let value = parse_yaml("{name: N, kind: const, other: y, flag: true}").unwrap();
    assert_eq!(
        value,
        json!({"name": "N", "kind": "const", "other": "y", "flag": true})
    );
}

/// An alternative's site gets the same locator checks as any other site.
#[test]
fn an_alternative_site_is_an_ordinary_locator() {
    let mut dataset = load("semantic-oracle-profile-python.example.yml");
    let site = dataset["entities"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entity| entity["id"] == "atlas.fast.run")
        .unwrap()["definitions"][0]["at"]
        .clone();
    let mut reversed = site.clone();
    reversed["start"] = json!([9, 2]);
    reversed["end"] = json!([1, 1]);
    let mut elsewhere = site.clone();
    elsewhere["path"] = json!("other.py");
    dataset["public-api"]["signatures"][0] = json!({"entity": "atlas.fast.run", "alternatives": [
        {"at": reversed, "returns": "int"}, {"at": elsewhere, "returns": "str"}]});
    let report = beholder_semantic_oracle::validate(&dataset, None);
    let found: Vec<(&str, &str)> = report
        .findings
        .iter()
        .map(|f| (f.rule.as_str(), f.at.as_str()))
        .collect();
    assert_eq!(
        found,
        vec![
            (
                "model:locator-order",
                "/public-api/signatures/0/alternatives/0/at"
            ),
            (
                "model:locator-document",
                "/public-api/signatures/0/alternatives/1/at/path"
            ),
        ]
    );
}

/// A conditional auto trait is one string, so that a changed condition is one
/// `wrong-value` on the key `auto-traits[Send]`.
#[test]
fn an_auto_traits_entry_may_state_a_condition() {
    let validator = dataset_validator().expect("build the validator");
    let mut dataset = load("semantic-oracle-profile-rust.example.yml");
    dataset["public-api"]["signatures"]
        .as_array_mut()
        .unwrap()
        .push(json!({"entity": "x", "auto-traits": [
            "RefUnwindSafe when T: RefUnwindSafe, U: RefUnwindSafe",
            "Send when T: @{ext.core.clone.Clone}",
            "Sync when <T as @{Tr}>::Item: Sync",
            "Unpin"]}));
    dataset["public-api"]["signatures"]
        .as_array_mut()
        .unwrap()
        .push(json!({"entity": "x", "auto-traits": [
            "#0 Send when T: Send", "#1 Sync", "#10 Unpin",
            "Send when T = u8"],
            "bases": [{"type": "@{Tr}", "bindings": [
                {"name": "make", "auto-traits": ["#0 Send"]},
                {"name": "refined", "returns": "u8"},
                {"name": "refined_opaque", "returns": "(impl @{A}, u8)", "auto-traits": ["#0 Send"]}]}]}));
    let found = errors(&validator, &dataset);
    assert!(found.is_empty(), "{}", describe(&found));
}

// ------------------------------------------------- the source-declaration table

/// The messages of one rule on a dataset, without project files.
fn rule_messages(dataset: &Value, rule: &str) -> Vec<String> {
    beholder_semantic_oracle::validate(dataset, None)
        .findings
        .into_iter()
        .filter(|f| f.rule == rule)
        .map(|f| format!("{} {}", f.at, f.message))
        .collect()
}

const REFERENCES: &str = "model:signature-references";

fn type_reference(
    path: &str,
    line: u64,
    column: u64,
    name: &str,
    from: &str,
    target: &str,
) -> Value {
    json!({
        "at": {"path": path, "start": [line, column], "end": [line, column + name.len() as u64], "text": name},
        "from": from, "target": target, "role": "type",
    })
}

/// Sets a value at a pointer, adding it to its object when it is new.
fn set_at(dataset: &mut Value, pointer: &str, value: Value) {
    if let Some(found) = dataset.pointer_mut(pointer) {
        *found = value;
        return;
    }
    let (parent, member) = pointer.rsplit_once('/').expect("a pointer has a member");
    dataset
        .pointer_mut(parent)
        .and_then(Value::as_object_mut)
        .unwrap_or_else(|| panic!("no object {parent}"))
        .insert(member.to_owned(), value);
}

/// The index of a member's signature record.
fn record_of(dataset: &Value, entity: &str) -> usize {
    dataset["public-api"]["signatures"]
        .as_array()
        .unwrap()
        .iter()
        .position(|record| record["entity"] == entity)
        .unwrap_or_else(|| panic!("no record of {entity}"))
}

/// Constructor 1 reads constructor 1's references, not constructor 0's.
#[test]
fn a_group_entry_reads_its_own_constructor() {
    let mut dataset = load("semantic-oracle-profile-typescript.example.yml");
    let mut second = dataset["entities"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["id"] == "shapes.Circle.constructor")
        .unwrap()
        .clone();
    second["id"] = json!("shapes.Circle.constructor2");
    second["path"] = json!("\"src/shapes.ts\"#Circle.constructor#2");
    second["definitions"] = json!([{"at": {"path": "src/shapes.ts", "start": [12, 3], "end": [12, 14], "text": "constructor"}}]);
    push(&mut dataset, "/entities", second);
    let at = record_of(&dataset, "shapes.Circle");
    set_at(
        &mut dataset,
        &format!("/public-api/signatures/{at}/constructors"),
        json!([
            {"parameters": [{"name": "radius", "type": "number"}]},
            {"returns": "@{shapes.Kind}"},
        ]),
    );
    let mut sibling = dataset.clone();
    push(
        &mut sibling,
        "/references",
        type_reference(
            "src/shapes.ts",
            11,
            40,
            "Kind",
            "shapes.Circle.constructor",
            "shapes.Kind",
        ),
    );
    let found = rule_messages(&sibling, REFERENCES);
    assert!(
        found.iter().any(|m| m.contains("constructors/1/returns")),
        "a reference under constructor 0 does not state constructor 1: {found:#?}"
    );
    let mut own = dataset.clone();
    push(
        &mut own,
        "/references",
        type_reference(
            "src/shapes.ts",
            12,
            40,
            "Kind",
            "shapes.Circle.constructor2",
            "shapes.Kind",
        ),
    );
    assert_eq!(rule_messages(&own, REFERENCES), Vec::<String>::new());
}

/// Overload 1 reads the references at its own signature site.
#[test]
fn an_overload_reads_its_own_signature_site() {
    let mut dataset = load("semantic-oracle-profile-typescript.example.yml");
    let entities = dataset["entities"].as_array_mut().unwrap();
    let area = entities
        .iter_mut()
        .find(|e| e["id"] == "shapes.Circle.area")
        .unwrap();
    area["definitions"] = json!([
        {"at": {"path": "src/shapes.ts", "start": [13, 3], "end": [13, 7], "text": "area"}},
        {"at": {"path": "src/shapes.ts", "start": [15, 3], "end": [15, 7], "text": "area"}},
    ]);
    let at = record_of(&dataset, "shapes.Circle.area");
    set_at(
        &mut dataset,
        &format!("/public-api/signatures/{at}"),
        json!({"entity": "shapes.Circle.area", "overloads": [
            {"returns": "number"}, {"returns": "@{shapes.Kind}"}]}),
    );
    let mut first_site = dataset.clone();
    push(
        &mut first_site,
        "/references",
        type_reference(
            "src/shapes.ts",
            13,
            12,
            "Kind",
            "shapes.Circle.area",
            "shapes.Kind",
        ),
    );
    let found = rule_messages(&first_site, REFERENCES);
    assert!(
        found.iter().any(|m| m.contains("overloads/1/returns")),
        "{found:#?}"
    );
    let mut own_site = dataset.clone();
    push(
        &mut own_site,
        "/references",
        type_reference(
            "src/shapes.ts",
            15,
            12,
            "Kind",
            "shapes.Circle.area",
            "shapes.Kind",
        ),
    );
    assert_eq!(rule_messages(&own_site, REFERENCES), Vec::<String>::new());
}

/// An alternative reads the references at its own definition.
#[test]
fn an_alternative_reads_its_own_definition() {
    let mut dataset = load("semantic-oracle-profile-python.example.yml");
    let first = json!({"path": "src/atlas/fast.py", "start": [3, 5], "end": [3, 8], "text": "run"});
    let second =
        json!({"path": "src/atlas/fast.py", "start": [6, 5], "end": [6, 8], "text": "run"});
    let entities = dataset["entities"].as_array_mut().unwrap();
    let run = entities
        .iter_mut()
        .find(|e| e["id"] == "atlas.fast.run")
        .unwrap();
    run["definitions"] = json!([{"at": first}, {"at": second}]);
    let at = record_of(&dataset, "atlas.fast.run");
    set_at(
        &mut dataset,
        &format!("/public-api/signatures/{at}"),
        json!({"entity": "atlas.fast.run", "alternatives": [
            {"at": first, "returns": "None"},
            {"at": second, "returns": "@{ext.builtins.float}"}]}),
    );
    let mut wrong = dataset.clone();
    push(
        &mut wrong,
        "/references",
        type_reference(
            "src/atlas/fast.py",
            3,
            15,
            "float",
            "atlas.fast.run",
            "ext.builtins.float",
        ),
    );
    let found = rule_messages(&wrong, REFERENCES);
    assert!(
        found.iter().any(|m| m.contains("alternatives/1/returns")),
        "{found:#?}"
    );
    let mut own = dataset.clone();
    push(
        &mut own,
        "/references",
        type_reference(
            "src/atlas/fast.py",
            6,
            15,
            "float",
            "atlas.fast.run",
            "ext.builtins.float",
        ),
    );
    assert_eq!(rule_messages(&own, REFERENCES), Vec::<String>::new());
}

/// A Rust base entry reads the implementation of that trait for that type, not
/// another extension in the dataset.
#[test]
fn a_rust_base_reads_its_own_implementation() {
    let mut dataset = load("semantic-oracle-profile-rust.example.yml");
    let at = record_of(&dataset, "geometry.shapes.Circle");
    // `ext.core.fmt.Display` has a reference only in `impl<T: Display> Area for T`.
    set_at(
        &mut dataset,
        &format!("/public-api/signatures/{at}/bases/0/where"),
        json!("@{ext.core.fmt.Display}: @{geometry.shapes.Area}"),
    );
    let found = rule_messages(&dataset, REFERENCES);
    assert!(
        found
            .iter()
            .any(|m| m.contains("bases/0/where") && m.contains("ext.core.fmt.Display")),
        "{found:#?}"
    );
    push(
        &mut dataset,
        "/references",
        type_reference(
            "src/shapes.rs",
            17,
            30,
            "Display",
            "geometry.shapes.impl-Area-for-Circle",
            "ext.core.fmt.Display",
        ),
    );
    assert_eq!(rule_messages(&dataset, REFERENCES), Vec::<String>::new());
}

/// A trait's `implementations` entry reads its own block too.
#[test]
fn an_implementations_entry_reads_its_own_block() {
    let dataset = load("semantic-oracle-profile-rust.example.yml");
    let at = record_of(&dataset, "geometry.shapes.Area");
    let mut other = dataset.clone();
    // `ext.core.clone.Clone` has references only in `Pair`.
    set_at(
        &mut other,
        &format!("/public-api/signatures/{at}/implementations/0"),
        json!({"type": "@{geometry.shapes.Area}", "self-type": "T", "where": "T: @{ext.core.clone.Clone}"}),
    );
    let found = rule_messages(&other, REFERENCES);
    assert!(
        found.iter().any(|m| m.contains("implementations/0/where")),
        "{found:#?}"
    );
}

/// An `impl` slot reads its own block.
#[test]
fn an_impl_slot_reads_the_member_block() {
    let mut dataset = load("semantic-oracle-profile-rust.example.yml");
    let block = json!({"id": "geometry.shapes.impl-Pair", "kind": "extension", "language-kind": "impl",
        "path": "geometry::shapes::impl geometry::shapes::Pair#1", "unit": "geometry-lib",
        "parent": "geometry.shapes", "origin": "source", "definitions": [],
        "declared-visibility": {"value": "private", "explicit": false},
        "effective-visibility": {"unit": {"access": "scoped", "scope": ["geometry.shapes"]},
                                 "project": "none", "external": "none"}});
    let method = json!({"id": "geometry.shapes.Pair.get", "kind": "callable", "language-kind": "method",
        "name": "get", "path": "geometry::shapes::Pair::get", "unit": "geometry-lib",
        "parent": "geometry.shapes.impl-Pair", "origin": "source",
        "definitions": [{"at": {"path": "src/shapes.rs", "start": [30, 12], "end": [30, 15], "text": "get"}}],
        "declared-visibility": {"value": "pub", "explicit": true},
        "effective-visibility": {"unit": "all", "project": "all", "external": "all"}});
    push(&mut dataset, "/entities", block);
    push(&mut dataset, "/entities", method);
    push(
        &mut dataset,
        "/public-api/members",
        json!({"distribution": "geometry", "entity": "geometry.shapes.Pair.get",
               "paths": ["geometry::shapes::Pair::get"], "access": "all"}),
    );
    push(
        &mut dataset,
        "/public-api/signatures",
        json!({"entity": "geometry.shapes.Pair.get",
               "impl": {"self-type": "@{geometry.shapes.Pair}<T>"}, "returns": "u8"}),
    );
    // A reference elsewhere does not state the block's self type.
    let mut outside = dataset.clone();
    push(
        &mut outside,
        "/references",
        type_reference(
            "src/shapes.rs",
            28,
            6,
            "Pair",
            "geometry.shapes",
            "geometry.shapes.Pair",
        ),
    );
    let found = rule_messages(&outside, REFERENCES);
    assert!(
        found.iter().any(|m| m.contains("/impl/self-type")),
        "{found:#?}"
    );
    let mut own = dataset.clone();
    push(
        &mut own,
        "/references",
        type_reference(
            "src/shapes.rs",
            29,
            6,
            "Pair",
            "geometry.shapes.impl-Pair",
            "geometry.shapes.Pair",
        ),
    );
    assert_eq!(rule_messages(&own, REFERENCES), Vec::<String>::new());
}

/// A same-named project type keeps its `@{id}` through the validator's resolver.
#[test]
fn the_validator_rewrites_only_a_standard_external() {
    let mut dataset = load("semantic-oracle-profile-typescript.example.yml");
    let at = record_of(&dataset, "shapes.Circle.area");
    // `ext.lib.es5.Array` is the standard declaration, so its spelling is `string[]`.
    set_at(
        &mut dataset,
        &format!("/public-api/signatures/{at}/returns"),
        json!("@{ext.lib.es5.Array}<string>"),
    );
    let found = rule_messages(&dataset, "model:signature-canonical");
    assert!(
        found
            .iter()
            .any(|m| m.contains("its one spelling is `string[]`")),
        "{found:#?}"
    );
    // A project type's path can end in the same name.
    let mut project = load("semantic-oracle-profile-typescript.example.yml");
    set_at(
        &mut project,
        &format!("/public-api/signatures/{at}/returns"),
        json!("@{shapes.Array}<string>"),
    );
    let mut entity = project["entities"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["id"] == "shapes.Shape")
        .unwrap()
        .clone();
    entity["id"] = json!("shapes.Array");
    entity["name"] = json!("Array");
    entity["path"] = json!("typescript/lib/lib.es5.d.ts#Array");
    push(&mut project, "/entities", entity);
    let found = rule_messages(&project, "model:signature-canonical");
    assert_eq!(found, Vec::<String>::new());
}

/// Nested sets are sorted at every level, in a group entry too.
#[test]
fn a_group_entry_has_sorted_modifiers() {
    let mut dataset = load("semantic-oracle-profile-typescript.example.yml");
    let at = record_of(&dataset, "shapes.Circle");
    set_at(
        &mut dataset,
        &format!("/public-api/signatures/{at}/constructors/0/modifiers"),
        json!(["static", "abstract"]),
    );
    let found = rule_messages(&dataset, "model:signature-canonical");
    assert!(
        found.iter().any(|m| m.contains("constructors/0/modifiers")),
        "{found:#?}"
    );
}

/// A Rust example with an `implements` relation at `at` and a base for it, whose
/// block no source declaration holds. Returns the dataset and the relation's site.
fn base_with_no_block(token: &str) -> (Value, Value) {
    let mut dataset = load("semantic-oracle-profile-rust.example.yml");
    let at = json!({"path": "src/shapes.rs", "start": [40, 10], "end": [40, 10 + token.len() as u64], "text": token});
    push(
        &mut dataset,
        "/relations",
        json!({"kind": "implements", "from": "geometry.shapes.Circle", "to": "geometry.shapes.Solid", "at": at}),
    );
    let index = record_of(&dataset, "geometry.shapes.Circle");
    push(
        &mut dataset,
        &format!("/public-api/signatures/{index}/bases"),
        json!({"type": "@{geometry.shapes.Solid}"}),
    );
    (dataset, at)
}

const BASE_REFERENCE: &str = "`@{geometry.shapes.Solid}` has no `type` reference";

/// A block inside a function body has no entity. Its base reads the `type` reference
/// at the relation's site, whose `from` is the enclosing function.
#[test]
fn a_body_local_base_reads_the_reference_at_its_relation() {
    let (base, _) = base_with_no_block("Solid");
    let found = rule_messages(&base, REFERENCES);
    assert!(
        found.iter().any(|m| m.contains(BASE_REFERENCE)),
        "with no reference: {found:#?}"
    );
    let reference = |line, from: &str| {
        type_reference(
            "src/shapes.rs",
            line,
            10,
            "Solid",
            from,
            "geometry.shapes.Solid",
        )
    };
    let mut own = base.clone();
    push(
        &mut own,
        "/references",
        reference(40, "geometry.make_hidden"),
    );
    assert_eq!(rule_messages(&own, REFERENCES), Vec::<String>::new());
    // At another site, or from something other than the enclosing function, it is not.
    let mut elsewhere = base.clone();
    push(
        &mut elsewhere,
        "/references",
        reference(41, "geometry.make_hidden"),
    );
    let found = rule_messages(&elsewhere, REFERENCES);
    assert!(
        found.iter().any(|m| m.contains(BASE_REFERENCE)),
        "another site: {found:#?}"
    );
    let mut module = base.clone();
    push(&mut module, "/references", reference(40, "geometry.shapes"));
    let found = rule_messages(&module, REFERENCES);
    assert!(
        found.iter().any(|m| m.contains(BASE_REFERENCE)),
        "from a module: {found:#?}"
    );
}

/// A derive's base has no `type` token. It is audited against the derive's token and
/// the generation rule, and a `type` reference is not fabricated for it.
#[test]
fn a_derived_base_reads_the_derive_token_and_the_generation_rule() {
    const MACRO: &str = "ext.core.fmt.macros.Debug";
    let derived = |macro_of_fact: &str, role: &str, origin: &str| {
        let (mut dataset, at) = base_with_no_block("Debug");
        push(
            &mut dataset,
            "/references",
            json!({"at": at, "from": "geometry.shapes", "target": MACRO, "role": role}),
        );
        let block = json!({"id": "geometry.shapes.Circle.impl-Debug", "kind": "extension",
            "language-kind": "trait-impl", "path": "geometry::shapes::impl Debug for Circle#1",
            "unit": "geometry-lib", "parent": "geometry.shapes", "origin": origin,
            "definitions": [], "anchor": at,
            "generated-by": {"generator": MACRO, "at": at},
            "declared-visibility": {"value": "private", "explicit": false},
            "effective-visibility": {"unit": {"access": "scoped", "scope": ["geometry.shapes"]},
                                     "project": "none", "external": "none"}});
        let mut block = block;
        if origin != "generated" {
            // The schema gives `generated-by` to a generated entity only.
            block.as_object_mut().unwrap().remove("generated-by");
        }
        push(&mut dataset, "/entities", block);
        push(
            &mut dataset,
            "/extensions/rust:macro-expansion",
            json!({"entity": "geometry.shapes.Circle.impl-Debug", "macro": macro_of_fact, "invocation": at}),
        );
        dataset
    };
    assert_eq!(
        rule_messages(&derived(MACRO, "attribute", "generated"), REFERENCES),
        Vec::<String>::new()
    );
    // A macro invocation generates a block the same way.
    assert_eq!(
        rule_messages(&derived(MACRO, "macro-invocation", "generated"), REFERENCES),
        Vec::<String>::new()
    );
    for (dataset, why) in [
        (
            derived("ext.std", "attribute", "generated"),
            "the fact names another macro",
        ),
        (
            derived(MACRO, "read", "generated"),
            "no token names the macro",
        ),
        (
            derived(MACRO, "attribute", "source"),
            "the block is not generated",
        ),
    ] {
        let found = rule_messages(&dataset, REFERENCES);
        assert!(
            found.iter().any(|m| m.contains(BASE_REFERENCE)),
            "{why}: {found:#?}"
        );
    }
    // With no generated block at all, the derive's token alone states nothing.
    let (mut bare, at) = base_with_no_block("Debug");
    push(
        &mut bare,
        "/references",
        json!({"at": at, "from": "geometry.shapes", "target": MACRO, "role": "attribute"}),
    );
    let found = rule_messages(&bare, REFERENCES);
    assert!(
        found.iter().any(|m| m.contains(BASE_REFERENCE)),
        "{found:#?}"
    );
}

// ------------------------------------------------------- Python bindings

/// `atlas.fast` with two star imports that bind `shared`, in statement order.
fn two_star_imports(first: &str, second: &str) -> Value {
    let mut dataset = load("semantic-oracle-profile-python.example.yml");
    for (line, target) in [(10, first), (11, second)] {
        push(
            &mut dataset,
            "/relations",
            json!({"kind": "exports", "from": "atlas.fast", "to": target, "name": "shared",
                   "visibility": "re-export",
                   "at": {"path": "src/atlas/fast.py", "start": [line, 20], "end": [line, 21], "text": "*"}}),
        );
    }
    dataset
}

/// At run time the last star import wins, and both statements keep their relations.
#[test]
fn a_name_two_star_imports_bind_is_its_last_binding() {
    let findings = recomputed(&two_star_imports("atlas.portable.run", "atlas.fast.run"));
    let joined = findings.join("\n");
    assert!(
        joined.contains("`atlas.fast.shared`") && joined.contains("`atlas.fast.run`"),
        "{joined}"
    );
    assert!(
        !joined
            .lines()
            .any(|line| line.contains("`atlas.portable.run`")
                && line.contains("`atlas.fast.shared`")),
        "the earlier star import gives no path: {joined}"
    );
    // In the other statement order, the other entity is the member.
    let findings = recomputed(&two_star_imports("atlas.fast.run", "atlas.portable.run"));
    let joined = findings.join("\n");
    assert!(
        joined.contains("`atlas.portable.run`") && joined.contains("`atlas.fast.shared`"),
        "{joined}"
    );
    assert!(
        !joined
            .lines()
            .any(|line| line.contains("`atlas.fast.run`") && line.contains("`atlas.fast.shared`")),
        "the earlier star import gives no path: {joined}"
    );
}

/// A declaration after an import binds the name last, and an import after a
/// declaration replaces it.
#[test]
fn a_declaration_and_an_import_of_one_name_read_statement_order() {
    // `atlas.fast.run` is declared at line 3.
    let import_at = |line: u64| {
        let mut dataset = load("semantic-oracle-profile-python.example.yml");
        push(
            &mut dataset,
            "/relations",
            json!({"kind": "exports", "from": "atlas.fast", "to": "atlas.portable.run", "name": "run",
                   "visibility": "re-export",
                   "at": {"path": "src/atlas/fast.py", "start": [line, 20], "end": [line, 23], "text": "run"}}),
        );
        recomputed(&dataset).join("\n")
    };
    // The declaration binds last, so the example's membership stands as it is.
    assert_eq!(import_at(1), "");
    // The import binds last, so the name now denotes its target.
    let after = import_at(9);
    assert!(
        after.contains("`atlas.portable.run`") && after.contains("`atlas.fast.run`"),
        "{after}"
    );
}

/// A non-public import as the last binding hides an earlier re-export.
#[test]
fn a_last_binding_that_is_no_re_export_is_no_member() {
    let mut dataset = two_star_imports("atlas.portable.run", "atlas.portable.run");
    let relations = dataset["relations"].as_array_mut().unwrap();
    relations.last_mut().unwrap()["visibility"] = json!("import");
    let joined = recomputed(&dataset).join("\n");
    assert!(!joined.contains("`atlas.fast.shared`"), "{joined}");
}

/// A property can have parameter entities, and its accessors hold no list to read.
#[test]
fn a_property_with_parameter_entities_has_no_list_to_compare() {
    let mut dataset = load("semantic-oracle-profile-python.example.yml");
    let parameter = json!({"id": "atlas.shapes.Circle.area.self", "kind": "parameter",
        "language-kind": "param", "name": "self", "path": "atlas.shapes.Circle.area.(self)",
        "unit": "shapes-mod", "parent": "atlas.shapes.Circle.area", "origin": "source",
        "definitions": [{"at": {"path": "src/atlas/shapes.py", "start": [19, 14], "end": [19, 18], "text": "self"}}]});
    push(&mut dataset, "/entities", parameter);
    assert_eq!(
        rule_messages(&dataset, "model:signature-parameters"),
        Vec::<String>::new()
    );
    // A list that a property does state, as an indexer's is, is still read.
    let at = record_of(&dataset, "atlas.shapes.Circle.area");
    set_at(
        &mut dataset,
        &format!("/public-api/signatures/{at}/parameters"),
        json!([{"name": "other"}]),
    );
    let found = rule_messages(&dataset, "model:signature-parameters");
    assert!(
        found.iter().any(|m| m.contains("the parameter list is")),
        "{found:#?}"
    );
}

/// The findings of one rule, as `at message`.
fn findings_of(dataset: &Value, rule: &str) -> Vec<String> {
    beholder_semantic_oracle::validate(dataset, None)
        .findings
        .into_iter()
        .filter(|f| f.rule == rule)
        .map(|f| format!("{} {}", f.at, f.message))
        .collect()
}

fn set(dataset: &mut Value, pointer: &str, value: Value) {
    *dataset
        .pointer_mut(pointer)
        .unwrap_or_else(|| panic!("no member {pointer}")) = value;
}

/// The index of the entity with `id`.
fn entity_index(dataset: &Value, id: &str) -> usize {
    dataset["entities"]
        .as_array()
        .unwrap()
        .iter()
        .position(|entity| entity["id"] == id)
        .unwrap_or_else(|| panic!("no entity {id}"))
}

/// The index of the signature record of `id`.
fn signature_index(dataset: &Value, id: &str) -> usize {
    dataset["public-api"]["signatures"]
        .as_array()
        .unwrap()
        .iter()
        .position(|record| record["entity"] == id)
        .unwrap_or_else(|| panic!("no signature {id}"))
}

#[test]
fn a_csharp_unit_root_is_its_assembly() {
    let mut dataset = load("semantic-oracle-profile-csharp.example.yml");
    assert_eq!(
        findings_of(&dataset, "csharp:unit-root"),
        Vec::<String>::new()
    );
    // A namespace of the unit, with the unit's name, is not its assembly.
    set(&mut dataset, "/units/0/root", json!("Geometry"));
    let findings = findings_of(&dataset, "csharp:unit-root");
    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert!(findings[0].starts_with("/units/0/root "), "{findings:#?}");
    assert!(findings[0].contains("`assembly`"), "{findings:#?}");
}

#[test]
fn a_csharp_unit_root_has_the_unit_name() {
    let mut dataset = load("semantic-oracle-profile-csharp.example.yml");
    let root = entity_index(&dataset, "Geometry.assembly");
    set(
        &mut dataset,
        &format!("/entities/{root}/name"),
        json!("Atlas"),
    );
    let findings = findings_of(&dataset, "csharp:unit-root");
    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert!(findings[0].starts_with("/units/0/root "), "{findings:#?}");
    assert!(findings[0].contains("not the unit's name"), "{findings:#?}");
}

#[test]
fn a_csharp_unit_root_is_no_parent() {
    let mut dataset = load("semantic-oracle-profile-csharp.example.yml");
    let child = entity_index(&dataset, "Geometry.IShape");
    set(
        &mut dataset,
        &format!("/entities/{child}/parent"),
        json!("Geometry.assembly"),
    );
    assert_eq!(
        findings_of(&dataset, "csharp:unit-root"),
        vec![format!(
            "/entities/{child}/parent `Geometry.assembly` is a unit root (`assembly`), \
             which is no entity's parent"
        )]
    );
}

#[test]
fn volatile_is_a_csharp_field_modifier_only() {
    let mut dataset = load("semantic-oracle-profile-csharp.example.yml");
    let at = |d: &Value, id: &str| format!("/public-api/signatures/{}", signature_index(d, id));
    let volatile = |d: &Value| -> Vec<String> {
        findings_of(d, "model:signature-slots")
            .into_iter()
            .filter(|f| f.contains("`volatile`"))
            .collect()
    };
    // The example's static volatile field is valid.
    assert_eq!(volatile(&dataset), Vec::<String>::new());
    let method = at(&dataset, "Geometry.Shape.Scale");
    let class = at(&dataset, "Geometry.Shape");
    set(
        &mut dataset,
        &format!("{method}/modifiers"),
        json!(["virtual", "volatile"]),
    );
    set(
        &mut dataset,
        &format!("{class}/modifiers"),
        json!(["abstract", "volatile"]),
    );
    let findings = volatile(&dataset);
    assert_eq!(findings.len(), 2, "{findings:#?}");
    assert!(
        findings
            .iter()
            .any(|f| f.starts_with(&format!("{method}/modifiers/1 "))
                && f.contains("not of a `method`")),
        "{findings:#?}"
    );
    assert!(
        findings
            .iter()
            .any(|f| f.starts_with(&format!("{class}/modifiers/1 "))
                && f.contains("not of a `class`")),
        "{findings:#?}"
    );
}

#[test]
fn a_dart_enum_generative_constructor_is_no_member_and_a_factory_is() {
    let mut dataset = load("semantic-oracle-profile-dart.example.yml");
    for (id, language_kind, name) in [
        ("shapes.Kind.new", "constructor", None),
        ("shapes.Kind.named", "constructor", Some("named")),
        ("shapes.Kind.pick", "factory", Some("pick")),
    ] {
        let path = format!(
            "package:geometry/src/shapes.dart::Kind.{}",
            name.unwrap_or("new")
        );
        let mut record = entity(
            id,
            "callable",
            language_kind,
            &path,
            "shapes.Kind",
            "shapes-lib",
        );
        match name {
            Some(name) => record["name"] = json!(name),
            None => {
                record.as_object_mut().unwrap().remove("name");
            }
        }
        record["declared-visibility"] = json!({"value": "public", "explicit": true});
        push(&mut dataset, "/entities", record);
    }
    let findings = recomputed(&dataset);
    assert!(
        findings
            .iter()
            .all(|f| !f.contains("shapes.Kind.new") && !f.contains("shapes.Kind.named")),
        "{findings:#?}"
    );
    assert!(
        findings.iter().any(|f| f.contains("`shapes.Kind.pick`")),
        "{findings:#?}"
    );
}

#[test]
fn a_dart_enum_value_index_is_its_source_position() {
    let mut dataset = load("semantic-oracle-profile-dart.example.yml");
    assert_eq!(
        findings_of(&dataset, "model:signature-index"),
        Vec::<String>::new()
    );
    let round = signature_index(&dataset, "shapes.Kind.round");
    let square = signature_index(&dataset, "shapes.Kind.square");
    // Reordering the values swaps their indexes; a dataset that keeps the old
    // ones is wrong.
    set(
        &mut dataset,
        &format!("/public-api/signatures/{round}/index"),
        json!(1),
    );
    set(
        &mut dataset,
        &format!("/public-api/signatures/{square}/index"),
        json!(0),
    );
    let findings = findings_of(&dataset, "model:signature-index");
    assert_eq!(findings.len(), 2, "{findings:#?}");
    assert!(
        findings[0].starts_with(&format!("/public-api/signatures/{round}/index "))
            && findings[0].contains("its position is 0"),
        "{findings:#?}"
    );
}

#[test]
fn a_dart_enum_value_always_has_an_index() {
    let mut dataset = load("semantic-oracle-profile-dart.example.yml");
    let round = signature_index(&dataset, "shapes.Kind.round");
    dataset["public-api"]["signatures"]
        .as_array_mut()
        .unwrap()
        .remove(round);
    let findings = findings_of(&dataset, "model:signature-slots");
    assert!(
        findings
            .iter()
            .any(|f| f.contains("`shapes.Kind.round` is a `enum-value`, which has index")),
        "{findings:#?}"
    );
}

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
        let entries = dataset["public-api"].as_array_mut().unwrap();
        let removed = entries.pop().expect("the example states public API");
        let report = beholder_semantic_oracle::validate(&dataset, None);
        let recomputed: Vec<_> = report
            .findings
            .iter()
            .filter(|f| f.rule == "model:public-api-recomputed")
            .collect();
        assert_eq!(recomputed.len(), 1, "{example}: {recomputed:#?}");
        let entity = removed["entity"].as_str().unwrap();
        assert_eq!(recomputed[0].at, "/public-api", "{example}");
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
        .filter(|f| f.rule == "model:public-api-recomputed")
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
    let entries = dataset["public-api"].as_array().unwrap();
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

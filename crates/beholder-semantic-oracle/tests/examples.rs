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
use serde_json::Value;

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

/// Errors the validator reports on the valid examples that wait on
/// contract calibration, as `(example, rule, location)`. The examples have
/// no project files, so only rules that need none apply. The examples are
/// excerpts, so coverage warnings are expected and not checked.
const CALIBRATION: [(&str, &str, &str); 1] = [
    // A Go package's definition sites are its package clauses, so a
    // build-constrained file changes the site set by context; see #17.
    (
        "semantic-oracle-profile-go.example.yml",
        "model:site-membership",
        "/entities/0/definitions/1/at/path",
    ),
];

#[test]
fn valid_examples_pass_validation_and_audit_without_project_files() {
    let mut unexpected = Vec::new();
    let mut seen = BTreeSet::new();
    for example in assets(".example.yml") {
        let dataset = load(&example);
        let report = beholder_semantic_oracle::audit(&dataset, None);
        for finding in report
            .findings
            .iter()
            .filter(|finding| finding.severity == beholder_semantic_oracle::Severity::Error)
        {
            let key = (example.as_str(), finding.rule.as_str(), finding.at.as_str());
            if CALIBRATION
                .iter()
                .any(|known| (known.0, known.1, known.2) == key)
            {
                seen.insert((key.0.to_string(), key.1.to_string(), key.2.to_string()));
            } else {
                unexpected.push(format!(
                    "{example}: {} {} {}: {}",
                    finding.severity, finding.rule, finding.at, finding.message
                ));
            }
        }
    }
    assert!(unexpected.is_empty(), "{}", unexpected.join("\n"));
    assert_eq!(
        seen.len(),
        CALIBRATION.len(),
        "a calibration finding no longer occurs"
    );
}

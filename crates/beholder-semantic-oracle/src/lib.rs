// ---
// relationships:
//   implements: semantic-oracle-model
// ---
//! Semantic-oracle dataset schemas.
//!
//! The schemas are the JSON Schema assets of the `semantic-oracle-model` and
//! `semantic-oracle-profile-<language>` specifications in
//! `docs/specifications`. This crate embeds them, states the versions they
//! implement, and builds Draft 2020-12 validators from them.
//!
//! Schema validation is structural. Id resolution, id uniqueness, context
//! partitions, locator checks and the profile audit are not schema checks.

use anyhow::{anyhow, Context, Result};
use jsonschema::{Draft, Registry, Validator};
use serde_json::Value;

/// The model version a dataset names in `model`.
pub const MODEL_VERSION: &str = "semantic-oracle/1";

/// The `$id` of the model schema. The path carries the model version.
pub const MODEL_SCHEMA_ID: &str = "https://wyrd.foo/beholder/schemas/semantic-oracle/1";

const MODEL_SCHEMA: &str =
    include_str!("../../../docs/specifications/semantic-oracle-model.json-schema.yml");

/// One language profile of the model version this crate implements.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Profile {
    /// The profile language, as a dataset names it in `project.language`.
    pub language: &'static str,
    /// The profile version, as a dataset names it in `profile`.
    pub version: &'static str,
    /// The `$id` of the profile schema: the model schema `$id` followed by
    /// the profile version.
    pub schema_id: &'static str,
    /// The base name of the profile specification in `docs/specifications`.
    pub specification: &'static str,
    schema: &'static str,
}

macro_rules! profile {
    ($language:literal) => {
        Profile {
            language: $language,
            version: concat!($language, "/1"),
            schema_id: concat!(
                "https://wyrd.foo/beholder/schemas/semantic-oracle/1/",
                $language,
                "/1"
            ),
            specification: concat!("semantic-oracle-profile-", $language),
            schema: include_str!(concat!(
                "../../../docs/specifications/semantic-oracle-profile-",
                $language,
                ".json-schema.yml"
            )),
        }
    };
}

/// Every profile of model version 1.
pub const PROFILES: [Profile; 6] = [
    profile!("csharp"),
    profile!("dart"),
    profile!("go"),
    profile!("python"),
    profile!("rust"),
    profile!("typescript"),
];

/// Parses one YAML 1.2 document into a JSON value.
pub fn parse_yaml(text: &str) -> Result<Value> {
    serde_saphyr::from_str(text).map_err(|error| anyhow!("{error}"))
}

/// The model schema as a JSON value.
pub fn model_schema() -> Result<Value> {
    parse_yaml(MODEL_SCHEMA).context("parse the model schema")
}

impl Profile {
    /// The profile schema as a JSON value.
    pub fn schema(&self) -> Result<Value> {
        parse_yaml(self.schema).with_context(|| format!("parse the {} schema", self.version))
    }

    /// Finds a profile by the version a dataset names.
    pub fn by_version(version: &str) -> Option<Profile> {
        PROFILES
            .into_iter()
            .find(|profile| profile.version == version)
    }
}

/// A validator for any model version 1 dataset. It applies the model
/// structure and the schema of the profile the dataset names, and rejects a
/// profile it does not define.
pub fn dataset_validator() -> Result<Validator> {
    validator_for(MODEL_SCHEMA_ID)
}

/// A validator that accepts only datasets of one profile.
pub fn profile_validator(profile: &Profile) -> Result<Validator> {
    validator_for(profile.schema_id)
}

fn validator_for(schema_id: &str) -> Result<Validator> {
    let mut registry = Registry::new().add(MODEL_SCHEMA_ID, model_schema()?)?;
    for profile in PROFILES {
        registry = registry.add(profile.schema_id, profile.schema()?)?;
    }
    let registry = registry.prepare()?;
    jsonschema::options()
        .with_draft(Draft::Draft202012)
        .with_registry(&registry)
        .offline()
        .build(&serde_json::json!({ "$ref": schema_id }))
        .with_context(|| format!("build the validator for {schema_id}"))
}

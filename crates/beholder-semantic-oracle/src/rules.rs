//! Profile rules the validator and the audit apply, as data. Each table
//! restates one section of a `semantic-oracle-profile-<language>`
//! specification; the specification governs.

/// What an id-valued member of an extension fact names.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Names {
    /// A project entity.
    Entity,
    /// A project entity or an external entity.
    EntityOrExternal,
    /// A unit.
    Unit,
    /// A unit by id, or an external assembly by name.
    UnitOrName,
    /// A distribution.
    Distribution,
    /// A relation with an `id`.
    Relation,
    /// A reference with an `id`.
    Reference,
    /// A locator.
    Locator,
    /// A list of canonical document paths.
    Documents,
    /// A sorted list of strings.
    SortedList,
}

/// One extension kind: its members that name records, and its key.
#[derive(Debug, Clone, Copy)]
pub struct Extension {
    pub kind: &'static str,
    pub members: &'static [(&'static str, Names)],
    pub key: &'static [&'static str],
}

/// The profile's public API rule, which the validator recomputes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PublicApiRule {
    CSharp,
    Dart,
    Go,
    Python,
    Rust,
    TypeScript,
}

/// One profile's rules.
#[derive(Debug, Clone, Copy)]
pub struct ProfileRules {
    pub language: &'static str,
    /// The unit members, in order, that form the unit key. `kind` and
    /// `name` are the model's members; the rest are profile attributes.
    pub unit_key: &'static [&'static str],
    /// Context parameters whose list value the profile states is sorted.
    pub sorted_parameters: &'static [&'static str],
    /// Unit attributes whose list value the profile states is sorted.
    pub sorted_unit_attributes: &'static [&'static str],
    pub extensions: &'static [Extension],
    /// File name suffixes of program source in the profile language.
    pub source_suffixes: &'static [&'static str],
    /// Source suffixes whose files the profile may also classify as
    /// another language's files (TypeScript `.js` without `allowJs`).
    pub optional_source_suffixes: &'static [&'static str],
    /// Non-source suffixes whose files the profile may index.
    pub indexable_suffixes: &'static [&'static str],
    /// File names of source files the profile allows as configuration.
    pub source_configuration: &'static [&'static str],
    /// Whether the profile defines an augmentation form, through which a
    /// site in one unit's document belongs to a fact of another unit.
    pub augmentation: bool,
    /// File name patterns the profile classifies as configuration. `*`
    /// matches any run of characters within the file name.
    pub configuration: &'static [&'static str],
    /// The public API membership rule.
    pub public_api: PublicApiRule,
}

use Names::*;

const RUST: ProfileRules = ProfileRules {
    language: "rust",
    unit_key: &[
        "package",
        "package-version",
        "kind",
        "target-name",
        "platform",
    ],
    sorted_parameters: &["selection", "cfg"],
    sorted_unit_attributes: &["features"],
    extensions: &[
        Extension {
            kind: "rust:reachable-unnameable",
            members: &[
                ("entity", Entity),
                ("distribution", Distribution),
                ("via", Entity),
            ],
            key: &["entity", "distribution"],
        },
        Extension {
            kind: "rust:macro-expansion",
            members: &[
                ("entity", Entity),
                ("macro", EntityOrExternal),
                ("invocation", Locator),
            ],
            key: &["entity"],
        },
    ],
    source_suffixes: &[".rs"],
    optional_source_suffixes: &[],
    indexable_suffixes: &[],
    source_configuration: &[],
    augmentation: false,
    configuration: &["Cargo.toml", "rust-toolchain.toml", ".cargo/config.toml"],
    public_api: PublicApiRule::Rust,
};

const CSHARP: ProfileRules = ProfileRules {
    language: "csharp",
    unit_key: &["name", "assembly-version", "project"],
    sorted_parameters: &["properties"],
    sorted_unit_attributes: &[],
    extensions: &[
        Extension {
            kind: "csharp:type-forward",
            members: &[("unit", Unit), ("type", EntityOrExternal), ("at", Locator)],
            key: &["unit", "type"],
        },
        Extension {
            kind: "csharp:extern-alias",
            members: &[("at", Locator), ("unit", UnitOrName)],
            key: &["at"],
        },
        Extension {
            kind: "csharp:reference-alias",
            members: &[
                ("unit", Unit),
                ("reference", UnitOrName),
                ("aliases", SortedList),
            ],
            key: &["unit", "reference"],
        },
    ],
    source_suffixes: &[".cs"],
    optional_source_suffixes: &[],
    indexable_suffixes: &[],
    source_configuration: &[],
    augmentation: false,
    configuration: &[
        "*.csproj",
        "*.sln",
        "*.slnx",
        "Directory.Build.props",
        "Directory.Build.targets",
        "global.json",
        ".editorconfig",
        "NuGet.config",
    ],
    public_api: PublicApiRule::CSharp,
};

const DART: ProfileRules = ProfileRules {
    language: "dart",
    unit_key: &["name"],
    sorted_parameters: &["defines", "library-flags", "options"],
    sorted_unit_attributes: &[],
    extensions: &[
        Extension {
            kind: "dart:visibility-annotation",
            members: &[("entity", Entity)],
            key: &["entity", "annotation"],
        },
        Extension {
            kind: "dart:mixin-application",
            members: &[("relation", Relation)],
            key: &["relation"],
        },
        Extension {
            kind: "dart:deferred-import",
            members: &[("unit", Unit), ("library", Entity)],
            key: &["unit", "prefix"],
        },
    ],
    source_suffixes: &[".dart"],
    optional_source_suffixes: &[],
    indexable_suffixes: &[],
    source_configuration: &[],
    augmentation: false,
    configuration: &[
        "pubspec.yaml",
        "analysis_options.yaml",
        "build.yaml",
        "pubspec_overrides.yaml",
    ],
    public_api: PublicApiRule::Dart,
};

const GO: ProfileRules = ProfileRules {
    language: "go",
    unit_key: &["name", "kind"],
    sorted_parameters: &["tags"],
    sorted_unit_attributes: &[],
    extensions: &[
        Extension {
            kind: "go:embeds",
            members: &[
                ("struct", Entity),
                ("field", Entity),
                ("type", EntityOrExternal),
            ],
            key: &["struct", "field"],
        },
        Extension {
            kind: "go:pointer-receiver",
            members: &[("type", Entity), ("interface", EntityOrExternal)],
            key: &["type", "interface"],
        },
        Extension {
            kind: "go:linkname",
            members: &[
                ("entity", Entity),
                ("target", EntityOrExternal),
                ("at", Locator),
            ],
            key: &["entity"],
        },
        Extension {
            kind: "go:embed-files",
            members: &[("variable", Entity), ("documents", Documents)],
            key: &["variable"],
        },
    ],
    source_suffixes: &[".go"],
    optional_source_suffixes: &[],
    indexable_suffixes: &[],
    source_configuration: &[],
    augmentation: false,
    configuration: &["go.mod", "go.work", "go.sum"],
    public_api: PublicApiRule::Go,
};

const PYTHON: ProfileRules = ProfileRules {
    language: "python",
    unit_key: &["name", "kind"],
    sorted_parameters: &["options"],
    sorted_unit_attributes: &[],
    extensions: &[
        Extension {
            kind: "python:stub-of",
            members: &[("stub", Entity), ("runtime", Entity)],
            key: &["stub"],
        },
        Extension {
            kind: "python:all-member",
            members: &[("module", Entity)],
            key: &["module", "name"],
        },
        Extension {
            kind: "python:module-getattr",
            members: &[("module", Entity), ("target", EntityOrExternal)],
            key: &["module", "name"],
        },
    ],
    source_suffixes: &[".py", ".pyi"],
    optional_source_suffixes: &[],
    indexable_suffixes: &[],
    source_configuration: &["setup.py"],
    augmentation: false,
    configuration: &["pyproject.toml", "setup.cfg", "*.pth", "py.typed"],
    public_api: PublicApiRule::Python,
};

const TYPESCRIPT: ProfileRules = ProfileRules {
    language: "typescript",
    unit_key: &["name"],
    sorted_parameters: &[],
    sorted_unit_attributes: &[],
    extensions: &[
        Extension {
            kind: "typescript:module-format",
            members: &[("unit", Unit)],
            key: &["unit"],
        },
        Extension {
            kind: "typescript:type-only",
            members: &[("reference", Reference)],
            key: &["reference"],
        },
        Extension {
            kind: "typescript:entry",
            members: &[("distribution", Distribution), ("module", Entity)],
            key: &["distribution", "mode", "subpath"],
        },
        Extension {
            kind: "typescript:augments",
            members: &[
                ("entity", Entity),
                ("target", EntityOrExternal),
                ("at", Locator),
            ],
            key: &["entity"],
        },
    ],
    source_suffixes: &[".ts", ".tsx", ".mts", ".cts"],
    optional_source_suffixes: &[".js", ".jsx", ".mjs", ".cjs"],
    indexable_suffixes: &[".json"],
    source_configuration: &[],
    augmentation: true,
    configuration: &["package.json", "tsconfig*.json", "jsconfig*.json"],
    public_api: PublicApiRule::TypeScript,
};

/// The rules of every profile of model version 1.
pub const RULES: [ProfileRules; 6] = [CSHARP, DART, GO, PYTHON, RUST, TYPESCRIPT];

impl ProfileRules {
    pub fn for_language(language: &str) -> Option<&'static ProfileRules> {
        RULES.iter().find(|rules| rules.language == language)
    }

    pub fn extension(&self, kind: &str) -> Option<&'static Extension> {
        self.extensions
            .iter()
            .find(|extension| extension.kind == kind)
    }

    /// Whether a document path is program source of the profile language.
    pub fn is_source(&self, path: &str) -> bool {
        self.source_suffixes
            .iter()
            .any(|suffix| path.ends_with(suffix))
    }

    pub fn is_optional_source(&self, path: &str) -> bool {
        self.optional_source_suffixes
            .iter()
            .any(|suffix| path.ends_with(suffix))
    }

    pub fn is_indexable(&self, path: &str) -> bool {
        self.is_source(path)
            || self.is_optional_source(path)
            || self
                .indexable_suffixes
                .iter()
                .any(|suffix| path.ends_with(suffix))
    }

    /// Whether the profile allows a source file as configuration.
    pub fn is_source_configuration(&self, path: &str) -> bool {
        let name = path.rsplit('/').next().unwrap_or(path);
        self.source_configuration.contains(&name)
    }

    /// Whether a document path matches a configuration file class.
    pub fn is_configuration(&self, path: &str) -> bool {
        let name = path.rsplit('/').next().unwrap_or(path);
        self.configuration.iter().any(|pattern| {
            if pattern.contains('/') {
                path == *pattern || path.ends_with(&format!("/{pattern}"))
            } else {
                glob(pattern, name)
            }
        })
    }
}

/// Matches `name` against `pattern`, where `*` matches any run of
/// characters.
fn glob(pattern: &str, name: &str) -> bool {
    match pattern.split_once('*') {
        None => pattern == name,
        Some((head, tail)) => {
            name.len() >= head.len() + tail.len() && name.starts_with(head) && name.ends_with(tail)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::PROFILES;

    #[test]
    fn every_profile_has_rules() {
        for profile in PROFILES {
            assert!(
                ProfileRules::for_language(profile.language).is_some(),
                "{}",
                profile.language
            );
        }
    }

    #[test]
    fn rules_name_every_extension_kind_of_the_profile_schema() {
        for profile in PROFILES {
            let rules = ProfileRules::for_language(profile.language).unwrap();
            let schema = profile.schema().unwrap();
            let kinds: Vec<&str> = schema["$defs"]["extensions"]["properties"]
                .as_object()
                .unwrap()
                .keys()
                .map(String::as_str)
                .collect();
            let mut named: Vec<&str> = rules.extensions.iter().map(|e| e.kind).collect();
            named.sort();
            let mut kinds = kinds;
            kinds.sort();
            assert_eq!(named, kinds, "{}", profile.language);
            for extension in rules.extensions {
                let members = &schema["$defs"]["extensions"]["properties"][extension.kind]["items"]
                    ["properties"];
                for (member, _) in extension.members.iter() {
                    assert!(members.get(member).is_some(), "{} {member}", extension.kind);
                }
                for member in extension.key {
                    assert!(members.get(member).is_some(), "{} {member}", extension.kind);
                }
            }
        }
    }

    #[test]
    fn configuration_patterns() {
        let csharp = ProfileRules::for_language("csharp").unwrap();
        assert!(csharp.is_configuration("src/App/App.csproj"));
        assert!(!csharp.is_configuration("src/App/App.cs"));
        let typescript = ProfileRules::for_language("typescript").unwrap();
        assert!(typescript.is_configuration("tsconfig.build.json"));
        assert!(typescript.is_configuration("packages/a/package.json"));
        let rust = ProfileRules::for_language("rust").unwrap();
        assert!(rust.is_configuration(".cargo/config.toml"));
        assert!(rust.is_configuration("crates/a/.cargo/config.toml"));
        assert!(!rust.is_configuration("config.toml"));
    }
}

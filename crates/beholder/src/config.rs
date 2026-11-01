//! Analysis configuration.
//!
//! Configuration is part of the analysis fingerprint: a stored phase result is
//! only reusable when it was produced under the same configuration. See
//! [`Config::fingerprint`].

use std::path::Path;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

/// File name looked up at the repository root.
pub const CONFIG_FILE: &str = "beholder.toml";

/// Configured ignores, layered on top of `.gitignore`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct Config {
    /// File extensions to skip, without the leading dot.
    pub ignored_extensions: Vec<String>,
    /// Substrings of repo-relative paths to skip.
    pub ignored_paths: Vec<String>,
    /// When non-empty, only repo-relative paths containing one of these
    /// substrings are analyzed.
    pub only_paths: Vec<String>,
    /// Additional repository-relative exact files or directory prefixes.
    #[serde(skip_deserializing, skip_serializing_if = "Vec::is_empty")]
    pub selected_paths: Vec<String>,
}

impl Config {
    /// Apply an additional selection without widening configured exclusions.
    pub fn select_paths(&mut self, paths: &[String]) -> Result<()> {
        let mut normalized = Vec::new();
        for path in paths {
            anyhow::ensure!(
                !path.is_empty()
                    && !path.starts_with('/')
                    && !path.contains('\\')
                    && !path.contains(':'),
                "selector must be a non-empty repository-relative path"
            );
            let mut parts = Vec::new();
            for part in path.split('/') {
                match part {
                    "" | "." => {}
                    ".." => anyhow::bail!("selector must not contain .."),
                    value => parts.push(value),
                }
            }
            anyhow::ensure!(!parts.is_empty(), "selector must name a file or directory");
            normalized.push(parts.join("/"));
        }
        normalized.sort();
        normalized.dedup();
        self.selected_paths = normalized;
        Ok(())
    }

    /// Parse configuration from TOML text.
    pub fn from_toml(body: &str) -> Result<Self> {
        let mut config: Config = toml::from_str(body).context("parsing beholder configuration")?;
        config.normalize();
        Ok(config)
    }

    /// Load `beholder.toml` from a repository root, or the default when absent.
    pub fn load(repo_root: &Path) -> Result<Self> {
        let path = repo_root.join(CONFIG_FILE);
        match std::fs::read_to_string(&path) {
            Ok(body) => Self::from_toml(&body),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(Self::default()),
            Err(err) => Err(err).with_context(|| format!("reading {}", path.display())),
        }
    }

    /// Sort and dedupe so the fingerprint does not depend on authoring order.
    fn normalize(&mut self) {
        for list in [
            &mut self.ignored_extensions,
            &mut self.ignored_paths,
            &mut self.only_paths,
        ] {
            list.sort();
            list.dedup();
        }
    }

    /// Should this repo-relative path be analyzed?
    pub fn accepts(&self, path: &str) -> bool {
        self.extension_allowed(path)
            && self.path_allowed(path)
            && self.only_path_allowed(path)
            && (self.selected_paths.is_empty()
                || self.selected_paths.iter().any(|p| {
                    path == p
                        || path
                            .strip_prefix(p)
                            .is_some_and(|tail| tail.starts_with('/'))
                }))
    }

    fn extension_allowed(&self, path: &str) -> bool {
        match Path::new(path).extension().and_then(|e| e.to_str()) {
            Some(ext) => !self.ignored_extensions.iter().any(|i| i == ext),
            None => true,
        }
    }

    fn path_allowed(&self, path: &str) -> bool {
        !self.ignored_paths.iter().any(|i| path.contains(i.as_str()))
    }

    fn only_path_allowed(&self, path: &str) -> bool {
        self.only_paths.is_empty() || self.only_paths.iter().any(|p| path.contains(p.as_str()))
    }

    /// Stable identity of this configuration, used to validate stored results.
    ///
    /// Derived from the normalized values only. It carries no absolute path, no
    /// timestamp, and no machine-local state.
    pub fn fingerprint(&self) -> String {
        let canonical =
            serde_json::to_string(self).expect("configuration is always serializable to JSON");
        crate::hash::hex_sha256(canonical.as_bytes())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selection_is_canonical_and_cannot_be_loaded_from_toml() {
        assert!(Config::from_toml("selected_paths = [\"src\"]").is_err());
        let mut a = Config::default();
        a.select_paths(&["./src//lib/".into(), "tests".into(), "tests".into()])
            .unwrap();
        let mut b = Config::default();
        b.select_paths(&["tests".into(), "src/lib".into()]).unwrap();
        assert_eq!(a.fingerprint(), b.fingerprint());
        assert_ne!(a.fingerprint(), Config::default().fingerprint());
        b.select_paths(&["src/library".into()]).unwrap();
        assert_ne!(a.fingerprint(), b.fingerprint());
    }

    #[test]
    fn missing_configuration_is_the_default() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(Config::load(dir.path()).unwrap(), Config::default());
    }

    #[test]
    fn ignores_extensions_and_paths() {
        let config = Config::from_toml(
            r#"
            ignored_extensions = ["lock", "svg"]
            ignored_paths = ["vendor/"]
        "#,
        )
        .unwrap();

        assert!(!config.accepts("Cargo.lock"));
        assert!(!config.accepts("assets/logo.svg"));
        assert!(!config.accepts("vendor/thing.rs"));
        assert!(config.accepts("src/main.rs"));
    }

    #[test]
    fn only_paths_restricts_the_walk() {
        let config = Config::from_toml(r#"only_paths = ["crates/"]"#).unwrap();

        assert!(config.accepts("crates/beholder/src/lib.rs"));
        assert!(!config.accepts("xtask/src/main.rs"));
    }

    #[test]
    fn fingerprint_ignores_authoring_order() {
        let a = Config::from_toml(r#"ignored_extensions = ["svg", "lock"]"#).unwrap();
        let b = Config::from_toml(r#"ignored_extensions = ["lock", "svg"]"#).unwrap();

        assert_eq!(a.fingerprint(), b.fingerprint());
    }

    #[test]
    fn fingerprint_changes_with_configuration() {
        let a = Config::default();
        let b = Config::from_toml(r#"ignored_paths = ["vendor/"]"#).unwrap();

        assert_ne!(a.fingerprint(), b.fingerprint());
    }
}

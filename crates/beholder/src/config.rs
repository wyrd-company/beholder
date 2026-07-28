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
}

impl Config {
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
        self.extension_allowed(path) && self.path_allowed(path) && self.only_path_allowed(path)
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

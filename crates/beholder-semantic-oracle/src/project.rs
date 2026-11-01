//! The files of one fixture project: the tracked file list and file bytes
//! that the document inventory and locators are checked against.

use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

use anyhow::{anyhow, Context, Result};
use sha2::{Digest, Sha256};

/// The file name of a dataset at its project root.
pub const DATASET_FILE: &str = "semantic-oracle.yml";

/// The tracked files of a project, by canonical path relative to the
/// project root, and their bytes.
pub trait ProjectFiles {
    /// Every tracked file under the project root except the dataset file,
    /// sorted.
    fn tracked(&self) -> Result<Vec<String>>;

    /// The bytes of a file, or `None` when it does not exist.
    fn read(&self, path: &str) -> Result<Option<Vec<u8>>>;
}

/// The lowercase hexadecimal SHA-256 digest of `bytes`.
pub fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

/// A project in a git working tree. Tracked files are the entries of the
/// repository index under the project root; bytes come from the working
/// tree.
pub struct GitProject {
    root: PathBuf,
    tracked: Vec<String>,
}

impl GitProject {
    /// Opens the project whose root is `root`, inside a git working tree.
    pub fn open(root: &Path) -> Result<Self> {
        let root = root
            .canonicalize()
            .with_context(|| format!("open project {}", root.display()))?;
        let repo = gix::discover(&root)
            .with_context(|| format!("{} is not in a git working tree", root.display()))?;
        let workdir = repo
            .workdir()
            .ok_or_else(|| anyhow!("the repository of {} has no working tree", root.display()))?
            .canonicalize()?;
        let prefix = root
            .strip_prefix(&workdir)
            .with_context(|| format!("{} is outside its working tree", root.display()))?;
        // Index paths are `/`-separated bytes; join the components so that
        // a literal backslash in a Unix directory name is kept.
        let mut prefix_bytes = Vec::new();
        for component in prefix.components() {
            let name = component
                .as_os_str()
                .to_str()
                .ok_or_else(|| anyhow!("the project path is not UTF-8"))?;
            prefix_bytes.extend_from_slice(name.as_bytes());
            prefix_bytes.push(b'/');
        }

        let index = repo.index_or_empty()?;
        let mut tracked = Vec::new();
        for entry in index.entries() {
            // A submodule is a commit, not a file.
            if entry.mode == gix::index::entry::Mode::COMMIT {
                continue;
            }
            let path: &[u8] = entry.path(&index).as_ref();
            // Paths outside the project are not its business, whatever their
            // encoding.
            let Some(relative) = path.strip_prefix(prefix_bytes.as_slice()) else {
                continue;
            };
            let relative = std::str::from_utf8(relative).map_err(|_| {
                anyhow!(
                    "a tracked path is not UTF-8: {}",
                    String::from_utf8_lossy(relative)
                )
            })?;
            if relative != DATASET_FILE {
                tracked.push(relative.to_owned());
            }
        }
        tracked.sort();
        tracked.dedup();
        Ok(Self { root, tracked })
    }

    pub fn root(&self) -> &Path {
        &self.root
    }
}

impl ProjectFiles for GitProject {
    fn tracked(&self) -> Result<Vec<String>> {
        Ok(self.tracked.clone())
    }

    fn read(&self, path: &str) -> Result<Option<Vec<u8>>> {
        let full = self.root.join(path);
        let metadata = match full.symlink_metadata() {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(error).with_context(|| format!("read {path}")),
        };
        // Git stores a symbolic link as its target path.
        if metadata.file_type().is_symlink() {
            let target = std::fs::read_link(&full)?;
            return Ok(Some(link_bytes(&target, path)?));
        }
        if !metadata.is_file() {
            return Ok(None);
        }
        std::fs::read(&full)
            .map(Some)
            .with_context(|| format!("read {path}"))
    }
}

/// The bytes git stores for a symbolic link: its target path.
#[cfg(unix)]
fn link_bytes(target: &Path, _path: &str) -> Result<Vec<u8>> {
    use std::os::unix::ffi::OsStrExt;
    Ok(target.as_os_str().as_bytes().to_vec())
}

#[cfg(not(unix))]
fn link_bytes(target: &Path, path: &str) -> Result<Vec<u8>> {
    let target = target
        .to_str()
        .ok_or_else(|| anyhow!("the link target of {path} is not UTF-8"))?;
    Ok(target.replace('\\', "/").into_bytes())
}

/// A project held in memory, for tests and for checks that need no
/// working tree. Every file it holds is tracked.
#[derive(Debug, Clone, Default)]
pub struct MemoryProject {
    files: BTreeMap<String, Vec<u8>>,
}

impl MemoryProject {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with(mut self, path: &str, contents: impl Into<Vec<u8>>) -> Self {
        self.files.insert(path.to_owned(), contents.into());
        self
    }
}

impl ProjectFiles for MemoryProject {
    fn tracked(&self) -> Result<Vec<String>> {
        Ok(self
            .files
            .keys()
            .filter(|path| path.as_str() != DATASET_FILE)
            .cloned()
            .collect())
    }

    fn read(&self, path: &str) -> Result<Option<Vec<u8>>> {
        Ok(self.files.get(path).cloned())
    }
}

/// The `documents` block of a dataset for the project's tracked files, as
/// YAML text. Each entry has the file's path and current digest. An entry
/// the dataset already has keeps its disposition, reason, origin and
/// members; a new entry has none, so validation fails until an author
/// classifies it.
pub fn inventory(files: &dyn ProjectFiles, dataset: Option<&serde_json::Value>) -> Result<String> {
    use crate::locator::yaml_scalar;
    use serde_json::Value;

    let existing: BTreeMap<&str, &Value> = dataset
        .and_then(|dataset| dataset.get("documents"))
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|document| Some((document.get("path")?.as_str()?, document)))
        .collect();
    let tracked = files.tracked()?;
    if tracked.is_empty() {
        return Ok("documents: []\n".into());
    }
    let mut out = String::from("documents:\n");
    for path in tracked {
        let bytes = files
            .read(&path)?
            .ok_or_else(|| anyhow!("tracked file `{path}` is missing from the working tree"))?;
        out.push_str(&format!("  - path: {}\n", yaml_scalar(&path)));
        out.push_str(&format!("    sha256: {}\n", sha256_hex(&bytes)));
        let Some(document) = existing.get(path.as_str()) else {
            out.push_str("    # new: needs a disposition\n");
            continue;
        };
        for member in ["disposition", "reason", "origin"] {
            if let Some(value) = document.get(member).and_then(Value::as_str) {
                out.push_str(&format!("    {member}: {}\n", yaml_scalar(value)));
            }
        }
        if let Some(members) = document.get("members").and_then(Value::as_array) {
            out.push_str("    members:\n");
            for member in members {
                let unit = member
                    .get("unit")
                    .and_then(Value::as_str)
                    .unwrap_or_default();
                out.push_str(&format!("      - unit: {}\n", yaml_scalar(unit)));
                if let Some(contexts) = member.get("contexts").and_then(Value::as_array) {
                    let contexts: Vec<String> = contexts
                        .iter()
                        .filter_map(Value::as_str)
                        .map(yaml_scalar)
                        .collect();
                    out.push_str(&format!("        contexts: [{}]\n", contexts.join(", ")));
                }
            }
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inventory_keeps_classifications_and_refreshes_digests() {
        let files = MemoryProject::new()
            .with("src/a.rs", "fn a() {}\n")
            .with("README.md", "# readme\n")
            .with(DATASET_FILE, "model: semantic-oracle/1\n");
        let dataset = serde_json::json!({"documents": [
            {"path": "src/a.rs", "sha256": "0", "disposition": "indexed", "origin": "source",
             "members": [{"unit": "lib", "contexts": ["default"]}]}
        ]});
        let text = inventory(&files, Some(&dataset)).unwrap();
        let expected = format!(
            "documents:\n  - path: README.md\n    sha256: {}\n    # new: needs a disposition\n  - path: src/a.rs\n    sha256: {}\n    disposition: indexed\n    origin: source\n    members:\n      - unit: lib\n        contexts: [default]\n",
            sha256_hex(b"# readme\n"),
            sha256_hex(b"fn a() {}\n"),
        );
        assert_eq!(text, expected);
        let parsed = crate::parse_yaml(&text).unwrap();
        assert_eq!(
            parsed["documents"][1]["members"][0]["contexts"][0],
            "default"
        );
    }

    #[test]
    fn an_empty_inventory_is_an_empty_list() {
        let text = inventory(&MemoryProject::new(), None).unwrap();
        assert_eq!(
            crate::parse_yaml(&text).unwrap()["documents"],
            serde_json::json!([])
        );
    }

    #[test]
    fn digest_is_lowercase_hex_sha256() {
        assert_eq!(
            sha256_hex(b""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }
}

// ---
// relationships:
//   references: semantic-oracle-model
// ---
//! The `semantic-oracle` binary against a git repository built with `git`.
//!
//! The fixture project is copied below the repository root, so the tests
//! also cover a project root that is not the working tree root.

use std::{
    path::{Path, PathBuf},
    process::{Command, Output},
};

use serde_json::Value;

const FIXTURE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../beholder-semantic-oracle/tests/fixtures/tally"
);
const FILES: [&str; 4] = [
    "Cargo.toml",
    "README.md",
    "src/lib.rs",
    "semantic-oracle.yml",
];

struct Repo {
    dir: tempfile::TempDir,
}

impl Repo {
    /// A repository with the fixture project at `projects/tally`, tracked.
    fn new() -> Self {
        let repo = Self {
            dir: tempfile::tempdir().unwrap(),
        };
        repo.git(&["init", "-q"]);
        for file in FILES {
            let to = repo.project().join(file);
            std::fs::create_dir_all(to.parent().unwrap()).unwrap();
            std::fs::copy(Path::new(FIXTURE).join(file), &to).unwrap();
        }
        repo.git(&["add", "."]);
        repo
    }

    fn project(&self) -> PathBuf {
        self.dir.path().join("projects/tally")
    }

    fn git(&self, args: &[&str]) {
        let status = Command::new("git")
            .args(args)
            .current_dir(self.dir.path())
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .env("GIT_CONFIG_SYSTEM", "/dev/null")
            .status()
            .expect("run git");
        assert!(status.success(), "git {args:?}");
    }

    fn write(&self, path: &str, contents: &str) {
        std::fs::write(self.project().join(path), contents).unwrap();
    }

    fn run(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_semantic-oracle"))
            .args(args)
            .current_dir(self.dir.path())
            .output()
            .expect("run semantic-oracle")
    }
}

fn stdout(output: &Output) -> String {
    String::from_utf8(output.stdout.clone()).unwrap()
}

fn lines(output: &Output) -> Vec<Value> {
    stdout(output)
        .lines()
        .map(|line| serde_json::from_str(line).expect("a JSON line"))
        .collect()
}

#[test]
fn a_valid_dataset_passes_with_one_summary_line() {
    let repo = Repo::new();
    for command in ["validate", "audit"] {
        let output = repo.run(&[command, "projects/tally"]);
        assert_eq!(output.status.code(), Some(0), "{}", stdout(&output));
        assert_eq!(
            stdout(&output),
            "{\"summary\":{\"errors\":0,\"warnings\":0}}\n"
        );
    }
    let output = repo.run(&["validate", "projects/tally/semantic-oracle.yml"]);
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn a_shifted_source_file_fails_with_stale_locators() {
    let repo = Repo::new();
    repo.write(
        "src/lib.rs",
        "\npub struct Counter;\n\npub fn make() -> Counter {\n    Counter\n}\n",
    );
    let output = repo.run(&["validate", "projects/tally"]);
    assert_eq!(output.status.code(), Some(1));
    let lines = lines(&output);
    assert_eq!(lines.len(), 2, "{}", stdout(&output));
    assert_eq!(lines[0]["rule"], "model:locator-stale");
    assert_eq!(lines[0]["at"], "/documents/2/sha256");
    assert_eq!(lines[0]["severity"], "error");
    assert_eq!(lines[1]["summary"]["errors"], 1);
}

#[test]
fn only_tracked_files_need_inventory_entries() {
    let repo = Repo::new();
    repo.write("notes.txt", "scratch\n");
    let output = repo.run(&["validate", "projects/tally"]);
    assert_eq!(output.status.code(), Some(0), "{}", stdout(&output));

    repo.git(&["add", "projects/tally/notes.txt"]);
    let output = repo.run(&["validate", "projects/tally"]);
    assert_eq!(output.status.code(), Some(1));
    let lines = lines(&output);
    assert_eq!(lines[0]["rule"], "model:document-inventory");
    assert_eq!(
        lines[0]["message"],
        "tracked file `notes.txt` has no inventory entry"
    );
}

#[test]
fn locate_prints_a_locator_or_lists_the_occurrences() {
    let repo = Repo::new();
    let project = repo.project();
    let project = project.to_str().unwrap();
    let output = repo.run(&[
        "locate",
        "src/lib.rs",
        "Counter",
        "--line",
        "3",
        "--project",
        project,
    ]);
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(
        stdout(&output),
        "{path: src/lib.rs, start: [3, 18], end: [3, 25], text: Counter}\n"
    );

    let output = repo.run(&["locate", "src/lib.rs", "Counter", "--project", project]);
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(
        String::from_utf8(output.stderr).unwrap(),
        "semantic-oracle: src/lib.rs: the text occurs at [1, 12], [3, 18], [4, 5]\n"
    );
}

#[test]
fn inventory_lists_tracked_files_with_their_classification() {
    let repo = Repo::new();
    repo.write("CHANGES.md", "none\n");
    repo.git(&["add", "projects/tally/CHANGES.md"]);
    let output = repo.run(&["inventory", "projects/tally"]);
    assert_eq!(output.status.code(), Some(0));
    let text = stdout(&output);
    let paths: Vec<&str> = text
        .lines()
        .filter_map(|line| line.strip_prefix("  - path: "))
        .collect();
    assert_eq!(
        paths,
        ["CHANGES.md", "Cargo.toml", "README.md", "src/lib.rs"]
    );
    assert!(text.contains("  - path: CHANGES.md\n    sha256: "));
    assert!(text.contains("    # new: needs a disposition\n"));
    assert!(text.contains("    disposition: out-of-profile\n    reason: documentation\n"));
}

#[test]
fn unreadable_input_exits_2_and_bad_yaml_is_a_finding() {
    let repo = Repo::new();
    let output = repo.run(&["validate", "projects/none"]);
    assert_eq!(output.status.code(), Some(2));

    repo.write("semantic-oracle.yml", "model: [unclosed\n");
    let output = repo.run(&["validate", "projects/tally"]);
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(lines(&output)[0]["rule"], "model:yaml");
}

/// Paths git stores as bytes: a backslash in a directory name, a sibling
/// file name that is not UTF-8, and a link target that is not UTF-8.
// macOS and Windows file systems reject file names that are not UTF-8.
#[cfg(target_os = "linux")]
#[test]
fn inventory_reads_paths_and_links_as_git_stores_them() {
    use std::{ffi::OsStr, os::unix::ffi::OsStrExt};

    let dir = tempfile::tempdir().unwrap();
    let git = |args: &[&str]| {
        let status = Command::new("git")
            .args(args)
            .current_dir(dir.path())
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .env("GIT_CONFIG_SYSTEM", "/dev/null")
            .status()
            .unwrap();
        assert!(status.success(), "git {args:?}");
    };
    git(&["init", "-q"]);
    let project = dir.path().join("fix\\ture");
    std::fs::create_dir_all(&project).unwrap();
    std::fs::write(project.join("a.txt"), "a\n").unwrap();
    std::os::unix::fs::symlink(OsStr::from_bytes(b"bad-\xff"), project.join("link")).unwrap();
    std::fs::write(dir.path().join(OsStr::from_bytes(b"bad-\xff")), "x\n").unwrap();
    git(&["add", "."]);

    let output = Command::new(env!("CARGO_BIN_EXE_semantic-oracle"))
        .args(["inventory", "fix\\ture"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let text = stdout(&output);
    let paths: Vec<&str> = text
        .lines()
        .filter_map(|line| line.strip_prefix("  - path: "))
        .collect();
    assert_eq!(paths, ["a.txt", "link"]);
}

/// A project directory whose name is not UTF-8 still lists its files.
// macOS and Windows file systems reject file names that are not UTF-8.
#[cfg(target_os = "linux")]
#[test]
fn inventory_opens_a_project_whose_directory_name_is_not_utf8() {
    use std::{ffi::OsStr, os::unix::ffi::OsStrExt};

    let dir = tempfile::tempdir().unwrap();
    let git = |args: &[&str]| {
        let status = Command::new("git")
            .args(args)
            .current_dir(dir.path())
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .env("GIT_CONFIG_SYSTEM", "/dev/null")
            .status()
            .unwrap();
        assert!(status.success(), "git {args:?}");
    };
    git(&["init", "-q"]);
    let name = OsStr::from_bytes(b"proj-\xff");
    std::fs::create_dir_all(dir.path().join(name)).unwrap();
    std::fs::write(dir.path().join(name).join("a"), "a\n").unwrap();
    git(&["add", "."]);
    let output = Command::new(env!("CARGO_BIN_EXE_semantic-oracle"))
        .arg("inventory")
        .arg(name)
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(stdout(&output).contains("  - path: a\n"));
}

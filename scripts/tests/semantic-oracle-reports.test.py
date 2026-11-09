#!/usr/bin/env python3
# ---
# relationships:
#   references: semantic-oracle-model
# ---
"""Exercises each compiler report script on a small project in a temporary
directory, and checks slots it must state. Each script's toolchain is required: a
missing one fails this test rather than skipping it, so that a pass means all four
ran. C# and Dart have no script until their toolchains exist."""
import importlib.util
import json
import os
import pathlib
import platform
import shutil
import subprocess
import sys
import tempfile
import time
import zipfile

sys.dont_write_bytecode = True

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
SCRIPTS = os.path.join(ROOT, "scripts", "semantic-oracle")
TIMES = {}
RUNNING = f"{sys.version_info.major}.{sys.version_info.minor}"


def write(root, files):
    for name, text in files.items():
        path = os.path.join(root, name)
        os.makedirs(os.path.dirname(path), exist_ok=True)
        with open(path, "w") as out:
            out.write(text)


def members(out):
    with open(out) as source:
        report = json.load(source)
    assert report["format"] == "semantic-oracle-compiler-report/1"
    # The audit matches it against the context's release parameter.
    assert report["release"] and report["release"][0] in "0123456789g", report.get("release")
    # Rust's namespaces let one path name several kinds, keyed `<path>@<kind>` there.
    keyed = {}
    for member in report["members"]:
        keyed.setdefault(member["path"], member)
        if "kind" in member:
            keyed[f"{member['path']}@{member['kind']}"] = member
    return report, keyed


def expect(found, path, slot, value):
    actual = found[path]["slots"].get(slot)
    assert actual == value, f"{path} {slot}: {actual!r}, not {value!r}"


def run(language, command, cwd, out):
    started = time.monotonic()
    done = subprocess.run(command, cwd=cwd, capture_output=True, text=True)
    TIMES[language] = round(time.monotonic() - started, 2)
    assert done.returncode == 0, done.stderr + done.stdout
    return members(out)


def tree_unchanged(root, before):
    after = sorted(os.path.join(d, f) for d, _, files in os.walk(root) for f in files)
    assert after == before, f"the report wrote into the project: {set(after) - set(before)}"


def listing(root):
    return sorted(os.path.join(d, f) for d, _, files in os.walk(root) for f in files)


def rust(tmp):
    project = os.path.join(tmp, "rust")
    write(project, {
        "Cargo.toml": '[package]\nname = "sample"\nversion = "0.1.0"\nedition = "2021"\n[lib]\n[workspace]\n',
        "src/lib.rs": """
use std::rc::Rc;
pub struct Plain { pub radius: f64 }
pub struct Pair<T>(pub T);
pub struct Ref<'a, T>(pub &'a T);
pub struct Bounded<T: Clone + Send>(pub T);
pub struct Shared(pub Rc<u32>);
pub enum Kind { Round, Square }
pub const ORIGIN: u32 = 2 + 1;
pub fn make() -> impl Clone { 1u8 }
pub fn pair() -> (impl Clone, impl Clone) { (1u8, Rc::new(())) }
pub async fn plain_async() -> u8 { 1 }
pub async fn opaque_async() -> impl Clone { Rc::new(()) }
pub fn borrowed(x: u32, y: &str) -> impl Clone { let _ = y; x }
pub async fn owned_argument(x: String, y: &String) -> usize { x.len() + y.len() }
pub async fn takes_future<F: std::future::Future<Output = u8> + Send>(f: F) -> u8 { f.await }
pub fn vector() -> Vec<impl Clone> { vec![String::new()] }
pub fn iterates() -> impl Iterator<Item = impl Clone> { std::iter::once(Rc::new(())) }
pub fn generic<T: Clone>(t: T) -> impl Clone { t }
pub unsafe extern "C" fn convert(x: i32) -> i32 { x }
mod hidden { pub struct Reexported(pub std::rc::Rc<u8>); }
pub use hidden::Reexported;
pub struct Item;
pub trait Tr { fn made() -> impl Clone; }
impl Tr for Item { fn made() -> impl Clone { Rc::new(()) } }
pub trait Assoc { type Out; }
impl Assoc for Plain { type Out = u8; }
pub type Projected = <Plain as Assoc>::Out;
pub mod both { pub struct Inner; }
pub fn both() -> u32 { 1 }
pub struct Crossed;
pub enum Payload { Unit, Tuple(pub_type::Inner, u8), Named { field: u16 } }
pub mod pub_type { pub struct Inner; }
""",
    })
    before = listing(project)
    out = os.path.join(tmp, "rust.json")
    _, found = run("rust", [sys.executable, os.path.join(SCRIPTS, "report-rust.py"), "--project", project, "--context", "default", "--out", out], tmp, out)
    tree_unchanged(project, before)
    all5 = ["RefUnwindSafe", "Send", "Sync", "Unpin", "UnwindSafe"]
    for trait in all5:
        expect(found, "sample::Plain", f"auto-traits[{trait}]", trait)
        expect(found, "sample::make", f"auto-traits[{trait}]", trait)
        expect(found, "sample::Pair", f"auto-traits[{trait}]", f"{trait} when T: {trait}")
    assert "auto-traits[Send]" not in found["sample::Shared"]["slots"]
    assert "auto-traits[Send]" not in found["sample::hidden::Reexported"]["slots"]
    expect(found, "sample::Ref", "auto-traits[Send]", "Send when T: Sync")
    expect(found, "sample::Bounded", "auto-traits[Send]", "Send")
    expect(found, "sample::pair", "auto-traits[#0 Send]", "#0 Send")
    assert "auto-traits[#1 Send]" not in found["sample::pair"]["slots"]
    expect(found, "sample::opaque_async", "auto-traits[#0 Send]", "#0 Send")
    assert "auto-traits[#1 Send]" not in found["sample::opaque_async"]["slots"]
    expect(found, "sample::plain_async", "auto-traits[Send]", "Send")
    assert "auto-traits[Unpin]" not in found["sample::plain_async"]["slots"]
    expect(found, "sample::borrowed", "auto-traits[Send]", "Send")
    # A standard library name by a public path, and an argument `alloc` defines.
    expect(found, "sample::owned_argument", "auto-traits[Send]", "Send")
    assert not found["sample::owned_argument"]["unstated"]
    expect(found, "sample::takes_future", "auto-traits[Send]", "Send")
    # A generic future with no passing line for a trait states only that it could not tell.
    assert found["sample::takes_future"]["unstated"] == ["auto-traits[Unpin]"], found["sample::takes_future"]
    expect(found, "sample::vector", "auto-traits[#0 Send]", "#0 Send")
    expect(found, "sample::iterates", "auto-traits[#0 Unpin]", "#0 Unpin")
    assert "auto-traits[#1 Send]" not in found["sample::iterates"]["slots"]
    expect(found, "sample::generic", "auto-traits[Send]", "Send when T: Send")
    binding = "bases[@{sample::Tr}].bindings[made].auto-traits"
    expect(found, "sample::Item", f"{binding}[Unpin]", "Unpin")
    assert f"{binding}[Send]" not in found["sample::Item"]["slots"]
    expect(found, "sample::ORIGIN", "value", "3")
    expect(found, "sample::Kind::Square", "value", "1")
    expect(found, "sample::convert", "abi", "C")
    expect(found, "sample::Plain::radius", "type", "f64")
    expect(found, "sample::Projected", "type", "<@{sample::Plain} as @{sample::Assoc}>::@{sample::Assoc::Out}")
    expect(found, "sample::both@callable", "returns", "u32")
    expect(found, "sample::Payload::Tuple::0", "type", "@{sample::pub_type::Inner}")
    expect(found, "sample::Payload::Tuple::1", "type", "u8")
    expect(found, "sample::Payload::Named::field", "type", "u16")
    # The key holds the crate's version, so that another version's slot never answers.
    assert found["sample::Plain"]["unit"] == {"package": "sample", "package-version": "0.1.0"}, found["sample::Plain"]
    # A field has no `auto-traits`, so it does not read as a member the probe skipped.
    assert found["sample::Plain::radius"]["unstated"] == []


def failure(command, cwd, out, words, env=None):
    """A script that cannot check its project exits 2 and writes no report."""
    done = subprocess.run(command, cwd=cwd, capture_output=True, text=True, env=env)
    # `go run` reports any failure of its program as exit status 1.
    assert done.returncode != 0, f"{done.stdout} {done.stderr}"
    assert not os.path.exists(out), "a failed build wrote a report"
    for word in words:
        assert word in done.stderr, done.stderr


def rust_build_failures(tmp):
    project = os.path.join(tmp, "rust-features")
    write(project, {
        "Cargo.toml": '[package]\nname = "sample"\nversion = "0.1.0"\nedition = "2021"\n[lib]\n[features]\nchecked = []\n[workspace]\n',
        "src/lib.rs": """
#[cfg(feature = "checked")]
pub fn make() -> impl Clone { 1u8 }
#[cfg(not(feature = "checked"))]
pub fn make() -> impl Clone { std::rc::Rc::new(()) }
""",
    })
    script = [sys.executable, os.path.join(SCRIPTS, "report-rust.py"), "--project", project, "--context", "default"]
    # Two valid configurations with different auto traits: the probe reads the
    # configuration the context selects.
    out = os.path.join(tmp, "with.json")
    done = subprocess.run(script + ["--out", out, "--features", "checked"], capture_output=True, text=True)
    assert done.returncode == 0, done.stderr
    _, found = members(out)
    expect(found, "sample::make", "auto-traits[Send]", "Send")
    out = os.path.join(tmp, "without.json")
    done = subprocess.run(script + ["--out", out], capture_output=True, text=True)
    assert done.returncode == 0, done.stderr
    _, found = members(out)
    assert "auto-traits[Send]" not in found["sample::make"]["slots"]

    # A context's `cfg` selects what rustdoc and the probe read.
    configured = os.path.join(tmp, "rust-cfg")
    write(configured, {
        "Cargo.toml": '[package]\nname = "sample"\nversion = "0.1.0"\nedition = "2021"\n[lib]\n[workspace]\n[lints.rust]\nunexpected_cfgs = "allow"\n',
        "src/lib.rs": """
#[cfg(chosen)]
pub const VALUE: u32 = 1;
#[cfg(not(chosen))]
pub const VALUE: u32 = 2;
#[cfg(chosen)]
pub fn make() -> impl Clone { 1u8 }
#[cfg(not(chosen))]
pub fn make() -> impl Clone { std::rc::Rc::new(()) }
""",
    })
    command = [sys.executable, os.path.join(SCRIPTS, "report-rust.py"), "--project", configured, "--context", "default"]
    out = os.path.join(tmp, "cfg-on.json")
    done = subprocess.run(command + ["--out", out, "--cfg", "chosen"], capture_output=True, text=True)
    assert done.returncode == 0, done.stderr
    _, found = members(out)
    expect(found, "sample::VALUE", "value", "1")
    expect(found, "sample::make", "auto-traits[Send]", "Send")
    out = os.path.join(tmp, "cfg-off.json")
    done = subprocess.run(command + ["--out", out], capture_output=True, text=True)
    assert done.returncode == 0, done.stderr
    _, found = members(out)
    expect(found, "sample::VALUE", "value", "2")
    assert "auto-traits[Send]" not in found["sample::make"]["slots"]

    # A dependency that does not build certifies nothing.
    broken = os.path.join(tmp, "rust-broken")
    write(broken, {
        "Cargo.toml": '[package]\nname = "sample"\nversion = "0.1.0"\nedition = "2021"\n[lib]\n[features]\nchecked = []\n[workspace]\n',
        "src/lib.rs": """
#[cfg(all(not(doc), not(feature = "checked")))]
compile_error!("checked feature required");
pub fn make() -> impl Clone { std::rc::Rc::new(()) }
""",
    })
    out = os.path.join(tmp, "broken.json")
    command = [sys.executable, os.path.join(SCRIPTS, "report-rust.py"), "--project", broken, "--context", "default", "--out", out]
    failure(command, tmp, out, ["checked feature required"])
    # A project rustdoc cannot read certifies nothing either.
    undocumented = os.path.join(tmp, "rust-undocumented")
    write(undocumented, {
        "Cargo.toml": '[package]\nname = "sample"\nversion = "0.1.0"\nedition = "2021"\n[lib]\n[workspace]\n',
        "src/lib.rs": '#[cfg(doc)]\ncompile_error!("rustdoc cannot read this");\npub struct Plain;\n',
    })
    out_doc = os.path.join(tmp, "undocumented.json")
    failure([sys.executable, os.path.join(SCRIPTS, "report-rust.py"), "--project", undocumented, "--context", "default", "--out", out_doc], tmp, out_doc, ["rustdoc failed"])
    # A stale report does not survive a failed run.
    with open(out, "w") as stale:
        stale.write("{}")
    failure(command, tmp, out, [])
    # A report never lands in the project tree.
    inside = os.path.join(broken, "report.json")
    failure([sys.executable, os.path.join(SCRIPTS, "report-rust.py"), "--project", broken, "--context", "default", "--out", inside], tmp, inside, ["inside the project tree"])


def rust_workspace(tmp):
    """Each crate of a workspace gets its own probes, whichever crate comes first."""
    project = os.path.join(tmp, "rust-workspace")
    write(project, {
        "Cargo.toml": '[workspace]\nmembers = ["one", "two"]\nresolver = "2"\n',
        "one/Cargo.toml": '[package]\nname = "one"\nversion = "0.1.0"\nedition = "2021"\n',
        "one/src/lib.rs": "pub struct First;\n",
        "two/Cargo.toml": '[package]\nname = "two"\nversion = "0.1.0"\nedition = "2021"\n',
        "two/src/lib.rs": "pub struct Second;\n",
    })
    before = listing(project)
    out = os.path.join(tmp, "rust-workspace.json")
    _, found = run("rust", [sys.executable, os.path.join(SCRIPTS, "report-rust.py"), "--project", project, "--context", "default", "--out", out], tmp, out)
    tree_unchanged(project, before)
    for path in ("one::First", "two::Second"):
        assert found[path]["unstated"] == [], found[path]
        expect(found, path, "auto-traits[Send]", "Send")


def load_rust_report():
    specification = importlib.util.spec_from_file_location("report_rust", os.path.join(SCRIPTS, "report-rust.py"))
    module = importlib.util.module_from_spec(specification)
    specification.loader.exec_module(module)
    return module


def stops(function, *arguments):
    try:
        function(*arguments)
    except SystemExit as stop:
        assert stop.code == 2, stop.code
        return True
    return False


def rust_check_reading():
    """What a check's output proves, without running one."""
    report = load_rust_report()
    artifact = json.dumps({"reason": "compiler-artifact", "target": {"name": report.PROBE_TARGET}})
    assert report.read_check(0, artifact, "") == (set(), set())
    # A success that never checked the probe, and a failure with nothing to attribute.
    assert stops(report.read_check, 0, "", "")
    assert stops(report.read_check, 101, "", "")
    assert stops(report.read_check, 101, artifact, "")

    def message(target, code, spans):
        return json.dumps({"reason": "compiler-message", "target": {"name": target}, "message": {"level": "error", "code": {"code": code} if code else None, "spans": spans, "message": "m"}})

    line = [{"is_primary": True, "file_name": "src/lib.rs", "line_start": 9}]
    assert report.read_check(101, message(report.PROBE_TARGET, "E0277", line), "") == ({9}, set())
    assert report.read_check(101, message(report.PROBE_TARGET, "E0433", line), "") == (set(), {9})
    # An error in another crate's file of the same name, and one with no generated line.
    assert stops(report.read_check, 101, message("sample", "E0277", line), "")
    assert stops(report.read_check, 101, message(report.PROBE_TARGET, "E0463", []), "")
    # After a real failure too, an error with no generated line still stops the report.
    both = message(report.PROBE_TARGET, "E0277", line) + "\n" + message(report.PROBE_TARGET, "E0463", [])
    assert stops(report.read_check, 101, both, "")
    # The compiler's closing summary has no code, and says nothing new.
    summary = message(report.PROBE_TARGET, "E0277", line) + "\n" + message(report.PROBE_TARGET, None, [])
    assert report.read_check(101, summary, "") == ({9}, set())
    other_file = [{"is_primary": True, "file_name": "/elsewhere/src/lib.rs", "line_start": 9}]
    assert stops(report.read_check, 101, message(report.PROBE_TARGET, "E0277", other_file), "")

    class Selection:
        features = "a,missing"
        no_default_features = True
        all_features = False

    package = {"name": "sample", "manifest_path": "/p/Cargo.toml", "features": {"a": [], "b": []}}
    line = report.dependency_line(package, Selection)
    assert '"sample"' in line and '"/p"' in line and '["a"]' in line and "default-features = false" in line, line
    Selection.no_default_features = False
    Selection.all_features = True
    line = report.dependency_line(package, Selection)
    assert '["a", "b"]' in line and "default-features" not in line, line

    assert report.public_path("core::future::future::Future") == "core::future::Future"
    assert report.public_path("alloc::string::String") == "alloc::string::String"
    assert report.public_path("sample::a::b::C") == "sample::a::b::C"


def rust_toolchain_selection(tmp):
    """One toolchain, the one the project selects, for every command."""
    report = load_rust_report()
    bin_dir = os.path.join(tmp, "fake-rustup")
    os.makedirs(bin_dir)
    rustup = os.path.join(bin_dir, "rustup")
    with open(rustup, "w") as out:
        out.write('#!/bin/sh\necho "1.90.0-x86_64-unknown-linux-gnu (overridden by rust-toolchain.toml)"\n')
    os.chmod(rustup, 0o755)
    path = os.environ["PATH"]
    os.environ["PATH"] = bin_dir + os.pathsep + path
    try:
        report.select_toolchain(tmp)
    finally:
        os.environ["PATH"] = path
    assert report.TOOLCHAIN == {"RUSTUP_TOOLCHAIN": "1.90.0-x86_64-unknown-linux-gnu"}, report.TOOLCHAIN
    done = report.run([sys.executable, "-c", "import os; print(os.environ['RUSTUP_TOOLCHAIN'])"], tmp)
    assert done.stdout.strip() == "1.90.0-x86_64-unknown-linux-gnu", done.stdout


def go_build_failure(tmp):
    project = os.path.join(tmp, "go-broken")
    write(project, {
        "go.mod": "module example.com/sample\n\ngo 1.21\n",
        "sample.go": "package sample\n\nvar X int = \"text\"\n",
    })
    out = os.path.join(tmp, "go-broken.json")
    failure(["go", "run", os.path.join(SCRIPTS, "report-go.go"), "-context", "default", "-out", out], project, out, ["report-go"])


def typescript_build_failures(tmp):
    script = ["node", os.path.join(SCRIPTS, "report-typescript.cjs")]
    project = os.path.join(tmp, "ts-broken")
    write(project, {
        "tsconfig.json": json.dumps({"compilerOptions": {"noEmitOnError": True, "strict": True}, "include": ["input.ts"]}),
        "input.ts": "export const x: string = 1;\n",
    })
    out = os.path.join(tmp, "ts-broken.json")
    failure(script + ["--project", project, "--context", "x", "--out", out], tmp, out, ["TS2322"])
    project = os.path.join(tmp, "ts-bad-config")
    write(project, {"tsconfig.json": '{"compilerOptions": {"target": "nonsense"}}', "input.ts": "export const x = 1;\n"})
    out = os.path.join(tmp, "ts-bad-config.json")
    failure(script + ["--project", project, "--context", "x", "--out", out], tmp, out, ["report-typescript"])
    # A report never lands in the project tree.
    inside = os.path.join(project, "report.json")
    failure(script + ["--project", project, "--context", "x", "--out", inside], tmp, inside, ["inside the project tree"])
    # Each of the project's own emit destinations stays empty.
    for name, options in (("decl", {"declarationDir": "decl"}), ("out", {"outFile": "bundle.js", "module": "amd"})):
        project = os.path.join(tmp, f"ts-{name}")
        write(project, {
            "tsconfig.json": json.dumps({"compilerOptions": {"strict": True, "target": "es2022", **options}, "include": ["input.ts"]}),
            "input.ts": "export const x = 1;\nexport function y() { return 'a'; }\n",
        })
        before = listing(project)
        out = os.path.join(tmp, f"ts-{name}.json")
        _, found = run("typescript", script + ["--project", project, "--context", "x", "--out", out], tmp, out)
        tree_unchanged(project, before)
        assert '"input.ts"#y' in found and found['"input.ts"#y']["slots"]["returns"] == "string", found


def typescript_declaration_files(tmp):
    """A declaration-file project is checked, whichever way a file got in."""
    project = os.path.join(tmp, "ts-declarations")
    write(project, {
        "tsconfig.json": json.dumps({"compilerOptions": {"strict": True, "target": "es2022", "module": "commonjs", "moduleResolution": "node10", "noEmit": False, "types": []}, "include": ["src/*.ts", "ambient/*.d.ts"]}),
        "src/use.ts": "import { read } from '../packages/lib/dist';\nimport { readScoped } from '@scope/name';\nexport const a = read('x');\nexport const b = readScoped('y');\n",
        "packages/lib/dist/index.d.ts": "export declare function read(value: string): string;\nexport interface Marker { readonly modern: true; }\n",
        "ambient/scope.d.ts": "declare module '@scope/name' {\n  export function readScoped(value: string): string;\n  export interface ScopedMarker { source: 'scoped'; }\n  export function make(): ScopedMarker;\n}\n",
    })
    before = listing(project)
    out = os.path.join(tmp, "ts-declarations.json")
    _, found = run("typescript", ["node", os.path.join(SCRIPTS, "report-typescript.cjs"), "--project", project, "--context", "x", "--out", out], tmp, out)
    tree_unchanged(project, before)
    # Reached only by resolution, and not a root.
    expect(found, '"packages/lib/dist/index.d.ts"#read', "returns", "string")
    expect(found, '"packages/lib/dist/index.d.ts"#Marker.modern', "type", "true")
    # An emitted declaration's import resolves as it does for a consumer.
    expect(found, '"src/use.ts"#a', "type", "string")
    # A member of an ambient module, under its `ambient:` path.
    expect(found, 'ambient:"@scope/name"#readScoped', "parameters[0].type", "string")
    expect(found, 'ambient:"@scope/name"#ScopedMarker.source', "type", "'scoped'")
    # A name that denotes an ambient member, under the same path.
    expect(found, 'ambient:"@scope/name"#make', "returns", '@{ambient:"@scope/name"#ScopedMarker}')


def go(tmp):
    project = os.path.join(tmp, "go")
    write(project, {
        "go.mod": "module example.com/sample\n\ngo 1.21\n",
        "sample.go": """package sample

type Base struct {
	Label string `json:"label"`
}
type Shape interface{ Area() float64 }
type Circle struct {
	Base
	radius float64
}
func (c *Circle) Area() float64 { return c.radius }
type Meters = float64
const A = 1
const Pi = 3.14159
const Name string = "x"
const Large = 0x1p4095
const NotLarge = 0x1p4094
const Small = 0x1p-4097
const Mixed = 0x1.8p5000
const Negative = -0x1p4095
const ComputedOne = 1e5000 * 1e-5000
const ComputedMinusOne = -(1e5000 * 1e-5000)
const ComputedZero = 1e5000 - 1e5000
const ComputedRatio = 1e5000 * 1.5e-5000
const ComputedLarge = 1e5000 * 1e-5000 * 0x1p4095
const ComputedNotLarge = 1e5000 * 1e-5000 * 0x1p4094
const ComputedComplex = 1e5000*1e-5000 + 1i
const ComputedImaginary = (1e5000 * 1.5e-5000) * 1i
const NearlyOne = 1e5000 / 5 * 5e-5000
const SameAsNearlyOne = 1 + 1.0/(1<<511)
var Table = map[string]int{}
func Greet(names ...string) string { return "" }
func Two(x, y int) (int, error) { return 0, nil }
type Pair[T any] struct{ Left, Right T }
type Num interface{ ~string | ~int }
type Named interface {
	Shape
	Name() string
}
type opener interface{ Open() string }
type Shelf struct{}
func (Shelf) Open() string { return "" }
""",
        "sample_test.go": """package sample

func Helper() int { return 1 }
type Probe struct{}
func (Probe) Area() float64 { return 0 }
""",
        "external_test.go": """package sample_test

import "example.com/sample"

var External = sample.Helper()
""",
    })
    before = listing(project)
    out = os.path.join(tmp, "go.json")
    report, found = run("go", ["go", "run", os.path.join(SCRIPTS, "report-go.go"), "-context", "default", "-out", out], project, out)
    tree_unchanged(project, before)
    path = "example.com/sample."
    expect(found, path + "A", "type", "untyped int")
    expect(found, path + "A", "value", "1")
    expect(found, path + "Pi", "value", "314159/100000")
    expect(found, path + "Name", "value", '"x"')
    # The big float form the value check accepts, as `go/constant` prints it.
    expect(found, path + "Large", "value", "0x.8p+4096")
    expect(found, path + "Small", "value", "0x.8p-4096")
    expect(found, path + "Mixed", "value", "0x.cp+5001")
    expect(found, path + "Negative", "value", "-0x.8p+4096")
    assert found[path + "NotLarge"]["slots"]["value"].isdigit(), found[path + "NotLarge"]
    # A computed value has the spelling of the same value written as a literal.
    expect(found, path + "ComputedOne", "value", "1")
    expect(found, path + "ComputedMinusOne", "value", "-1")
    expect(found, path + "ComputedZero", "value", "0")
    expect(found, path + "ComputedRatio", "value", "3/2")
    expect(found, path + "ComputedLarge", "value", "0x.8p+4096")
    expect(found, path + "ComputedComplex", "value", "(1 + 1i)")
    expect(found, path + "ComputedImaginary", "value", "(0 + 3/2i)")
    expect(found, path + "ComputedNotLarge", "value", found[path + "NotLarge"]["slots"]["value"])
    # An equal value computed two ways, and not rounded to the integer next to it.
    nearly_one = found[path + "NearlyOne"]["slots"]["value"]
    assert "/" in nearly_one, nearly_one
    expect(found, path + "SameAsNearlyOne", "value", nearly_one)
    expect(found, path + "Table", "type", "map[string]int")
    expect(found, path + "Greet", "parameters[0].type", "...string")
    expect(found, path + "Two", "returns", "(int, error)")
    expect(found, path + "Circle.Area", "receiver", "*@{example.com/sample.Circle}")
    expect(found, path + "Circle", "underlying", "struct { @{example.com/sample.Base}; radius float64 }")
    expect(found, path + "Base.Label", "tag", '"json:\\"label\\""')
    expect(found, path + "Meters", "type", "float64")
    expect(found, path + "Pair", "type-parameters[0].bounds", "any")
    expect(found, path + "Num", "underlying", "interface { ~string | ~int }")
    # `bases`, keyed by the base's path: an embedded interface, an implemented one
    # through `*T`, and an unexported one.
    def bases(name):
        return sorted(slot for slot in found[path + name]["slots"] if slot.startswith("bases"))
    expect(found, path + "Named", "bases[@{example.com/sample.Shape}].type", "@{example.com/sample.Shape}")
    expect(found, path + "Circle", "bases[@{example.com/sample.Shape}].type", "@{example.com/sample.Shape}")
    expect(found, path + "Shelf", "bases[@{example.com/sample.opener}].type", "@{example.com/sample.opener}")
    assert bases("Circle") == ["bases[@{example.com/sample.Shape}].type"], bases("Circle")
    assert bases("Base") == [] and bases("Shape") == [] and bases("Num") == [], found[path + "Base"]
    # go/types cannot decide this for an uninstantiated generic type.
    assert "bases" in found[path + "Pair"]["unstated"], found[path + "Pair"]
    # A build reads no test file.
    assert path + "Helper" not in found and "example.com/sample_test.External" not in found
    assert "bases[].type" in report["covers"], report["covers"]

    # Mode `test` reads the in-package `_test.go` files and the external `_test`
    # package, which sees the test files' names.
    out = os.path.join(tmp, "go-test.json")
    _, found = run("go-test", ["go", "run", os.path.join(SCRIPTS, "report-go.go"), "-context", "test", "-mode", "test", "-out", out], project, out)
    tree_unchanged(project, before)
    expect(found, path + "Helper", "returns", "int")
    expect(found, path + "Probe", "bases[@{example.com/sample.Shape}].type", "@{example.com/sample.Shape}")
    expect(found, "example.com/sample_test.External", "type", "int")
    expect(found, path + "Circle", "bases[@{example.com/sample.Shape}].type", "@{example.com/sample.Shape}")


def go_test_graph(tmp):
    """A test binary rebuilds each package that imports the tested one against its
    test variant, so that one named type stays one type: `q` imports `p`, and `p`'s
    tests and external test package use both."""
    project = os.path.join(tmp, "go-graph")
    write(project, {
        "go.mod": "module example.com/graph\n\ngo 1.21\n",
        "p/p.go": "package p\n\ntype X struct{}\ntype T struct{}\n\nfunc (T) M(X) {}\n",
        "p/p_test.go": "package p\n\nfunc helper() int { return 1 }\n",
        "p/x_test.go": """package p_test

import (
	"example.com/graph/p"
	"example.com/graph/q"
)

var Used = q.Accept(p.T{})
""",
        "q/q.go": "package q\n\nimport \"example.com/graph/p\"\n\ntype I interface{ M(p.X) }\n\nfunc Accept(I) bool { return true }\n",
    })
    base = "bases[@{example.com/graph/q.I}].type"
    for mode in ("build", "test"):
        out = os.path.join(tmp, f"go-graph-{mode}.json")
        report, found = run(f"go-graph-{mode}", ["go", "run", os.path.join(SCRIPTS, "report-go.go"), "-context", mode, "-mode", mode, "-out", out], project, out)
        assert report["mode"] == mode, report["mode"]
        expect(found, "example.com/graph/p.T", base, "@{example.com/graph/q.I}")
    expect(found, "example.com/graph/p_test.Used", "type", "bool")


def go_test_binaries(tmp):
    """Each test binary's graph holds the variants it links, and no other package.
    `p`'s tests give `T` a method `M`. `q`, which no test of `p` links, embeds `T`
    beside another `M`, and declares an interface `T` would implement; `r`, which
    `p`'s external test links, declares another."""
    project = os.path.join(tmp, "go-binaries")
    write(project, {
        "go.mod": "module example.com/binaries\n\ngo 1.21\n",
        "p/p.go": "package p\n\ntype T struct{}\n",
        "p/p_test.go": "package p\n\nfunc (T) M() {}\n",
        "p/x_test.go": "package p_test\n\nimport \"example.com/binaries/r\"\n\nvar _ = r.R{}\n",
        "q/q.go": """package q

import "example.com/binaries/p"

type I interface{ M() }
type O struct{}

func (O) M() {}

type S struct {
	p.T
	O
}

func F(s S) { s.M() }
""",
        "r/r.go": "package r\n\nimport \"example.com/binaries/p\"\n\ntype Mer interface{ M() }\ntype R struct{ p.T }\n",
    })
    t = "example.com/binaries/p.T"
    found = {}
    for mode in ("build", "test"):
        out = os.path.join(tmp, f"go-binaries-{mode}.json")
        _, found[mode] = run(f"go-binaries-{mode}", ["go", "run", os.path.join(SCRIPTS, "report-go.go"), "-context", mode, "-mode", mode, "-out", out], project, out)
    assert not any(slot.startswith("bases") for slot in found["build"][t]["slots"]), found["build"][t]
    # In `p.test`, `T` has `M` and `r` is linked; `q` is not.
    expect(found["test"], t, "bases[@{example.com/binaries/r.Mer}].type", "@{example.com/binaries/r.Mer}")
    assert "bases[@{example.com/binaries/q.I}].type" not in found["test"][t]["slots"], found["test"][t]
    expect(found["test"], t + ".M", "receiver", "@{" + t + "}")


def go_replaced_module(tmp):
    """A dependency module that imports the project sees the project's types: `b`,
    replaced by a local directory, takes a `p.T`, and the project passes one."""
    project = os.path.join(tmp, "go-replaced")
    write(project, {
        "go.mod": "module example.com/a\n\ngo 1.21\n\nrequire example.com/b v0.0.0\n\nreplace example.com/b => ../go-replaced-b\n",
        "p/p.go": "package p\n\ntype T struct{}\n",
        "use.go": """package a

import (
	"example.com/a/p"
	"example.com/b"
)

var Used = b.Accept(p.T{})
""",
    })
    write(os.path.join(tmp, "go-replaced-b"), {
        "go.mod": "module example.com/b\n\ngo 1.21\n",
        "b.go": "package b\n\nimport \"example.com/a/p\"\n\nfunc Accept(p.T) bool { return true }\n",
    })
    for mode in ("build", "test"):
        out = os.path.join(tmp, f"go-replaced-{mode}.json")
        _, found = run(f"go-replaced-{mode}", ["go", "run", os.path.join(SCRIPTS, "report-go.go"), "-context", mode, "-mode", mode, "-out", out], project, out)
        expect(found, "example.com/a.Used", "type", "bool")
        # A dependency's members are no project member.
        assert "example.com/b.Accept" not in found, found["example.com/b.Accept"]


def go_dependency_build_failure(tmp):
    """Dependencies are type-checked without function bodies, so the report runs the
    context's real build: an error in a function body of a local `replace` module
    fails the report, in mode build and in mode test, and writes nothing. One that
    only a `_test.go` file links fails mode test alone."""
    command = ["go", "run", os.path.join(SCRIPTS, "report-go.go")]
    project = os.path.join(tmp, "go-dependency-broken")
    write(project, {
        "go.mod": "module example.com/a\n\ngo 1.21\n\nrequire example.com/b v0.0.0\n\nreplace example.com/b => ../go-dependency-broken-b\n",
        "a.go": "package a\n\nimport \"example.com/b\"\n\nvar Used = b.Value()\n",
    })
    write(os.path.join(tmp, "go-dependency-broken-b"), {
        "go.mod": "module example.com/b\n\ngo 1.21\n",
        "b.go": "package b\n\nfunc Value() int { return \"text\" }\n",
    })
    for mode in ("build", "test"):
        out = os.path.join(tmp, f"go-dependency-broken-{mode}.json")
        failure(command + ["-context", mode, "-mode", mode, "-out", out], project, out, ["report-go", "does not build", "b.go"])

    project = os.path.join(tmp, "go-test-only-broken")
    write(project, {
        "go.mod": "module example.com/c\n\ngo 1.21\n\nrequire example.com/d v0.0.0\n\nreplace example.com/d => ../go-test-only-broken-d\n",
        "c.go": "package c\n\nconst A = 1\n",
        "c_test.go": "package c\n\nimport \"example.com/d\"\n\nvar _ = d.Value()\n",
        # A command builds an executable, which must not land in the project.
        "cmd/tool/main.go": "package main\n\nfunc main() {}\n",
    })
    write(os.path.join(tmp, "go-test-only-broken-d"), {
        "go.mod": "module example.com/d\n\ngo 1.21\n",
        "d.go": "package d\n\nfunc Value() int { return \"text\" }\n",
    })
    before = listing(project)
    out = os.path.join(tmp, "go-test-only-broken-build.json")
    _, found = run("go-test-only-broken-build", command + ["-context", "build", "-out", out], project, out)
    expect(found, "example.com/c.A", "value", "1")
    out = os.path.join(tmp, "go-test-only-broken-test.json")
    failure(command + ["-context", "test", "-mode", "test", "-out", out], project, out, ["report-go", "does not build", "d.go"])
    tree_unchanged(project, before)


def host_report_go(tmp):
    """`go run` under a context's GOOS would build the script for that platform:
    build it once for the host, to run it under another GOOS."""
    binary = os.path.join(tmp, "report-go-host" + (".exe" if os.name == "nt" else ""))
    if not os.path.exists(binary):
        subprocess.run(["go", "build", "-o", binary, os.path.join(SCRIPTS, "report-go.go")], check=True)
    return binary


def go_build_every_package(tmp):
    """Mode build compiles every selected package, not only the commands: a library
    beside a command, whose replaced dependency has an error in a function body,
    fails the report, as `go build ./...` does, for the host and for Windows."""
    command = ["go", "run", os.path.join(SCRIPTS, "report-go.go")]
    project = os.path.join(tmp, "go-library-broken")
    write(project, {
        "go.mod": "module example.com/e\n\ngo 1.21\n\nrequire example.com/f v0.0.0\n\nreplace example.com/f => ../go-library-broken-f\n",
        "cmd/tool/main.go": "package main\n\nfunc main() {}\n",
        "lib/lib.go": "package lib\n\nimport \"example.com/f\"\n\nvar Used = f.Value()\n",
    })
    write(os.path.join(tmp, "go-library-broken-f"), {
        "go.mod": "module example.com/f\n\ngo 1.21\n",
        "f.go": "package f\n\nfunc Value() int { return \"bad\" }\n",
    })
    before = listing(project)
    out = os.path.join(tmp, "go-library-broken-host.json")
    failure(command + ["-context", "build", "-out", out], project, out, ["report-go", "does not build", "f.go"])
    out = os.path.join(tmp, "go-library-broken-windows.json")
    windows = dict(os.environ, GOOS="windows")
    failure([host_report_go(tmp), "-context", "build", "-out", out], project, out, ["report-go", "does not build", "f.go"], windows)
    tree_unchanged(project, before)


def go_test_binaries_same_name(tmp):
    """Mode test builds every test binary, as `go test` does: two tested packages of
    one name do not collide, and the report runs neither binary, nor its `init`."""
    project = os.path.join(tmp, "go-same-name")
    write(project, {
        "go.mod": "module example.com/g\n\ngo 1.21\n",
        "one/same/same.go": "package same\n\nconst One = 1\n",
        "one/same/same_test.go": "package same\n\nimport \"testing\"\n\nfunc TestOne(t *testing.T) {}\n",
        "two/same/same.go": "package same\n\nconst Two = 2\n",
        "two/same/same_test.go": "package same\n\nimport (\n\t\"os\"\n\t\"testing\"\n)\n\nfunc init() { os.WriteFile(\"ran\", nil, 0o644); panic(\"ran\") }\n\nfunc TestTwo(t *testing.T) {}\n",
        "cmd/tool/main.go": "package main\n\nfunc main() {}\n",
    })
    before = listing(project)
    command = ["go", "run", os.path.join(SCRIPTS, "report-go.go"), "-context", "test", "-mode", "test", "-out"]
    out = os.path.join(tmp, "go-same-name.json")
    _, found = run("go-same-name", command + [out], project, out)
    expect(found, "example.com/g/one/same.One", "value", "1")
    expect(found, "example.com/g/two/same.Two", "value", "2")
    # Cross-compiled test binaries are linked, and never executed.
    out = os.path.join(tmp, "go-same-name-windows.json")
    done = subprocess.run([host_report_go(tmp), "-context", "test", "-mode", "test", "-out", out], cwd=project, capture_output=True, text=True, env=dict(os.environ, GOOS="windows"))
    assert done.returncode == 0, done.stderr + done.stdout
    _, found = members(out)
    expect(found, "example.com/g/two/same.Two", "value", "2")
    tree_unchanged(project, before)


def go_module_flag(tmp):
    """`-mod=mod` would add a missing requirement to `go.mod`, so the report refuses
    it, from GOFLAGS or from the go env file, and leaves `go.mod` and `go.sum` as
    they were. Without it, a missing requirement fails rather than being added."""
    project = os.path.join(tmp, "go-missing-require")
    write(project, {
        "go.mod": "module example.com/h\n\ngo 1.21\n\nreplace example.com/i v0.0.0 => ../go-missing-require-i\n",
        "go.sum": "",
        "h.go": "package h\n\nimport \"example.com/i\"\n\nconst V = i.V\n",
    })
    write(os.path.join(tmp, "go-missing-require-i"), {
        "go.mod": "module example.com/i\n\ngo 1.21\n",
        "i.go": "package i\n\nconst V = 1\n",
    })
    def contents():
        found = {}
        for name in ("go.mod", "go.sum"):
            with open(os.path.join(project, name), "rb") as source:
                found[name] = source.read()
        return found
    before, files = contents(), listing(project)
    goenv = os.path.join(tmp, "go-missing-require.env")
    with open(goenv, "w") as config:
        config.write("GOFLAGS=-mod=mod\n")
    command = ["go", "run", os.path.join(SCRIPTS, "report-go.go"), "-context", "build", "-out"]
    for name, env in (
        ("flags", dict(os.environ, GOFLAGS="-mod=mod")),
        ("goenv", dict(os.environ, GOENV=goenv)),
        ("tested", dict(os.environ, GOFLAGS="-mod=mod")),
        ("default", None),
    ):
        out = os.path.join(tmp, f"go-missing-require-{name}.json")
        words = ["report-go"] + (["-mod=mod"] if name != "default" else [])
        mode = ["-mode", "test"] if name == "tested" else []
        failure(command + [out] + mode, project, out, words, env)
        assert contents() == before, f"{name}: the report rewrote go.mod or go.sum"
        tree_unchanged(project, files)


def snapshot(root):
    found = {}
    for path in listing(root):
        with open(path, "rb") as source:
            found[path] = source.read()
    return found


def go_proxy(tmp, name):
    """An offline module proxy, served as files, holding `example.com/x v1.0.0`:
    a module the go command downloads, so it has a checksum to record."""
    proxy = os.path.join(tmp, name)
    module, version = "example.com/x", "v1.0.0"
    versions = os.path.join(proxy, module, "@v")
    gomod = f"module {module}\n\ngo 1.21\n"
    write(versions, {
        "list": version + "\n",
        f"{version}.info": json.dumps({"Version": version, "Time": "2020-01-01T00:00:00Z"}),
        f"{version}.mod": gomod,
    })
    with zipfile.ZipFile(os.path.join(versions, f"{version}.zip"), "w") as archive:
        archive.writestr(f"{module}@{version}/go.mod", gomod)
        archive.writestr(f"{module}@{version}/x.go", "package x\n\nconst V = 1\n")
    return pathlib.Path(proxy).as_uri()


def go_workspace_sums(tmp):
    """`-mod=readonly` does not stop the go command recording a missing checksum in
    `go.work.sum`: in workspace mode the report runs against a copy of `go.work`
    outside the project, and leaves the tree byte for byte as it was, with or
    without a `go.work.sum`, and removes the copy. A vendored workspace, whose
    `vendor/modules.txt` names its local replacements relative to `go.work`, keeps
    its own `go.work` and records nothing."""
    proxy = go_proxy(tmp, "go-sums-proxy")
    temporary = os.path.join(tmp, "go-sums-tmp")
    os.makedirs(temporary)
    os.makedirs(os.path.join(tmp, "go-sums-gotmp"))
    env = dict(os.environ, GOPROXY=proxy, GOSUMDB="off", GOFLAGS="-modcacherw", GOTOOLCHAIN="local",
               GOMODCACHE=os.path.join(tmp, "go-sums-cache"), GOTMPDIR=os.path.join(tmp, "go-sums-gotmp"), TMPDIR=temporary, TMP=temporary, TEMP=temporary)
    root = os.path.join(tmp, "go-sums")
    write(root, {
        "go.work": "go 1.21\n\nuse ./project\n\nreplace example.com/y => ./y\n",
        "project/go.mod": "module example.com/p\n\ngo 1.21\n\nrequire (\n\texample.com/x v1.0.0\n\texample.com/y v0.0.0\n)\n",
        "project/p.go": "package p\n\nimport (\n\t\"example.com/x\"\n\t\"example.com/y\"\n)\n\nconst V = x.V + y.W\n",
        "project/p_test.go": "package p\n\nimport \"testing\"\n\nfunc TestV(t *testing.T) {}\n",
        "y/go.mod": "module example.com/y\n\ngo 1.21\n",
        "y/y.go": "package y\n\nconst W = 2\n",
    })
    command = ["go", "run", os.path.join(SCRIPTS, "report-go.go"), "-patterns", "./project/...", "-out"]
    def report(name, mode, cwd, environment):
        out = os.path.join(tmp, f"go-sums-{name}-{mode}.json")
        started = time.monotonic()
        done = subprocess.run(command + [out, "-context", mode, "-mode", mode], cwd=cwd, env=environment, capture_output=True, text=True)
        TIMES[f"go-sums-{name}-{mode}"] = round(time.monotonic() - started, 2)
        assert done.returncode == 0, f"{name} {mode}: " + done.stderr + done.stdout
        _, found = members(out)
        expect(found, "example.com/p.V", "type", "untyped int")
        assert os.listdir(temporary) == [], f"{name} {mode}: the workspace copy remains: {os.listdir(temporary)}"
    for name, sums in (("unrecorded", None), ("recorded", "example.com/z v1.0.0/go.mod h1:AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=\n")):
        if sums is not None:
            write(root, {"go.work.sum": sums})
        before = snapshot(root)
        for mode in ("build", "test"):
            report(name, mode, root, env)
            assert snapshot(root) == before, f"{name} {mode}: the report wrote into the workspace: {sorted(set(snapshot(root)) ^ set(before))}"
    # A workspace whose vendor directory has no `modules.txt` written for a
    # workspace ignores it and reads the module cache, so the copy is still used.
    for case, version, modules in (("empty-121", "1.21", None), ("empty-122", "1.22", None), ("empty-122rc1", "1.22rc1", None),
                                   ("module-modules-txt", "1.22", "# example.com/x v1.0.0\n## explicit\nexample.com/x\n"),
                                   # The go command reads only the first 512 bytes.
                                   ("annotation-past-512", "1.22", "## " + " " * 510 + "workspace\n")):
        ignored = os.path.join(tmp, f"go-sums-ignored-{case}")
        shutil.copytree(root, ignored)
        os.remove(os.path.join(ignored, "go.work.sum"))
        with open(os.path.join(ignored, "go.work"), "w") as handle:
            handle.write(f"go {version}\n\nuse ./project\n\nreplace example.com/y => ./y\n")
        os.makedirs(os.path.join(ignored, "vendor"))
        if modules is not None:
            write(ignored, {"vendor/modules.txt": modules})
        before = snapshot(ignored)
        for mode in ("build", "test"):
            done = subprocess.run(command + [os.path.join(tmp, f"go-sums-ignored-{case}-{mode}.json"), "-context", mode, "-mode", mode],
                                  cwd=ignored, env=env, capture_output=True, text=True)
            assert snapshot(ignored) == before, f"ignored {case} {mode}: the report wrote into the workspace: {sorted(set(snapshot(ignored)) ^ set(before))}"
            assert os.listdir(temporary) == [], f"ignored {case} {mode}: the workspace copy remains"
            if modules is None:
                assert done.returncode == 0, f"ignored {case} {mode}: " + done.stderr
    vendored = os.path.join(tmp, "go-sums-vendored")
    shutil.copytree(root, vendored)
    with open(os.path.join(vendored, "go.work"), "w") as handle:
        handle.write("go 1.22\n\nuse ./project\n\nreplace example.com/y => ./y\n")
    subprocess.run(["go", "work", "vendor"], cwd=vendored, env=env, check=True, capture_output=True)
    os.remove(os.path.join(vendored, "go.work.sum"))
    before = snapshot(vendored)
    # Vendor mode reads no module: no proxy, no module cache.
    offline = dict(env, GOPROXY="off", GOMODCACHE=os.path.join(tmp, "go-sums-empty-cache"))
    for mode in ("build", "test"):
        report("vendored", mode, vendored, offline)
        assert snapshot(vendored) == before, f"vendored {mode}: the report wrote into the workspace"


def go_workspace(tmp):
    """From a `go.work` root, `./...` matches no module: the report exits nonzero
    and writes nothing, and the context's patterns report every module."""
    root = os.path.join(tmp, "go-workspace")
    write(root, {
        "go.work": "go 1.21\n\nuse (\n\t./a\n\t./b\n)\n",
        "a/go.mod": "module example.com/a\n\ngo 1.21\n",
        "a/p/p.go": "package p\n\ntype T struct{}\n",
        "b/go.mod": "module example.com/b\n\ngo 1.21\n",
        "b/b.go": "package b\n\nimport \"example.com/a/p\"\n\nfunc Accept(p.T) bool { return true }\n",
    })
    command = ["go", "run", os.path.join(SCRIPTS, "report-go.go"), "-context", "build", "-out"]
    out = os.path.join(tmp, "go-workspace.json")
    failure(command + [out], root, out, ["match no package"])
    _, found = run("go-workspace", command + [out, "-patterns", "./a/... ./b/..."], root, out)
    expect(found, "example.com/b.Accept", "parameters[0].type", "@{example.com/a/p.T}")
    assert "example.com/a/p.T" in found, sorted(found)


def go_cgo(tmp):
    """A cgo type that go/types cannot resolve leaves its slot unstated."""
    project = os.path.join(tmp, "go-cgo")
    write(project, {
        "go.mod": "module example.com/cgo\n\ngo 1.21\n",
        "c.go": """package c

// typedef int number;
// #define FOO 42
import "C"

const N = C.FOO
const Size = C.sizeof_int

type Box struct{ V C.number }

func Get() C.number { return 0 }
func Plain() int { return 0 }
const Message = "invalid type"
""",
    })
    out = os.path.join(tmp, "go-cgo.json")
    command = ["go", "run", os.path.join(SCRIPTS, "report-go.go"), "-context", "build", "-out", out]
    started = time.monotonic()
    done = subprocess.run(command, cwd=project, capture_output=True, text=True, env={**os.environ, "CGO_ENABLED": "1"})
    TIMES["go-cgo"] = round(time.monotonic() - started, 2)
    assert done.returncode == 0, done.stderr
    report, found = members(out)
    path = "example.com/cgo."
    text = json.dumps(report)
    assert "invalid type" not in text.replace('"\\"invalid type\\""', ""), text
    assert "returns" in found[path + "Get"]["unstated"], found[path + "Get"]
    assert "type" in found[path + "Box.V"]["unstated"], found[path + "Box.V"]
    assert "underlying" in found[path + "Box"]["unstated"], found[path + "Box"]
    expect(found, path + "Plain", "returns", "int")
    expect(found, path + "Message", "value", '"invalid type"')
    # A cgo constant's value is unknown to go/types, and stated as no value.
    for name in ("N", "Size"):
        assert "value" in found[path + name]["unstated"], found[path + name]
        assert "value" not in found[path + name]["slots"], found[path + name]


def go_listing_errors(tmp):
    """A pattern or package the go command cannot load fails the report, even
    beside a package that loads."""
    project = os.path.join(tmp, "go-listing")
    write(project, {
        "go.mod": "module example.com/listing\n\ngo 1.21\n",
        "ok.go": "package listing\n\nconst A = 1\n",
        "broken/b.go": "package broken\n\nimport \"example.com/listing/absent\"\n\nvar _ = absent.X\n",
    })
    out = os.path.join(tmp, "go-listing.json")
    command = ["go", "run", os.path.join(SCRIPTS, "report-go.go"), "-context", "build", "-out", out]
    failure(command + ["-patterns", "./ ./missing"], project, out, ["cannot load every package", "missing"])
    failure(command + ["-patterns", "./..."], project, out, ["cannot load every package", "absent"])
    run("go-listing", command + ["-patterns", "."], project, out)


def python(tmp):
    project = os.path.join(tmp, "python")
    write(project, {
        "pkg/__init__.py": "",
        "pkg/raises/__init__.py": "raise RuntimeError('on purpose')\n",
        "pkg/raises/child.py": "VALUE = 1\n",
        "pkg/exits/__init__.py": "raise SystemExit(3)\n",
        "pkg/aliases.py": """
from typing import TypeAlias
FLAG = True
Imported: TypeAlias = int
Chain: TypeAlias = Imported
""",
        "pkg/mod.py": """
from dataclasses import dataclass
from enum import Enum
from typing import Optional
from typing import Literal
Status = Literal["open", "closed"]
from typing import TypeAlias, List, Text, TypeVar
from .aliases import Imported as Renamed, Chain
from . import aliases
Scalar: TypeAlias = int
First: TypeAlias = list[int]
Second: TypeAlias = First
Implicit = int
T = TypeVar("T")
type Modern = int
if aliases.FLAG:
    Conditional = int
else:
    Conditional = str
def scalar(x: Scalar) -> Scalar:
    return x
def second(x: Second) -> Second:
    return x
def first(x: First) -> First:
    return x
def implicit(x: Implicit) -> Implicit:
    return x
def modern(x: Modern) -> Modern:
    return x
def imported(x: Renamed, y: Chain, z: aliases.Imported) -> aliases.Chain:
    return x
def standard(x: List[int], y: T) -> T:
    return y
def renamed_external(x: Text) -> None:
    pass
def conditional(x: Conditional) -> None:
    pass
try:
    from .aliases import Imported as Either
except ImportError:
    Either = str
def either(x: Either) -> None:
    pass
def uses_alias(a: Status) -> Status:
    return a
def quoted(a: "Status", b: 'Literal["a"]') -> None:
    pass
def unresolvable(a: "NotDefinedAnywhere") -> None:
    pass
def plain(a: int, b: "str" = "x", *args: int, c: Optional[int] = None, **kw: float) -> list[int]:
    return []
def unannotated(a, b=1 + 1):
    return a
class Color(Enum):
    RED = 1
@dataclass
class Point:
    x: int
    y: int = 0
    @property
    def norm(self) -> float:
        return 0.0
""",
    })
    before = listing(project)
    out = os.path.join(tmp, "python.json")
    report, found = run("python", [sys.executable, os.path.join(SCRIPTS, "report-python.py"), "--search-path", project, "--context", "runtime", "--out", out, "--python", RUNNING], tmp, out)
    # A package that raises on import is listed, and the report goes on.
    failed = " ".join(report["failed"])
    assert "pkg.raises: RuntimeError" in failed and "pkg.exits: SystemExit" in failed, report["failed"]
    tree_unchanged(project, before)
    # A declared alias keeps the name written, not its value and not another alias
    # of the same value.
    for slot in ("parameters[0].type", "returns"):
        expect(found, "pkg.mod.scalar", slot, "@{pkg.mod.Scalar}")
        expect(found, "pkg.mod.second", slot, "@{pkg.mod.Second}")
        expect(found, "pkg.mod.first", slot, "@{pkg.mod.First}")
        expect(found, "pkg.mod.implicit", slot, "@{pkg.mod.Implicit}")
        expect(found, "pkg.mod.modern", slot, "@{pkg.mod.Modern}")
    # An import resolves to the statement that binds the alias, under any name.
    expect(found, "pkg.mod.imported", "parameters[0].type", "@{pkg.aliases.Imported}")
    expect(found, "pkg.mod.imported", "parameters[1].type", "@{pkg.aliases.Chain}")
    expect(found, "pkg.mod.imported", "parameters[2].type", "@{pkg.aliases.Imported}")
    expect(found, "pkg.mod.imported", "returns", "@{pkg.aliases.Chain}")
    # A standard name and a type variable are not aliases.
    expect(found, "pkg.mod.standard", "parameters[0].type", "@{typing.List}[int]")
    expect(found, "pkg.mod.standard", "parameters[1].type", "T")
    # An external name that does not answer to its own name is unknown.
    assert "parameters[0].type" not in found["pkg.mod.renamed_external"]["slots"]
    assert found["pkg.mod.renamed_external"]["unstated"] == ["parameters[0].type"], found["pkg.mod.renamed_external"]
    # Two assignments of one name are one alias, whichever runs.
    expect(found, "pkg.mod.conditional", "parameters[0].type", "@{pkg.mod.Conditional}")
    # An import and an assignment of one name: the source does not fix which it is.
    assert found["pkg.mod.either"]["unstated"] == ["parameters[0].type"], found["pkg.mod.either"]
    # The written name, resolved: the runtime's expansion is not the slot's text.
    expect(found, "pkg.mod.uses_alias", "parameters[0].type", "@{pkg.mod.Status}")
    expect(found, "pkg.mod.uses_alias", "returns", "@{pkg.mod.Status}")
    expect(found, "pkg.mod.quoted", "parameters[0].type", "@{pkg.mod.Status}")
    expect(found, "pkg.mod.quoted", "parameters[1].type", "@{typing.Literal}[\"a\"]")
    expect(found, "pkg.mod.unresolvable", "parameters[0].name", "a")
    assert "parameters[0].type" not in found["pkg.mod.unresolvable"]["slots"]
    assert found["pkg.mod.unresolvable"]["unstated"] == ["parameters[0].type"], found["pkg.mod.unresolvable"]
    python_without_written_text(project)
    python_interpreter(project, tmp)
    expect(found, "pkg.mod.plain", "parameters[0].type", "int")
    expect(found, "pkg.mod.plain", "parameters[1].type", "str")
    expect(found, "pkg.mod.plain", "parameters[1].default", '"x"')
    expect(found, "pkg.mod.plain", "parameters[2].mode", "var-positional")
    expect(found, "pkg.mod.plain", "parameters[3].mode", "keyword-only")
    expect(found, "pkg.mod.plain", "parameters[3].type", "@{typing.Optional}[int]")
    expect(found, "pkg.mod.plain", "parameters[3].default", "None")
    expect(found, "pkg.mod.plain", "returns", "list[int]")
    assert "returns" not in found["pkg.mod.unannotated"]["slots"]
    expect(found, "pkg.mod.unannotated", "parameters[1].default", "2")
    expect(found, "pkg.mod.Color.RED", "value", "1")
    expect(found, "pkg.mod.Point.__init__", "parameters[1].type", "int")
    expect(found, "pkg.mod.Point.norm", "get.type", "float")


def python_interpreter(project, tmp):
    """A report runs under the context's interpreter, says so, and never under another."""
    script = [sys.executable, os.path.join(SCRIPTS, "report-python.py"), "--search-path", project, "--context", "runtime"]
    out = os.path.join(tmp, "python-running.json")
    report, _ = run("python", script + ["--out", out, "--python", RUNNING], tmp, out)
    assert report["tool"]["version"].startswith(RUNNING + "."), report["tool"]
    assert report["tool"]["interpreter"], report["tool"]
    absent = os.path.join(tmp, "python-absent.json")
    failure(script + ["--out", absent, "--python", "3.99"], tmp, absent, ["python3.99 is absent"])
    # A context that names the micro version runs under it, and under no other.
    exact = platform.python_version()
    out = os.path.join(tmp, "python-exact.json")
    report, _ = run("python", script + ["--out", out, "--python", exact], tmp, out)
    assert report["release"] == exact, report["release"]
    other_micro = f"{RUNNING}.{sys.version_info.micro + 1}"
    wrong = os.path.join(tmp, "python-wrong-micro.json")
    failure(script + ["--out", wrong, "--python", other_micro], tmp, wrong, [f"not the context's {other_micro}"])
    absent_micro = os.path.join(tmp, "python-absent-micro.json")
    failure(script + ["--out", absent_micro, "--python", "3.99.1"], tmp, absent_micro, ["python3.99 is absent"])
    # Another interpreter on PATH answers for its version, whichever started the script.
    other = next((v for v in ("3.14", "3.15", "3.13", "3.12") if v != RUNNING and shutil.which(f"python{v}")), None)
    if other is None:
        print("python_interpreter: no second interpreter on PATH, re-execution not exercised")
        return
    out = os.path.join(tmp, "python-other.json")
    report, _ = run("python", script + ["--out", out, "--python", other], tmp, out)
    assert report["tool"]["version"].startswith(other + "."), report["tool"]


def python_without_written_text(project):
    """An interpreter that returns only the evaluated annotation has lost the alias's
    name: the slot is unstated, and is neither an error nor agreed."""
    specification = importlib.util.spec_from_file_location("report_python", os.path.join(SCRIPTS, "report-python.py"))
    module = importlib.util.module_from_spec(specification)
    specification.loader.exec_module(module)
    module.STRING_FORMAT = None
    module.PROJECT_ROOTS.append(project)
    sys.path.insert(0, project)
    try:
        imported = importlib.import_module("pkg.mod")
        module.remember_aliases(imported)
        report = module.Report()
        report.module(imported)
        member = report.members["pkg.mod.uses_alias"]
        assert "parameters[0].type" not in member["slots"], member
        assert member["unstated"] == ["parameters[0].type", "returns"], member
        # A declared alias of a class too: its value cannot be told from its name.
        member = report.members["pkg.mod.scalar"]
        assert "returns" not in member["slots"] and "returns" in member["unstated"], member
        # A form an assignment binds nowhere is still stated.
        assert report.members["pkg.mod.plain"]["slots"]["parameters[1].default"] == '"x"'
        # An evaluated `int` cannot be told from an alias of it, inside a form too.
        assert "returns" in report.members["pkg.mod.plain"]["unstated"]
    finally:
        sys.path.remove(project)
        for name in [name for name in sys.modules if name == "pkg" or name.startswith("pkg.")]:
            del sys.modules[name]


def typescript(tmp):
    project = os.path.join(tmp, "typescript")
    write(project, {
        "tsconfig.json": json.dumps({"compilerOptions": {"target": "es2022", "module": "nodenext", "moduleResolution": "nodenext", "strict": True}, "include": ["src"]}),
        "package.json": '{"type":"module"}',
        "src/shapes.ts": """
export interface Shape { area(): number; label?: string; (v: string): string; }
export class Circle implements Shape2 { constructor(readonly radius: number) {} get diameter() { return this.radius * 2; } area() { return 1; } }
interface Shape2 {}
export function inferred() { return "a"; }
export function over(a: string): string;
export function over(a: number): number;
export function over(a: any) { return a; }
export const made = new Circle(1);
export type Id = string | number;
export enum Kind { Round = 3 }
export function dflt(first: number = 1, second: string): string { return second; }
export interface Sized extends Shape2 { size: number; }
export class Box<T> { constructor(readonly item: T) {} }
export class Crate extends Box<number> implements Sized, Shape2 { size = 1; }
function mix<B extends new (...a: any[]) => object>(b: B) { return class extends b { mixed = true; }; }
export class Plain {}
export class Mixed extends mix(Plain) {}
""",
    })
    before = listing(project)
    out = os.path.join(tmp, "typescript.json")
    _, found = run("typescript", ["node", os.path.join(SCRIPTS, "report-typescript.cjs"), "--project", project, "--context", "esm", "--out", out], tmp, out)
    tree_unchanged(project, before)
    path = '"src/shapes.ts"#'
    expect(found, path + "inferred", "returns", "string")
    expect(found, path + "over", "overloads[0].parameters[0].type", "string")
    expect(found, path + "over", "overloads[1].returns", "number")
    expect(found, path + "Circle", "constructors[0].parameters[0].type", "number")
    expect(found, path + "Circle.area", "returns", "number")
    expect(found, path + "Circle.diameter", "get.type", "number")
    expect(found, path + "made", "type", f"@{{{path}Circle}}")
    expect(found, path + "Id", "type", "string | number")
    expect(found, path + "Kind.Round", "value", "3")
    expect(found, path + "dflt", "parameters[0].type", "number | undefined")
    assert "parameters[0].mode" not in found[path + "dflt"]["slots"]
    expect(found, path + "Shape", "call-signatures[0].returns", "string")
    expect(found, path + "Shape.label", "modifiers[optional]", "optional")
    # `bases`, keyed by the base's text with its names as paths.
    shape2 = f"@{{{path}Shape2}}"
    expect(found, path + "Circle", f"bases[{shape2}].type", shape2)
    expect(found, path + "Sized", f"bases[{shape2}].type", shape2)
    box = f"@{{{path}Box}}<number>"
    expect(found, path + "Crate", f"bases[{box}].type", box)
    expect(found, path + "Crate", f"bases[@{{{path}Sized}}].type", f"@{{{path}Sized}}")
    expect(found, path + "Crate", f"bases[{shape2}].type", shape2)
    # A mixin's base is a variable the emit generates, which no base can name.
    assert "bases" in found[path + "Mixed"]["unstated"], found[path + "Mixed"]
    assert not any(slot.startswith("bases") for slot in found[path + "Mixed"]["slots"]), found[path + "Mixed"]


def main():
    tmp = tempfile.mkdtemp(prefix="semantic-oracle-report-test-")
    try:
        rust_check_reading()
        rust_toolchain_selection(tmp)
        for test in (rust, rust_workspace, rust_build_failures, go, go_test_graph, go_test_binaries, go_replaced_module, go_dependency_build_failure, go_build_every_package, go_test_binaries_same_name, go_module_flag, go_workspace_sums, go_workspace, go_cgo, go_listing_errors, go_build_failure, python, typescript, typescript_build_failures, typescript_declaration_files):
            test(tmp)
    finally:
        shutil.rmtree(tmp, ignore_errors=True)
    print("report seconds:", json.dumps(TIMES))


if __name__ == "__main__":
    main()

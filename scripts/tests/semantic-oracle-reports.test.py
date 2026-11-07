#!/usr/bin/env python3
# ---
# relationships:
#   references: semantic-oracle-model
# ---
"""Exercises each compiler report script on a small project in a temporary
directory, and checks slots it must state. Each script's toolchain is required: a
missing one fails this test rather than skipping it, so that a pass means all five
ran. C# has no script until its toolchain exists. The Dart script's dependencies come
from `dart pub get` in scripts/semantic-oracle."""
import importlib.util
import json
import os
import platform
import shutil
import subprocess
import sys
import tempfile
import time

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


def failure(command, cwd, out, words):
    """A script that cannot check its project exits 2 and writes no report."""
    done = subprocess.run(command, cwd=cwd, capture_output=True, text=True)
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
""",
    })
    before = listing(project)
    out = os.path.join(tmp, "go.json")
    _, found = run("go", ["go", "run", os.path.join(SCRIPTS, "report-go.go"), "-context", "default", "-out", out], project, out)
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


def dart_command(project, out, context, entry, platform, *flags):
    """`platform` is required; None leaves `--platform` out."""
    platform_flags = [] if platform is None else ["--platform", platform]
    # A project with no dataset is reported standalone, unless a flag names one.
    has_dataset = os.path.exists(os.path.join(project, "semantic-oracle.yml")) or "--dataset" in flags
    standalone = [] if has_dataset or "--no-standalone" in flags else ["--standalone"]
    flags = [f for f in flags if f != "--no-standalone"]
    return ["dart", "run", os.path.join(SCRIPTS, "report-dart.dart"), "--project", project,
            "--context", context, "--entry", entry, "--out", out, *platform_flags, *standalone, *flags]


def dart(tmp):
    project = os.path.join(tmp, "dart")
    write(project, {
        "pubspec.yaml": "name: sample\nversion: 1.0.0\nenvironment:\n  sdk: '>=3.12.0 <4.0.0'\n",
        "lib/sample.dart": """import 'platform_stub.dart' if (dart.library.io) 'platform_io.dart';
part 'sample_part.dart';

typedef Callback = int Function(String label, {required int width});
const int answer = 42;
const String greeting = 'hi';
const double ratio = 1.5;
final List<String>? names = null;
enum Color { red, green }
enum Level {
  low(1), high(2);
  const Level(this.n);
  const Level.named() : n = 0;
  factory Level.of(int n) => n > 1 ? high : low;
  final int n;
}
const Color favorite = Color.green;
abstract base class Shape<T extends num> {
  Shape.named(this.size);
  factory Shape.unit() => throw UnimplementedError();
  final T size;
  static int count = 0;
  int get area;
  set area(covariant int value) {}
  T scale([T? by, int times = 2]) => size;
  Future<Map<String, T>> group({required String key, Color color = Color.red}) async => {};
  static Shape<int>? make<U extends Object>(U seed) => null;
}
mixin Labelled on Object, Comparable<int> {}
extension Doubling<E> on List<E> {
  List<E> doubled() => [...this, ...this];
}
extension on int {
  int get twice => this * 2;
}
extension type Meters(double value) {}
(int, {String name}) pair() => (1, name: 'x');
void ignore(int _, String _) {}
String platformName() => selected();
class Base<T> {
  T get x => throw 0;
  set y(T value) {}
}
class Sub extends Base<int> {
  set x(int value) {}
  int get y => 0;
}
mixin class Both {}
mixin OnObject on Object {}
mixin Plain {}
""",
        "lib/sample_part.dart": "part of 'sample.dart';\n\nint fromPart() => 1;\n",
        "lib/platform_stub.dart": "String selected() => 'stub';\n",
        "lib/platform_io.dart": "String selected() => 'io';\n",
        "lib/src/hidden.dart": "class Hidden {}\n",
        "bin/main.dart": "import 'package:sample/sample.dart';\n\nvoid main() => print(answer);\n",
    })
    before = listing(project)
    out = os.path.join(tmp, "dart.json")
    report, found = run("dart", dart_command(project, out, "analysis", "analysis", "vm"), project, out)
    tree_unchanged(project, before)
    assert report["release"].startswith("3."), report["release"]
    path = "package:sample/sample.dart::"
    expect(found, path + "Callback", "type", "int Function(String, {required int width})")
    expect(found, path + "answer", "type", "int")
    expect(found, path + "answer", "value", "42")
    expect(found, path + "greeting", "value", '"hi"')
    expect(found, path + "ratio", "value", "1.5")
    expect(found, path + "favorite", "value", "@{package:sample/sample.dart::Color.green}")
    expect(found, path + "names", "type", "@{dart:core::List}<String>?")
    expect(found, path + "names", "modifiers[final]", "final")
    expect(found, path + "Color.values", "type", "@{dart:core::List}<@{package:sample/sample.dart::Color}>")
    # An enum value's index is its position in `values`.
    expect(found, path + "Color.red", "index", "0")
    expect(found, path + "Color.green", "index", "1")
    expect(found, path + "Level.high", "index", "1")
    # A consumer can neither call nor tear off an enum's generative constructor;
    # it can a factory constructor.
    assert path + "Level.new" not in found and path + "Level.named" not in found, sorted(found)
    expect(found, path + "Level.of", "parameters[0].type", "int")
    expect(found, path + "Shape", "modifiers[abstract]", "abstract")
    expect(found, path + "Shape", "modifiers[base]", "base")
    expect(found, path + "Shape", "type-parameters[0].bounds", "num")
    expect(found, path + "Shape.named", "parameters[0].type", "T")
    expect(found, path + "Shape.size", "type", "T")
    expect(found, path + "Shape.count", "modifiers[static]", "static")
    expect(found, path + "Shape.area", "get.type", "int")
    expect(found, path + "Shape.area", "set.type", "int")
    expect(found, path + "Shape.area", "set.modifiers[covariant]", "covariant")
    expect(found, path + "Shape.scale", "parameters[0].mode", "optional")
    expect(found, path + "Shape.scale", "parameters[0].default", "null")
    expect(found, path + "Shape.scale", "parameters[1].default", "2")
    expect(found, path + "Shape.scale", "returns", "T")
    expect(found, path + "Shape.group", "parameters[0].mode", "required-named")
    assert "parameters[0].default" not in found[path + "Shape.group"]["slots"]
    expect(found, path + "Shape.group", "parameters[1].default", "@{package:sample/sample.dart::Color.red}")
    expect(found, path + "Shape.group", "returns", "@{dart:async::Future}<@{dart:core::Map}<String, T>>")
    expect(found, path + "Shape.make", "returns", "@{package:sample/sample.dart::Shape}<int>?")
    expect(found, path + "Shape.make", "type-parameters[0].bounds", "Object")
    expect(found, path + "Shape.make", "modifiers[static]", "static")
    assert "modifiers[static]" not in found[path + "pair"]["slots"]
    expect(found, path + "Labelled", "bounds", "@{dart:core::Comparable}<int>, Object")
    expect(found, path + "Doubling", "receiver", "@{dart:core::List}<E>")
    expect(found, path + "Doubling", "type-parameters[0].name", "E")
    expect(found, path + "Doubling.doubled", "returns", "@{dart:core::List}<E>")
    expect(found, path + "extension#1", "receiver", "int")
    expect(found, path + "extension#1.twice", "get.type", "int")
    expect(found, path + "Meters", "underlying", "double")
    # The unnamed constructor's consumer path segment is `new`.
    expect(found, path + "Meters.new", "parameters[0].type", "double")
    expect(found, path + "Meters.new", "parameters[0].name", "value")
    expect(found, path + "pair", "returns", "(int, {String name})")
    # A wildcard parameter binds no name.
    assert "parameters[0].name" not in found[path + "ignore"]["slots"]
    expect(found, path + "ignore", "parameters[1].type", "String")
    expect(found, path + "fromPart", "returns", "int")
    # An accessor half the class does not declare comes from its inherited
    # interface, with the supertype's type arguments substituted.
    expect(found, path + "Sub.x", "get.type", "int")
    expect(found, path + "Sub.x", "set.type", "int")
    expect(found, path + "Sub.y", "get.type", "int")
    expect(found, path + "Sub.y", "set.type", "int")
    expect(found, path + "Base.x", "get.type", "T")
    assert "set.type" not in found[path + "Base.x"]["slots"]
    expect(found, path + "Both", "modifiers[mixin]", "mixin")
    # A written `on Object` is a bound; the implicit constraint is none.
    expect(found, path + "OnObject", "bounds", "Object")
    assert "bounds" not in found[path + "Plain"]["slots"]
    assert "package:sample/src/hidden.dart::Hidden" in found
    assert "bin/main.dart::main" not in found
    # A context's entry reaches the libraries its directives select.
    out = os.path.join(tmp, "dart-vm.json")
    _, found = run("dart", dart_command(project, out, "vm", "bin/main.dart", "vm", "--library-flag", "dart.library.io"), project, out)
    assert "bin/main.dart::main" in found and "package:sample/platform_io.dart::selected" in found, sorted(found)
    assert "package:sample/platform_stub.dart::selected" not in found
    assert "package:sample/src/hidden.dart::Hidden" not in found
    # A js context resolves against the analyzer's SDK when the platform provides
    # every `dart:` library the context reaches.
    out = os.path.join(tmp, "dart-js.json")
    _, found = run("dart", dart_command(project, out, "js", "bin/main.dart", "js"), project, out)
    assert "package:sample/platform_stub.dart::selected" in found, sorted(found)
    tree_unchanged(project, before)


def dart_build_failure(tmp):
    project = os.path.join(tmp, "dart-broken")
    write(project, {
        "pubspec.yaml": "name: broken\nenvironment:\n  sdk: '>=3.12.0 <4.0.0'\n",
        "lib/broken.dart": "int x = 'text';\n",
    })
    out = os.path.join(tmp, "dart-broken.json")
    with open(out, "w") as stale:
        stale.write("{}")
    failure(dart_command(project, out, "analysis", "analysis", "vm"), project, out, ["report-dart", "broken.dart"])
    # `dart compile js` rejects a library the js platform does not provide.
    mirrors = os.path.join(tmp, "dart-mirrors")
    write(mirrors, {
        "pubspec.yaml": "name: mirrors\nenvironment:\n  sdk: '>=3.12.0 <4.0.0'\n",
        "lib/reflect.dart": "import 'dart:mirrors';\n\nString name(Object o) => reflect(o).toString();\n",
        "bin/main.dart": "import 'package:mirrors/reflect.dart';\n\nvoid main() => print(name(1));\n",
    })
    out = os.path.join(tmp, "dart-mirrors-js.json")
    failure(dart_command(mirrors, out, "js", "bin/main.dart", "js"), mirrors, out, ["dart:mirrors", "js"])
    out = os.path.join(tmp, "dart-mirrors-vm.json")
    run("dart", dart_command(mirrors, out, "vm", "bin/main.dart", "vm"), mirrors, out)
    # A platform is never assumed: a js context left without one would pass as vm.
    out = os.path.join(tmp, "dart-mirrors-none.json")
    failure(dart_command(mirrors, out, "js", "bin/main.dart", None), mirrors, out, ["missing --platform"])
    failure(dart_command(project, os.path.join(project, "out.json"), "analysis", "analysis", "vm"), project,
            os.path.join(project, "out.json"), ["inside the project tree"])


def dart_dependency_platform(tmp):
    """The platform check reads the packages a project depends on; members are
    reported for project libraries only."""
    dep = os.path.join(tmp, "dart-dep")
    write(dep, {
        "pubspec.yaml": "name: dep\nenvironment:\n  sdk: '>=3.12.0 <4.0.0'\n",
        "lib/dep.dart": "import 'dart:mirrors';\n\nString name(Object o) => reflect(o).toString();\n",
    })
    project = os.path.join(tmp, "dart-uses-dep")
    write(project, {
        "pubspec.yaml": "name: user\nenvironment:\n  sdk: '>=3.12.0 <4.0.0'\n"
                        f"dependencies:\n  dep:\n    path: {dep}\n",
        "bin/main.dart": "import 'package:dep/dep.dart';\n\nvoid main() => print(name(1));\n",
    })
    subprocess.run(["dart", "pub", "get", "--offline"], cwd=project, check=True, capture_output=True)
    out = os.path.join(tmp, "dart-dep-js.json")
    failure(dart_command(project, out, "js", "bin/main.dart", "js"), project, out,
            ["package:dep/dep.dart", "dart:mirrors", "js"])
    out = os.path.join(tmp, "dart-dep-wasm.json")
    failure(dart_command(project, out, "wasm", "bin/main.dart", "wasm"), project, out, ["dart:mirrors", "wasm"])
    out = os.path.join(tmp, "dart-dep-vm.json")
    _, found = run("dart", dart_command(project, out, "vm", "bin/main.dart", "vm"), project, out)
    assert "bin/main.dart::main" in found, sorted(found)
    assert not any(path.startswith("package:dep/") for path in found), sorted(found)


def dart_dataset_agreement(tmp):
    """A report checks its arguments against the dataset's context parameters."""
    project = os.path.join(tmp, "dart-dataset")
    write(project, {
        "pubspec.yaml": "name: agreed\nenvironment:\n  sdk: '>=3.12.0 <4.0.0'\n",
        "bin/main.dart": "void main() {}\n",
        "semantic-oracle.yml": """contexts:
  - id: js
    parameters:
      platform: js
      entry: bin/main.dart
      defines: [MODE=web]
      library-flags: [dart.library.core, dart.library.html]
      options: []
      release: '3.13'
    command: dart compile js bin/main.dart
""",
    })
    agreed = ["--define", "MODE=web", "--library-flag", "dart.library.html", "--library-flag", "dart.library.core"]
    out = os.path.join(tmp, "dart-dataset.json")
    run("dart", dart_command(project, out, "js", "bin/main.dart", "js", *agreed), project, out)
    # The project's dataset is read by default.
    failure(dart_command(project, out, "js", "bin/main.dart", "vm", *agreed), project, out,
            ["platform", '"js"', '"vm"'])
    failure(dart_command(project, out, "js", "bin/main.dart", "js", *agreed[2:]), project, out, ["defines"])
    failure(dart_command(project, out, "js", "bin/main.dart", "js", *agreed[:4]), project, out, ["library-flags"])
    failure(dart_command(project, out, "js", "bin/other.dart", "js", *agreed), project, out, ["entry"])
    failure(dart_command(project, out, "vm", "bin/main.dart", "vm"), project, out, ["no context `vm`"])
    # `--dataset` names another dataset.
    elsewhere = os.path.join(tmp, "dart-dataset-elsewhere.yml")
    with open(elsewhere, "w") as dataset:
        dataset.write("contexts:\n  - id: vm\n    parameters: {platform: vm, entry: bin/main.dart, defines: [], library-flags: []}\n")
    run("dart", dart_command(project, out, "vm", "bin/main.dart", "vm", "--dataset", elsewhere), project, out)
    # With no dataset at all, the report refuses unless it is told to run standalone.
    bare = os.path.join(tmp, "dart-no-dataset")
    write(bare, {"pubspec.yaml": "name: bare\nenvironment:\n  sdk: '>=3.12.0 <4.0.0'\n", "bin/main.dart": "void main() {}\n"})
    lone = os.path.join(tmp, "dart-no-dataset.json")
    failure(dart_command(bare, lone, "nonexistent-web-context", "bin/main.dart", "vm", "--no-standalone"), bare, lone,
            ["no dataset", "--standalone"])
    done = subprocess.run(dart_command(bare, lone, "nonexistent-web-context", "bin/main.dart", "vm"),
                          cwd=bare, capture_output=True, text=True)
    assert done.returncode == 0 and os.path.exists(lone), done.stderr
    assert "warning: --standalone" in done.stderr, done.stderr


def main():
    tmp = tempfile.mkdtemp(prefix="semantic-oracle-report-test-")
    try:
        rust_check_reading()
        rust_toolchain_selection(tmp)
        for test in (rust, rust_workspace, rust_build_failures, go, go_build_failure, python, typescript, typescript_build_failures, typescript_declaration_files, dart, dart_build_failure,
                     dart_dependency_platform, dart_dataset_agreement):
            test(tmp)
    finally:
        shutil.rmtree(tmp, ignore_errors=True)
    print("report seconds:", json.dumps(TIMES))


if __name__ == "__main__":
    main()

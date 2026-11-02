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
import shutil
import subprocess
import sys
import tempfile
import time

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
SCRIPTS = os.path.join(ROOT, "scripts", "semantic-oracle")
TIMES = {}


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
        "pkg/mod.py": """
from dataclasses import dataclass
from enum import Enum
from typing import Optional
from typing import Literal
Status = Literal["open", "closed"]
def uses_alias(a: Status) -> Status:
    return a
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
    _, found = run("python", [sys.executable, os.path.join(SCRIPTS, "report-python.py"), "--search-path", project, "--context", "runtime", "--out", out], tmp, out)
    tree_unchanged(project, before)
    expect(found, "pkg.mod.uses_alias", "parameters[0].type", "@{pkg.mod.Status}")
    expect(found, "pkg.mod.plain", "parameters[0].type", "int")
    expect(found, "pkg.mod.plain", "parameters[1].type", "str")
    expect(found, "pkg.mod.plain", "parameters[1].default", '"x"')
    expect(found, "pkg.mod.plain", "parameters[2].mode", "var-positional")
    expect(found, "pkg.mod.plain", "parameters[3].mode", "keyword-only")
    expect(found, "pkg.mod.plain", "parameters[3].type", "int | None")
    expect(found, "pkg.mod.plain", "parameters[3].default", "None")
    expect(found, "pkg.mod.plain", "returns", "list[int]")
    assert "returns" not in found["pkg.mod.unannotated"]["slots"]
    expect(found, "pkg.mod.unannotated", "parameters[1].default", "2")
    expect(found, "pkg.mod.Color.RED", "value", "1")
    expect(found, "pkg.mod.Point.__init__", "parameters[1].type", "int")
    expect(found, "pkg.mod.Point.norm", "get.type", "float")


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


def main():
    tmp = tempfile.mkdtemp(prefix="semantic-oracle-report-test-")
    try:
        rust_check_reading()
        rust_toolchain_selection(tmp)
        for test in (rust, rust_workspace, rust_build_failures, go, go_build_failure, python, typescript, typescript_build_failures, typescript_declaration_files):
            test(tmp)
    finally:
        shutil.rmtree(tmp, ignore_errors=True)
    print("report seconds:", json.dumps(TIMES))


if __name__ == "__main__":
    main()

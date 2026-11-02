#!/usr/bin/env python3
# ---
# relationships:
#   references: semantic-oracle-model
# ---
"""Exercises each compiler report script on a small project in a temporary
directory, and checks slots it must state. Each script's toolchain is required: a
missing one fails this test rather than skipping it, so that a pass means all four
ran. C# and Dart have no script until their toolchains exist."""
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
    return report, {m["path"]: m for m in report["members"]}


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
pub fn generic<T: Clone>(t: T) -> impl Clone { t }
pub unsafe extern "C" fn convert(x: i32) -> i32 { x }
mod hidden { pub struct Reexported(pub std::rc::Rc<u8>); }
pub use hidden::Reexported;
pub struct Item;
pub trait Tr { fn made() -> impl Clone; }
impl Tr for Item { fn made() -> impl Clone { Rc::new(()) } }
""",
    })
    subprocess.run(["cargo", "generate-lockfile", "--offline"], cwd=project, capture_output=True)
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
    expect(found, "sample::generic", "auto-traits[Send]", "Send when T: Send")
    binding = "bases[@{sample::Tr}].bindings[made].auto-traits"
    expect(found, "sample::Item", f"{binding}[Unpin]", "Unpin")
    assert f"{binding}[Send]" not in found["sample::Item"]["slots"]
    expect(found, "sample::ORIGIN", "value", "3")
    expect(found, "sample::Kind::Square", "value", "1")
    expect(found, "sample::convert", "abi", "C")
    expect(found, "sample::Plain::radius", "type", "f64")


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
        for test in (rust, go, python, typescript):
            test(tmp)
    finally:
        shutil.rmtree(tmp, ignore_errors=True)
    print("report seconds:", json.dumps(TIMES))


if __name__ == "__main__":
    main()

// ---
// relationships:
//   implements: semantic-oracle-model
//   references: semantic-oracle-profile-go
// ---

// Writes a Go compiler report for `semantic-oracle audit --report`, from go/types.
//
// Run it from the project root, so that the go command finds the module:
//
//	go run report-go.go -context ID -out FILE [-tags a,b] [-mode build|test]
//
// It parses and type-checks each package outside any build output, and writes
// nothing into the project tree. GOOS and GOARCH select a context's platform. In
// mode `test` it checks each package with its in-package `_test.go` files, and the
// external `_test` package, as `go test` compiles them. Before it writes a report it
// runs the context's real build, `go build`, or in mode `test` `go test` with a
// no-op in place of each test binary, and a context that does not build exits
// nonzero with the build's output. It refuses `-mod=mod`, which would rewrite
// `go.mod`, and otherwise loads with `-mod=readonly` unless a vendor directory is
// in use. In workspace mode it runs against a copy of `go.work` outside the
// project, so that a checksum the go command records lands in no `go.work.sum` of
// the project.
package main

import (
	"encoding/json"
	"flag"
	"fmt"
	"go/ast"
	"go/build"
	"go/constant"
	"go/importer"
	"go/parser"
	"go/token"
	"go/types"
	"math/big"
	"os"
	"os/exec"
	"os/signal"
	"path/filepath"
	"runtime"
	"sort"
	"strconv"
	"strings"
	"syscall"
)

// listed is one package of `go list -deps [-test] -json`. A test binary's
// variants carry the binary in their import path, as `p [p.test]`, and
// `ImportMap` names the variant each import resolves to.
type listed struct {
	Dir        string
	ImportPath string
	Name       string
	GoFiles    []string
	CgoFiles   []string
	Deps       []string
	ImportMap  map[string]string
	Standard   bool
	Error      *struct{ Err string }
	DepsErrors []struct{ Err string }
	Module     *struct{ Main bool }
}

// unresolved marks a type text go/types could not resolve, such as a cgo type
// under FakeImportC. No canonical text holds a NUL, and a string value is quoted.
const unresolved = "\x00invalid"

type member struct {
	Path     string            `json:"path"`
	Slots    map[string]string `json:"slots"`
	Unstated []string          `json:"unstated"`
}

var members = map[string]*member{}

func add(path, slot, value string) {
	found := members[path]
	if found == nil {
		found = &member{Path: path, Slots: map[string]string{}, Unstated: []string{}}
		members[path] = found
	}
	if slot == "" {
		return
	}
	// A slot whose type go/types could not resolve is unstated, never stated as
	// `invalid type`.
	if strings.Contains(value, unresolved) {
		for _, unstated := range found.Unstated {
			if unstated == slot {
				return
			}
		}
		found.Unstated = append(found.Unstated, slot)
		return
	}
	found.Slots[slot] = value
}

func typeText(t types.Type) string {
	switch t := t.(type) {
	case *types.Basic:
		if t.Kind() == types.Invalid {
			return unresolved
		}
		return t.Name()
	case *types.Alias:
		object := t.Obj()
		if object.Pkg() == nil {
			return object.Name()
		}
		return "@{" + object.Pkg().Path() + "." + object.Name() + "}" + typeArguments(t.TypeArgs())
	case *types.Named:
		object := t.Obj()
		if object.Pkg() == nil {
			return object.Name()
		}
		return "@{" + object.Pkg().Path() + "." + object.Name() + "}" + typeArguments(t.TypeArgs())
	case *types.Pointer:
		return "*" + typeText(t.Elem())
	case *types.Slice:
		return "[]" + typeText(t.Elem())
	case *types.Array:
		return fmt.Sprintf("[%d]%s", t.Len(), typeText(t.Elem()))
	case *types.Map:
		return "map[" + typeText(t.Key()) + "]" + typeText(t.Elem())
	case *types.Chan:
		switch t.Dir() {
		case types.SendOnly:
			return "chan<- " + typeText(t.Elem())
		case types.RecvOnly:
			return "<-chan " + typeText(t.Elem())
		}
		return "chan " + typeText(t.Elem())
	case *types.Signature:
		return "func" + signatureText(t)
	case *types.Struct:
		fields := make([]string, 0, t.NumFields())
		for i := 0; i < t.NumFields(); i++ {
			field := t.Field(i)
			text := typeText(field.Type())
			if !field.Embedded() {
				text = field.Name() + " " + text
			}
			if tag := t.Tag(i); tag != "" {
				encoded, _ := json.Marshal(tag)
				text += " " + string(encoded)
			}
			fields = append(fields, text)
		}
		if len(fields) == 0 {
			return "struct {}"
		}
		return "struct { " + strings.Join(fields, "; ") + " }"
	case *types.Interface:
		parts := []string{}
		for i := 0; i < t.NumExplicitMethods(); i++ {
			method := t.ExplicitMethod(i)
			parts = append(parts, method.Name()+signatureText(method.Type().(*types.Signature)))
		}
		for i := 0; i < t.NumEmbeddeds(); i++ {
			parts = append(parts, typeText(t.EmbeddedType(i)))
		}
		if len(parts) == 0 {
			return "any"
		}
		return "interface { " + strings.Join(parts, "; ") + " }"
	case *types.Union:
		terms := make([]string, 0, t.Len())
		for i := 0; i < t.Len(); i++ {
			term := t.Term(i)
			text := typeText(term.Type())
			if term.Tilde() {
				text = "~" + text
			}
			terms = append(terms, text)
		}
		return strings.Join(terms, " | ")
	case *types.TypeParam:
		return t.Obj().Name()
	case *types.Tuple:
		return tupleText(t, false)
	}
	return t.String()
}

func typeArguments(args *types.TypeList) string {
	if args == nil || args.Len() == 0 {
		return ""
	}
	parts := make([]string, 0, args.Len())
	for i := 0; i < args.Len(); i++ {
		parts = append(parts, typeText(args.At(i)))
	}
	return "[" + strings.Join(parts, ", ") + "]"
}

func tupleText(tuple *types.Tuple, variadic bool) string {
	parts := make([]string, 0, tuple.Len())
	for i := 0; i < tuple.Len(); i++ {
		text := typeText(tuple.At(i).Type())
		if variadic && i == tuple.Len()-1 {
			text = "..." + strings.TrimPrefix(text, "[]")
		}
		parts = append(parts, text)
	}
	return strings.Join(parts, ", ")
}

func signatureText(signature *types.Signature) string {
	text := "(" + tupleText(signature.Params(), signature.Variadic()) + ")"
	switch signature.Results().Len() {
	case 0:
	case 1:
		text += " " + typeText(signature.Results().At(0).Type())
	default:
		text += " (" + tupleText(signature.Results(), false) + ")"
	}
	return text
}

func signatureSlots(path string, signature *types.Signature) {
	add(path, "", "")
	parameters := signature.Params()
	index := 0
	for i := 0; i < parameters.Len(); i++ {
		parameter := parameters.At(i)
		text := typeText(parameter.Type())
		if signature.Variadic() && i == parameters.Len()-1 {
			text = "..." + strings.TrimPrefix(text, "[]")
		}
		if name := parameter.Name(); name != "" && name != "_" {
			add(path, fmt.Sprintf("parameters[%d].name", index), name)
		}
		add(path, fmt.Sprintf("parameters[%d].type", index), text)
		index++
	}
	results := signature.Results()
	switch results.Len() {
	case 0:
	case 1:
		add(path, "returns", typeText(results.At(0).Type()))
	default:
		add(path, "returns", "("+tupleText(results, false)+")")
	}
	typeParameters(path, signature.TypeParams())
}

func typeParameters(path string, parameters *types.TypeParamList) {
	if parameters == nil {
		return
	}
	for i := 0; i < parameters.Len(); i++ {
		parameter := parameters.At(i)
		add(path, fmt.Sprintf("type-parameters[%d].name", i), parameter.Obj().Name())
		add(path, fmt.Sprintf("type-parameters[%d].kind", i), "type")
		add(path, fmt.Sprintf("type-parameters[%d].bounds", i), constraintText(parameter.Constraint()))
	}
}

func constraintText(t types.Type) string {
	if inner, ok := t.(*types.Interface); ok && inner.NumExplicitMethods() == 0 && inner.NumEmbeddeds() == 1 {
		return typeText(inner.EmbeddedType(0))
	}
	return typeText(t)
}

func constantValue(value constant.Value) string {
	switch value.Kind() {
	case constant.String:
		encoded, _ := json.Marshal(constant.StringVal(value))
		return string(encoded)
	case constant.Bool:
		return fmt.Sprint(constant.BoolVal(value))
	}
	return normalizedNumber(value)
}

// bigFloatExponent is the binary exponent from which a float keeps go/constant's big
// float form: 0x1p4095 prints 0x.8p+4096, and 0x1p4094 prints an integer.
const bigFloatExponent = 4096

// normalizedNumber gives each numeric value one spelling. ExactString prints the same
// value as 1 or as 0x.8p+1, by how the constant expression computed it. The ratio is
// the exact value: constant.ToInt rounds a nearly integral float to an integer, which
// gives an equal value two spellings.
func normalizedNumber(value constant.Value) string {
	if value.Kind() == constant.Complex {
		return fmt.Sprintf("(%s + %si)", normalizedNumber(constant.Real(value)), normalizedNumber(constant.Imag(value)))
	}
	float, isFloat := constant.Val(value).(*big.Float)
	if !isFloat {
		return value.ExactString()
	}
	if exponent := float.MantExp(nil); exponent <= -bigFloatExponent || exponent >= bigFloatExponent {
		return value.ExactString()
	}
	ratio, _ := float.Rat(nil)
	return ratio.RatString()
}

func structSlots(path string, shape *types.Struct) {
	for i := 0; i < shape.NumFields(); i++ {
		field := shape.Field(i)
		child := path + "." + field.Name()
		add(child, "type", typeText(field.Type()))
		if tag := shape.Tag(i); tag != "" {
			encoded, _ := json.Marshal(tag)
			add(child, "tag", string(encoded))
		}
	}
}

func typeSlots(path string, object *types.TypeName) {
	if alias, ok := object.Type().(*types.Alias); ok {
		add(path, "type", typeText(alias.Rhs()))
		typeParameters(path, alias.TypeParams())
		return
	}
	named, ok := object.Type().(*types.Named)
	if !ok {
		return
	}
	add(path, "underlying", typeText(named.Underlying()))
	typeParameters(path, named.TypeParams())
	if shape, ok := named.Underlying().(*types.Struct); ok {
		structSlots(path, shape)
	}
	if shape, ok := named.Underlying().(*types.Interface); ok {
		for i := 0; i < shape.NumExplicitMethods(); i++ {
			method := shape.ExplicitMethod(i)
			signatureSlots(path+"."+method.Name(), method.Type().(*types.Signature))
		}
	}
	for i := 0; i < named.NumMethods(); i++ {
		method := named.Method(i)
		child := path + "." + method.Name()
		signature := method.Type().(*types.Signature)
		signatureSlots(child, signature)
		add(child, "receiver", typeText(signature.Recv().Type()))
	}
}

// project type-checks the project's packages from source, each variant once, so
// that a package's types are one set of objects in every package that imports
// that variant.
type project struct {
	fileset  *token.FileSet
	fallback types.Importer
	units    map[string]listed
	checked  map[string]*types.Package
	building map[string]bool
}

// importing resolves one package's imports to the variants its binary uses.
type importing struct {
	project *project
	unit    listed
}

func (i importing) Import(path string) (*types.Package, error) {
	if variant, ok := i.unit.ImportMap[path]; ok {
		path = variant
	}
	// Every package outside the standard library, the project's and its
	// dependencies', loads through one cache, so a type has one identity however it
	// is reached. The standard library imports nothing outside itself, so the
	// source importer's own cache keeps its identities.
	if _, ok := i.project.units[path]; ok {
		return i.project.load(path)
	}
	return i.project.fallback.Import(path)
}

// load type-checks the package `go list` lists as `id`, with the files that
// variant compiles: a test variant's `GoFiles` hold its `_test.go` files.
func (p *project) load(id string) (*types.Package, error) {
	if checked, ok := p.checked[id]; ok {
		return checked, nil
	}
	if p.building[id] {
		return nil, fmt.Errorf("import cycle through %s", id)
	}
	p.building[id] = true
	unit := p.units[id]
	files := []*ast.File{}
	for _, name := range append(append([]string{}, unit.GoFiles...), unit.CgoFiles...) {
		file, err := parser.ParseFile(p.fileset, filepath.Join(unit.Dir, name), nil, 0)
		if err != nil {
			return nil, err
		}
		files = append(files, file)
	}
	// A dependency module's function bodies state nothing the report reads: only
	// its package-level declarations' types reach a project slot.
	external := unit.Module == nil || !unit.Module.Main
	config := types.Config{Importer: importing{p, unit}, FakeImportC: true, IgnoreFuncBodies: external}
	checked, err := config.Check(packagePath(id), p.fileset, files, nil)
	if err != nil {
		return nil, err
	}
	p.checked[id] = checked
	return checked, nil
}

// packagePath is a variant's import path, without its ` [p.test]` binary.
func packagePath(id string) string {
	if i := strings.Index(id, " ["); i >= 0 {
		return id[:i]
	}
	return id
}

// state states the slots of a package's members.
func state(checked *types.Package) {
	scope := checked.Scope()
	for _, name := range scope.Names() {
		object := scope.Lookup(name)
		path := checked.Path() + "." + name
		switch object := object.(type) {
		case *types.Const:
			add(path, "type", typeText(object.Type()))
			// A cgo constant under FakeImportC has an unknown value: unstated.
			if object.Val().Kind() == constant.Unknown {
				add(path, "value", unresolved)
			} else {
				add(path, "value", constantValue(object.Val()))
			}
		case *types.Var:
			add(path, "type", typeText(object.Type()))
		case *types.Func:
			signatureSlots(path, object.Type().(*types.Signature))
		case *types.TypeName:
			typeSlots(path, object)
		}
	}
}

// definedTypes lists a package's package-level defined types, the only types the
// profile records.
func definedTypes(checked *types.Package) []*types.Named {
	found := []*types.Named{}
	scope := checked.Scope()
	for _, name := range scope.Names() {
		object, ok := scope.Lookup(name).(*types.TypeName)
		if !ok || object.IsAlias() {
			continue
		}
		if named, ok := object.Type().(*types.Named); ok {
			found = append(found, named)
		}
	}
	return found
}

// bases states each reported type's `bases`: the interfaces an interface embeds,
// and every project interface whose method set a non-interface type `T` or `*T`
// implements. go/types cannot decide this for an uninstantiated generic type, so a
// type that is generic, or that could implement a generic interface, has its
// `bases` unstated.
func bases(universe []*types.Package) {
	interfaces := []*types.Named{}
	for _, checked := range universe {
		for _, named := range definedTypes(checked) {
			if types.IsInterface(named) {
				interfaces = append(interfaces, named)
			}
		}
	}
	for _, checked := range universe {
		// A command's members are no consumer's, but its interfaces are the project's.
		if checked.Name() == "main" {
			continue
		}
		for _, named := range definedTypes(checked) {
			path := checked.Path() + "." + named.Obj().Name()
			if shape, ok := named.Underlying().(*types.Interface); ok {
				for i := 0; i < shape.NumEmbeddeds(); i++ {
					// An embedded name that denotes an interface; a union, a type
					// term and an interface literal name no base.
					embedded := shape.EmbeddedType(i)
					switch embedded.(type) {
					case *types.Named, *types.Alias:
					default:
						continue
					}
					if !types.IsInterface(embedded) {
						continue
					}
					base(path, typeText(embedded))
				}
				continue
			}
			if named.TypeParams().Len() > 0 {
				unstate(path, "bases")
				continue
			}
			pointer := types.NewPointer(named)
			for _, candidate := range interfaces {
				if candidate.TypeParams().Len() > 0 {
					if couldImplement(named, candidate) {
						unstate(path, "bases")
						break
					}
					continue
				}
				shape := candidate.Underlying().(*types.Interface)
				if types.Implements(named, shape) || types.Implements(pointer, shape) {
					base(path, typeText(candidate))
				}
			}
		}
	}
}

// base states one `bases` entry, unless another graph left the member's `bases`
// unstated.
func base(path, text string) {
	if found := members[path]; found != nil {
		for _, slot := range found.Unstated {
			if slot == "bases" {
				return
			}
		}
	}
	add(path, "bases["+text+"].type", text)
}

// couldImplement reports whether `*T` has a method of each name a generic
// interface's method set holds.
func couldImplement(named *types.Named, generic *types.Named) bool {
	methods := types.NewMethodSet(types.NewPointer(named))
	shape := generic.Underlying().(*types.Interface)
	for i := 0; i < shape.NumMethods(); i++ {
		method := shape.Method(i)
		if methods.Lookup(method.Pkg(), method.Name()) == nil {
			return false
		}
	}
	return true
}

func unstate(path, prefix string) {
	add(path, "", "")
	found := members[path]
	for _, slot := range found.Unstated {
		if slot == prefix {
			return
		}
	}
	found.Unstated = append(found.Unstated, prefix)
	for slot := range found.Slots {
		if strings.HasPrefix(slot, prefix) {
			delete(found.Slots, slot)
		}
	}
}

// noop, set in the environment of the build `compile` runs, makes this program
// exit 0 at once: it is the `-exec` of `go test`, so no test binary runs.
const noop = "REPORT_GO_NOOP"

// compile runs the context's real build: in mode build `go build -o <null>`,
// which compiles every selected package, commands and libraries, and discards
// what it links. In mode test `go test -exec <noop>`, which compiles and links
// every test binary, each in the go command's own work directory, so binaries of
// one name cannot collide, and compiles the packages without tests, as `go test`
// does; the no-op runs in place of each binary, so no `TestMain` or `init` runs,
// and a cross-compiled binary is never executed. It runs under the environment
// `go list` ran under, with its module flag, tags and patterns, and writes nothing
// into the project. A failed build exits with its output, before any report is
// written.
func compile(testing bool, module []string, tags, patterns string) {
	arguments := []string{"build", "-o", os.DevNull}
	environment := os.Environ()
	if testing {
		self, err := os.Executable()
		if err != nil {
			fmt.Fprintln(os.Stderr, "report-go: cannot name this program as go test's -exec:", err)
			exit(2)
		}
		// The go command splits -exec on spaces, outside quotes.
		quoted := "'" + self + "'"
		if strings.Contains(self, "'") {
			quoted = `"` + self + `"`
		}
		// vet is no part of the build, and its findings no compile error.
		arguments = []string{"test", "-count=1", "-run", "^$", "-vet=off", "-exec", quoted}
		environment = append(environment, noop+"=1")
	}
	arguments = append(arguments, module...)
	if tags != "" {
		arguments = append(arguments, "-tags", tags)
	}
	arguments = append(arguments, strings.Fields(patterns)...)
	build := exec.Command("go", arguments...)
	build.Env = environment
	built, err := build.CombinedOutput()
	if err != nil {
		os.Stderr.WriteString(named(string(built)))
		fmt.Fprintf(os.Stderr, "report-go: go %s failed (%v): the context does not build, and no report is written\n", strings.Join(arguments[:len(arguments)-len(strings.Fields(patterns))], " "), err)
		exit(2)
	}
}

// moduleFlag returns the -mod flag every go command of the report runs with. A
// dataset describes the code as checked in, and the report writes nothing into
// the project: `-mod=mod`, from GOFLAGS or the go env file, would add a missing
// requirement to `go.mod` and `go.sum`, so it exits. Otherwise, unless a vendor
// directory is in use, it is `-mod=readonly`, so a missing requirement fails the
// load rather than being added; `-mod=vendor`, set or implied, stays as it is.
//
// `-mod=readonly` does not stop a workspace's writes: the go command records a
// checksum it lacks in `go.work.sum`, beside `go.work`. In workspace mode, unless
// the vendor directory is in use, every later go command of the report runs
// against a copy of `go.work` outside the project, from `copyWorkspace`. Vendor
// mode reads no module cache and records no checksum, and its `vendor/modules.txt`
// must lie beside `go.work`, so it keeps the project's own.
func moduleFlag() []string {
	queried, err := exec.Command("go", "env", "-json", "GOFLAGS", "GOMOD", "GOWORK").Output()
	if err != nil {
		fmt.Fprintln(os.Stderr, "report-go: go env:", err)
		exit(2)
	}
	env := map[string]string{}
	if err := json.Unmarshal(queried, &env); err != nil {
		fmt.Fprintln(os.Stderr, "report-go: go env:", err)
		exit(2)
	}
	// The go command takes the last of repeated flags.
	mode := ""
	for _, flag := range strings.Fields(env["GOFLAGS"]) {
		for _, prefix := range []string{"-mod=", "--mod="} {
			if strings.HasPrefix(flag, prefix) {
				mode = strings.TrimPrefix(flag, prefix)
			}
		}
	}
	if mode == "mod" {
		fmt.Fprintln(os.Stderr, "report-go: GOFLAGS sets -mod=mod, which would modify the project's go.mod and go.sum, so the context cannot be checked as written; unset it, or set -mod=readonly")
		exit(2)
	}
	// `go env GOWORK` is `off` when GOWORK disables workspace mode.
	work := env["GOWORK"]
	if work == "off" {
		work = ""
	}
	root := ""
	if work != "" {
		root = filepath.Dir(work)
	} else if env["GOMOD"] != "" && env["GOMOD"] != os.DevNull {
		root = filepath.Dir(env["GOMOD"])
	}
	vendored := mode == "vendor"
	if mode == "" && root != "" {
		info, err := os.Stat(filepath.Join(root, "vendor"))
		vendored = err == nil && info.IsDir()
	}
	if work != "" && !vendored {
		copyWorkspace(work)
	}
	if mode != "" || root == "" || vendored {
		// readonly, vendor, or a value the go command itself rejects.
		return nil
	}
	return []string{"-mod=readonly"}
}

// workFile is `go work edit -json`'s form of a `go.work`.
type workFile struct {
	Go        string
	Toolchain string
	Godebug   []struct{ Key, Value string }
	Use       []struct{ DiskPath string }
	Replace   []struct {
		Old struct{ Path, Version string }
		New struct{ Path, Version string }
	}
}

// temporary is the directory of the workspace copy, removed at exit.
var temporary string

// rewrites name the workspace copy's files, then the project's, so that a
// message names the project's own files.
var rewrites []string

// exit removes the workspace copy, and exits.
func exit(code int) {
	if temporary != "" {
		os.RemoveAll(temporary)
	}
	os.Exit(code)
}

// named rewrites a go command's message to name the project's files.
func named(message string) string {
	return strings.NewReplacer(rewrites...).Replace(message)
}

// copyWorkspace writes `go.work` into a temporary directory, with its `go`,
// `toolchain` and `godebug` lines and its replaces, its `use` directories and
// local replacements made absolute, so that each module keeps its path and
// directory, and its `go.work.sum` beside it, and points GOWORK at the copy. A
// checksum the go command records lands in the copy, which exit removes.
func copyWorkspace(work string) {
	edited, err := exec.Command("go", "work", "edit", "-json", work).Output()
	if err != nil {
		fmt.Fprintln(os.Stderr, "report-go: go work edit -json", work+":", err)
		exit(2)
	}
	var parsed workFile
	if err := json.Unmarshal(edited, &parsed); err != nil {
		fmt.Fprintln(os.Stderr, "report-go: go work edit -json", work+":", err)
		exit(2)
	}
	root := filepath.Dir(work)
	absolute := func(path string) string {
		if filepath.IsAbs(path) {
			return path
		}
		return filepath.Join(root, path)
	}
	text := &strings.Builder{}
	if parsed.Go != "" {
		fmt.Fprintf(text, "go %s\n", parsed.Go)
	}
	if parsed.Toolchain != "" {
		fmt.Fprintf(text, "toolchain %s\n", parsed.Toolchain)
	}
	for _, setting := range parsed.Godebug {
		fmt.Fprintf(text, "godebug %s=%s\n", setting.Key, setting.Value)
	}
	for _, use := range parsed.Use {
		fmt.Fprintf(text, "use %s\n", strconv.Quote(absolute(use.DiskPath)))
	}
	for _, replace := range parsed.Replace {
		old := strconv.Quote(replace.Old.Path)
		if replace.Old.Version != "" {
			old += " " + replace.Old.Version
		}
		// A replacement without a version is a directory.
		replacement := strconv.Quote(replace.New.Path)
		if replace.New.Version == "" {
			replacement = strconv.Quote(absolute(replace.New.Path))
		} else {
			replacement += " " + replace.New.Version
		}
		fmt.Fprintf(text, "replace %s => %s\n", old, replacement)
	}
	temporary, err = os.MkdirTemp("", "report-go-work-")
	if err != nil {
		fmt.Fprintln(os.Stderr, "report-go: cannot create the workspace copy:", err)
		exit(2)
	}
	copied := filepath.Join(temporary, "go.work")
	if err := os.WriteFile(copied, []byte(text.String()), 0o644); err != nil {
		fmt.Fprintln(os.Stderr, "report-go: cannot write the workspace copy:", err)
		exit(2)
	}
	if sums, err := os.ReadFile(work + ".sum"); err == nil {
		if err := os.WriteFile(copied+".sum", sums, 0o644); err != nil {
			fmt.Fprintln(os.Stderr, "report-go: cannot write the workspace copy:", err)
			exit(2)
		}
	} else if !os.IsNotExist(err) {
		fmt.Fprintln(os.Stderr, "report-go: cannot read", work+".sum:", err)
		exit(2)
	}
	// Each go command the report runs, `go test`'s included, inherits GOWORK.
	os.Setenv("GOWORK", copied)
	rewrites = []string{copied, work, temporary, root}
	// A signal stops the go command it interrupts too, and the report removes
	// the copy.
	signals := make(chan os.Signal, 1)
	signal.Notify(signals, os.Interrupt, syscall.SIGTERM)
	go func() {
		<-signals
		exit(2)
	}()
}

func main() {
	if os.Getenv(noop) == "1" {
		os.Exit(0)
	}
	context := flag.String("context", "", "the dataset's context id")
	out := flag.String("out", "", "the report file")
	tags := flag.String("tags", "", "build tags")
	mode := flag.String("mode", "build", "the context's mode: build or test")
	patterns := flag.String("patterns", "./...", "the context's package patterns, separated by spaces")
	flag.Parse()
	if *mode != "build" && *mode != "test" {
		fmt.Fprintln(os.Stderr, "report-go: -mode is build or test, not", *mode)
		exit(2)
	}
	if absolute, err := filepath.Abs(*out); err == nil {
		root, _ := os.Getwd()
		if strings.HasPrefix(absolute, root+string(filepath.Separator)) {
			fmt.Fprintln(os.Stderr, "report-go: -out", absolute, "lies inside the project tree")
			exit(2)
		}
		os.Remove(absolute)
	}
	testing := *mode == "test"
	module := moduleFlag()
	arguments := append([]string{"list", "-json", "-e", "-deps"}, module...)
	if testing {
		arguments = append(arguments, "-test")
	}
	if *tags != "" {
		arguments = append(arguments, "-tags", *tags)
		// The source importer reads dependencies under the same tags.
		build.Default.BuildTags = strings.Split(*tags, ",")
	}
	arguments = append(arguments, strings.Fields(*patterns)...)
	listing, err := exec.Command("go", arguments...).Output()
	if err != nil {
		if failed, ok := err.(*exec.ExitError); ok {
			os.Stderr.WriteString(named(string(failed.Stderr)))
		}
		fmt.Fprintln(os.Stderr, "go list:", err)
		exit(2)
	}
	decoder := json.NewDecoder(strings.NewReader(string(listing)))
	fileset := token.NewFileSet()
	checker := &project{
		fileset:  fileset,
		fallback: importer.ForCompiler(fileset, "source", nil),
		units:    map[string]listed{},
		checked:  map[string]*types.Package{},
		building: map[string]bool{},
	}
	order := []listed{}
	inProject := map[string]bool{}
	problems := []string{}
	for decoder.More() {
		var unit listed
		if err := decoder.Decode(&unit); err != nil {
			fmt.Fprintln(os.Stderr, "go list:", err)
			exit(2)
		}
		if unit.Error != nil {
			problems = append(problems, unit.ImportPath+": "+named(unit.Error.Err))
		}
		for _, dependency := range unit.DepsErrors {
			problems = append(problems, unit.ImportPath+": "+named(dependency.Err))
		}
		if unit.Standard {
			continue
		}
		checker.units[unit.ImportPath] = unit
		// Only the main modules' packages are the project's, and reported.
		if unit.Module != nil && unit.Module.Main {
			inProject[unit.ImportPath] = true
			order = append(order, unit)
		}
	}
	// A report with no package states nothing, and must not pass for one that
	// checked the project. From a `go.work` root, `./...` matches no module: pass
	// the context's patterns.
	found := false
	for _, unit := range order {
		found = found || len(unit.GoFiles)+len(unit.CgoFiles) > 0
	}
	if !found {
		fmt.Fprintln(os.Stderr, "report-go: the patterns", *patterns, "match no package of a main module; run from the context's module-root with its patterns")
		for _, problem := range problems {
			fmt.Fprintln(os.Stderr, "go list:", problem)
		}
		exit(2)
	}
	// A pattern or package the go command cannot load fails the build, so a
	// report of the rest would certify a context that does not build.
	if len(problems) > 0 {
		for _, problem := range problems {
			fmt.Fprintln(os.Stderr, "go list:", problem)
		}
		fmt.Fprintln(os.Stderr, "report-go: the go command cannot load every package of the context")
		exit(2)
	}
	// A package that fails to type-check stops the report: nothing it would
	// state was checked.
	load := func(id string) *types.Package {
		checked, err := checker.load(id)
		if err != nil {
			fmt.Fprintln(os.Stderr, "report-go:", id+":", err)
			exit(2)
		}
		return checked
	}
	// A test binary is the package `p.test` that `go list -test` adds for `p`.
	binaries := []listed{}
	built := []*types.Package{}
	for _, unit := range order {
		if strings.Contains(unit.ImportPath, " [") {
			continue
		}
		if _, isBinary := checker.units[strings.TrimSuffix(unit.ImportPath, ".test")]; testing && isBinary && strings.HasSuffix(unit.ImportPath, ".test") && unit.Name == "main" {
			binaries = append(binaries, unit)
			continue
		}
		if len(unit.GoFiles)+len(unit.CgoFiles) == 0 {
			continue
		}
		checked := load(unit.ImportPath)
		built = append(built, checked)
		// A command's members are no consumer's, but its interfaces are the
		// project's. In mode test, a package with a test variant states that.
		_, tested := checker.units[unit.ImportPath+" ["+unit.ImportPath+".test]"]
		if unit.Name != "main" && !tested {
			state(checked)
		}
	}
	// The build graph: every package as a build compiles it.
	bases(built)
	// One graph per test binary, of exactly the variants it links. A relation in
	// the context is one in any graph.
	for _, binary := range binaries {
		tested := strings.TrimSuffix(binary.ImportPath, ".test")
		suffix := " [" + binary.ImportPath + "]"
		universe := []*types.Package{}
		for _, id := range binary.Deps {
			if !inProject[id] {
				continue
			}
			checked := load(id)
			universe = append(universe, checked)
			if checked.Name() != "main" && (id == tested+suffix || id == tested+"_test"+suffix) {
				state(checked)
			}
		}
		bases(universe)
	}
	// Dependencies are checked without function bodies, and go/types is not the
	// compiler: a report must not certify a context its real build rejects.
	compile(testing, module, *tags, *patterns)
	paths := make([]string, 0, len(members))
	for path := range members {
		paths = append(paths, path)
	}
	sort.Strings(paths)
	ordered := make([]*member, 0, len(paths))
	for _, path := range paths {
		ordered = append(ordered, members[path])
	}
	report := map[string]any{
		"format":   "semantic-oracle-compiler-report/1",
		"language": "go",
		"context":  *context,
		"tool":     map[string]string{"name": "go/types", "version": runtime.Version()},
		"release":  runtime.Version(),
		"mode":     *mode,
		"covers": []string{
			"parameters[].name", "parameters[].type", "returns", "receiver", "underlying",
			"type", "tag", "value", "type-parameters[].name", "type-parameters[].kind",
			"type-parameters[].bounds", "bases[].type",
		},
		"members": ordered,
	}
	encoded, _ := json.MarshalIndent(report, "", " ")
	if err := os.WriteFile(*out, encoded, 0o644); err != nil {
		fmt.Fprintln(os.Stderr, err)
		exit(2)
	}
	fmt.Printf("{\"members\": %d}\n", len(ordered))
	exit(0)
}

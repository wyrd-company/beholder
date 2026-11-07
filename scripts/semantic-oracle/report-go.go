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
// external `_test` package, as `go test` compiles them.
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
	"path/filepath"
	"runtime"
	"sort"
	"strings"
)

type listed struct {
	Dir          string
	ImportPath   string
	Name         string
	GoFiles      []string
	CgoFiles     []string
	TestGoFiles  []string
	XTestGoFiles []string
	Deps         []string
}

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
	if slot != "" {
		found.Slots[slot] = value
	}
}

func typeText(t types.Type) string {
	switch t := t.(type) {
	case *types.Basic:
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

// project type-checks the project's packages from source, so that one package's
// types are the same objects in every package that imports it.
type project struct {
	fileset  *token.FileSet
	fallback types.Importer
	units    map[string]listed
	plain    map[string]*types.Package
	failed   map[string]error
}

func (p *project) Import(path string) (*types.Package, error) {
	if unit, ok := p.units[path]; ok {
		return p.plainPackage(unit)
	}
	return p.fallback.Import(path)
}

// plainPackage is a package as a build compiles it, without its tests.
func (p *project) plainPackage(unit listed) (*types.Package, error) {
	if checked, ok := p.plain[unit.ImportPath]; ok {
		return checked, nil
	}
	if err, ok := p.failed[unit.ImportPath]; ok {
		return nil, err
	}
	checked, err := p.check(unit.ImportPath, unit.Dir, p, unit.GoFiles, unit.CgoFiles)
	if err != nil {
		p.failed[unit.ImportPath] = err
		return nil, err
	}
	p.plain[unit.ImportPath] = checked
	return checked, nil
}

// graph is the package graph of one test binary, as `go test` builds it: the
// tested package with its in-package test files, and every project package that
// imports it rebuilt against that variant, so that each package's types are one
// set of objects within the graph. Other packages are their build variants.
type graph struct {
	project  *project
	tested   string
	variant  *types.Package
	rebuilt  map[string]*types.Package
	building map[string]bool
}

func (g *graph) Import(path string) (*types.Package, error) {
	if path == g.tested {
		return g.variant, nil
	}
	unit, ok := g.project.units[path]
	if !ok || !dependsOn(unit, g.tested) {
		return g.project.Import(path)
	}
	if checked, ok := g.rebuilt[path]; ok {
		return checked, nil
	}
	if g.building[path] {
		return nil, fmt.Errorf("import cycle through %s", path)
	}
	g.building[path] = true
	checked, err := g.project.check(unit.ImportPath, unit.Dir, g, unit.GoFiles, unit.CgoFiles)
	if err != nil {
		return nil, err
	}
	g.rebuilt[path] = checked
	return checked, nil
}

func dependsOn(unit listed, path string) bool {
	for _, dependency := range unit.Deps {
		if dependency == path {
			return true
		}
	}
	return false
}

func (p *project) check(path, dir string, imports types.Importer, groups ...[]string) (*types.Package, error) {
	files := []*ast.File{}
	for _, group := range groups {
		for _, name := range group {
			file, err := parser.ParseFile(p.fileset, filepath.Join(dir, name), nil, 0)
			if err != nil {
				return nil, err
			}
			files = append(files, file)
		}
	}
	config := types.Config{Importer: imports, FakeImportC: true}
	return config.Check(path, p.fileset, files, nil)
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
			add(path, "value", constantValue(object.Val()))
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

func main() {
	context := flag.String("context", "", "the dataset's context id")
	out := flag.String("out", "", "the report file")
	tags := flag.String("tags", "", "build tags")
	mode := flag.String("mode", "build", "the context's mode: build or test")
	flag.Parse()
	if *mode != "build" && *mode != "test" {
		fmt.Fprintln(os.Stderr, "report-go: -mode is build or test, not", *mode)
		os.Exit(2)
	}
	if absolute, err := filepath.Abs(*out); err == nil {
		root, _ := os.Getwd()
		if strings.HasPrefix(absolute, root+string(filepath.Separator)) {
			fmt.Fprintln(os.Stderr, "report-go: -out", absolute, "lies inside the project tree")
			os.Exit(2)
		}
		os.Remove(absolute)
	}
	arguments := []string{"list", "-json", "-e"}
	if *tags != "" {
		arguments = append(arguments, "-tags", *tags)
		// The source importer reads dependencies under the same tags.
		build.Default.BuildTags = strings.Split(*tags, ",")
	}
	arguments = append(arguments, "./...")
	listing, err := exec.Command("go", arguments...).Output()
	if err != nil {
		fmt.Fprintln(os.Stderr, "go list:", err)
		os.Exit(2)
	}
	decoder := json.NewDecoder(strings.NewReader(string(listing)))
	fileset := token.NewFileSet()
	checker := &project{
		fileset:  fileset,
		fallback: importer.ForCompiler(fileset, "source", nil),
		units:    map[string]listed{},
		plain:    map[string]*types.Package{},
		failed:   map[string]error{},
	}
	order := []listed{}
	for decoder.More() {
		var unit listed
		if err := decoder.Decode(&unit); err != nil {
			fmt.Fprintln(os.Stderr, "go list:", err)
			os.Exit(2)
		}
		checker.units[unit.ImportPath] = unit
		order = append(order, unit)
	}
	// A package that fails to type-check stops the report: nothing it would
	// state was checked.
	stop := func(path string, err error) {
		fmt.Fprintln(os.Stderr, "report-go:", path+":", err)
		os.Exit(2)
	}
	testing := *mode == "test"
	// The build graph: every package as a build compiles it.
	built := []*types.Package{}
	for _, unit := range order {
		if len(unit.GoFiles)+len(unit.CgoFiles) == 0 {
			continue
		}
		checked, err := checker.plainPackage(unit)
		if err != nil {
			stop(unit.ImportPath, err)
		}
		built = append(built, checked)
		if unit.Name != "main" && !(testing && len(unit.TestGoFiles) > 0) {
			state(checked)
		}
	}
	bases(built)
	if testing {
		// One graph per test binary. A relation in the context is one in any of them.
		for _, unit := range order {
			if unit.Name == "main" || len(unit.TestGoFiles)+len(unit.XTestGoFiles) == 0 {
				continue
			}
			tested := &graph{project: checker, tested: unit.ImportPath, rebuilt: map[string]*types.Package{}, building: map[string]bool{}}
			if len(unit.TestGoFiles) > 0 {
				tested.variant, err = checker.check(unit.ImportPath, unit.Dir, checker, unit.GoFiles, unit.CgoFiles, unit.TestGoFiles)
				if err != nil {
					stop(unit.ImportPath, err)
				}
				state(tested.variant)
			} else if len(unit.GoFiles)+len(unit.CgoFiles) > 0 {
				// With no in-package test file, the tested package is its build variant.
				tested.variant = checker.plain[unit.ImportPath]
			}
			universe := []*types.Package{}
			for _, other := range order {
				if len(other.GoFiles)+len(other.CgoFiles) == 0 && other.ImportPath != unit.ImportPath {
					continue
				}
				checked, err := tested.Import(other.ImportPath)
				if err != nil {
					stop(other.ImportPath, err)
				}
				if checked != nil {
					universe = append(universe, checked)
				}
			}
			if len(unit.XTestGoFiles) > 0 {
				external, err := checker.check(unit.ImportPath+"_test", unit.Dir, tested, unit.XTestGoFiles)
				if err != nil {
					stop(unit.ImportPath+"_test", err)
				}
				state(external)
				universe = append(universe, external)
			}
			bases(universe)
		}
	}
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
		os.Exit(2)
	}
	fmt.Printf("{\"members\": %d}\n", len(ordered))
}

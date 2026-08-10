package toolkit

import (
	"reflect"

	"example.invalid/collections-toolkit/catalog"
)

// Namer is implemented implicitly by both concrete values below.
type Namer interface {
	Name() string
}

type NamedText string

func (text NamedText) Name() string {
	return string(text)
}

type pointerName struct {
	value string
}

func (name *pointerName) Name() string {
	if name == nil {
		return "nil"
	}
	return name.value
}

var _ Namer = NamedText("")
var _ Namer = (*pointerName)(nil)

// InterfaceDispatch contrasts static concrete and dynamic interface calls.
func InterfaceDispatch() (string, string) {
	concrete := NamedText("north")
	var dynamic Namer = concrete
	return concrete.Name(), dynamic.Name()
}

// TypedNil inspects nil interface, typed nil dynamic values, assertion, and
// reflection before calling a nil-safe receiver method.
func TypedNil() (bool, bool, bool, string) {
	var pointer *pointerName
	var dynamic Namer = pointer
	_, asserted := dynamic.(*pointerName)
	reflectedNil := reflect.ValueOf(dynamic).IsNil()
	return dynamic == nil, asserted, reflectedNil, dynamic.Name()
}

// NamedAndLabelled embeds an interface and adds a second method.
type NamedAndLabelled interface {
	Namer
	Label() string
}

type namedLabelled struct {
	name  string
	label string
}

func (value namedLabelled) Name() string  { return value.name }
func (value namedLabelled) Label() string { return value.label }

var _ NamedAndLabelled = namedLabelled{}

// OverlapLeft and OverlapRight have the same method identity, so their embedded
// overlap is merged by Combined. SealedFromCatalog cannot be implemented here.
type OverlapLeft interface {
	Name() string
}

type OverlapRight interface {
	Name() string
}

type Combined interface {
	OverlapLeft
	OverlapRight
}

func EmbeddedInterface(value namedLabelled) string {
	var combined Combined = value
	return combined.Name()
}

// AssertionAndSwitch demonstrates one- and two-result assertions, interface
// assertions, concrete cases, interface cases, and nil.
func AssertionAndSwitch(value any) (string, bool, bool) {
	text, textOK := value.(string)
	_, interfaceOK := value.(Namer)
	label := "default"
	switch typed := value.(type) {
	case string:
		label = typed
	case NamedText:
		label = typed.Name()
	case Namer:
		label = typed.Name()
	case nil:
		label = "nil"
	}
	return label + text, textOK, interfaceOK
}

func CrossPackageInterface() string {
	var value catalog.Reader = catalog.NewPublicRecord("north", 2)
	return value.Read()
}

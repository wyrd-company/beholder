package reflectkit

import (
	"fmt"
	"reflect"
)

// Subject contains both ordinary and hidden members for reflection examples.
type Subject struct {
	Name   string
	Count  int
	secret string
}

// Describe is resolved by MethodByName in Inspect.
func (s Subject) Describe() string {
	return fmt.Sprintf("%s:%d", s.Name, s.Count)
}

// Fail returns an error so callers can observe a method with an error result.
func (s Subject) Fail() (string, error) {
	return "", fmt.Errorf("%s is unavailable", s.Name)
}

// Named is used for a runtime interface-implementation check.
type Named interface {
	Describe() string
}

// Snapshot records the different reflection paths used by the fixture.
type Snapshot struct {
	TypeName           string
	TypeForMatches     bool
	NewValueSettable   bool
	DynamicField       string
	DynamicFieldFound  bool
	DynamicMethod      string
	DynamicMethodFound bool
	ConvertedKind      reflect.Kind
	ImplementsNamed    bool
	InterfaceAssertion bool
	MissingField       bool
	UnexportedSettable bool
	MutationVisible    string
	WrongKindIsString  bool
	MisuseWasRecovered bool
}

// Inspect exercises reflection with names supplied by the caller.
func Inspect(fieldName, methodName string) Snapshot {
	value := reflect.ValueOf(&Subject{Name: "meadow", Count: 3, secret: "hidden"}).Elem()
	typeOfSubject := value.Type()
	typeForSubject := reflect.TypeFor[Subject]()
	newValue := reflect.New(typeForSubject).Elem()

	dynamicField := value.FieldByName(fieldName)
	dynamicMethod := value.MethodByName(methodName)
	converted := reflect.ValueOf(int32(7)).Convert(reflect.TypeOf(int64(0)))
	namedType := reflect.TypeFor[Named]()
	interfaceValue := value.Interface()
	_, interfaceAssertion := interfaceValue.(Named)

	exported := value.FieldByName("Name")
	if exported.CanSet() {
		exported.SetString("updated")
	}
	unexported := value.FieldByName("secret")

	result := Snapshot{
		TypeName:           typeOfSubject.Name(),
		TypeForMatches:     typeOfSubject == typeForSubject,
		NewValueSettable:   newValue.CanSet(),
		DynamicField:       fieldValue(dynamicField),
		DynamicFieldFound:  dynamicField.IsValid(),
		DynamicMethod:      methodName,
		DynamicMethodFound: dynamicMethod.IsValid(),
		ConvertedKind:      converted.Kind(),
		ImplementsNamed:    typeOfSubject.Implements(namedType),
		InterfaceAssertion: interfaceAssertion,
		MissingField:       !value.FieldByName("Missing").IsValid(),
		UnexportedSettable: unexported.CanSet(),
		MutationVisible:    value.FieldByName("Name").String(),
		WrongKindIsString:  reflect.ValueOf(42).Kind() == reflect.String,
	}
	result.MisuseWasRecovered = recoverMisuse()
	return result
}

func fieldValue(value reflect.Value) string {
	if !value.IsValid() {
		return ""
	}
	if value.Kind() == reflect.String {
		return value.String()
	}
	return fmt.Sprint(value.Interface())
}

func recoverMisuse() (recovered bool) {
	defer func() {
		if recover() != nil {
			recovered = true
		}
	}()
	// Elem on a non-pointer is a valid program but a deliberate runtime misuse.
	_ = reflect.ValueOf(42).Elem()
	return false
}

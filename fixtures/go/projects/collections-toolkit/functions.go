package toolkit

import (
	"strings"

	"example.invalid/collections-toolkit/catalog"
)

// BinaryOperation is a named function type with a variadic sibling below.
type BinaryOperation func(int, int) int

type StringOperation func(string) string

func Add(left, right int) int {
	return left + right
}

func Sum(values ...int) int {
	total := 0
	for _, value := range values {
		total += value
	}
	return total
}

func ApplyOperation(operation BinaryOperation, values ...int) (int, int) {
	if len(values) < 2 {
		return 0, 0
	}
	return operation(values[0], values[1]), Sum(values...)
}

func FunctionValues() (int, int, string) {
	operation := BinaryOperation(Add)
	closure := func(value int) int { return value * 2 }
	values := []int{2, 3, 4}
	first, total := ApplyOperation(operation, values...)
	return first, total + closure(1), strings.ToUpper("north")
}

// AdapterFunc is a named function type with a method, making one function
// value satisfy Formatter without a wrapper struct.
type AdapterFunc func(string) string

func (adapter AdapterFunc) Format(value string) string {
	return adapter(value)
}

type Formatter interface {
	Format(string) string
}

func (adapter AdapterFunc) String() string {
	return adapter("value")
}

func FunctionAdapter() string {
	var formatter Formatter = AdapterFunc(strings.ToUpper)
	return formatter.Format("north")
}

// MethodValueRecord owns both value and pointer methods for method-set examples.
type MethodValueRecord struct {
	Name  string
	Count int
}

func (record MethodValueRecord) Label() string {
	return record.Name
}

func (record *MethodValueRecord) Add(amount int) int {
	if record == nil {
		return 0
	}
	record.Count += amount
	return record.Count
}

func MethodValuesAndExpressions() (string, int, string, int) {
	record := MethodValueRecord{Name: "north"}
	valueMethod := record.Label
	pointerMethod := record.Add
	methodExpression := MethodValueRecord.Label
	pointerExpression := (*MethodValueRecord).Add
	return valueMethod(), pointerMethod(2), methodExpression(record), pointerExpression(&record, 3)
}

// GenericMethodRecord is a local defined generic type with a receiver parameter.
type GenericMethodRecord[T any] struct {
	Value T
}

func (record GenericMethodRecord[T]) ValueOf() T {
	return record.Value
}

func GenericMethodValue() int {
	record := GenericMethodRecord[int]{Value: 7}
	return record.ValueOf()
}

// CrossPackageMethodValue gets a method through a generic type defined elsewhere.
func CrossPackageMethodValue() string {
	record := catalog.NewEnvelope("north", "label")
	return record.String()
}

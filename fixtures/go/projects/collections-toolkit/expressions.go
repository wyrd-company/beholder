package toolkit

import (
	"strconv"
	"strings"
)

// CompositeLiterals contains keyed, unkeyed, nested-elision, and generic
// instantiated literals for the standard composite kinds.
func CompositeLiterals() (map[string][]int, []TaggedRecord, GenericBox[int], [2]int) {
	array := [2]int{0: 1, 1: 2}
	slice := []TaggedRecord{{Name: "north"}, {Count: 2}}
	mapping := map[string][]int{
		"north": {1, 2},
		"south": []int{3, 4},
	}
	box := GenericBox[int]{Value: 5}
	return mapping, slice, box, array
}

func NestedCompositeLiterals() *struct {
	Items []map[string]int
} {
	return &struct{ Items []map[string]int }{
		Items: []map[string]int{{"north": 1}, {"south": 2}},
	}
}

// IndexAndSliceForms covers strings, arrays, pointers-to-arrays, slices, maps,
// and two- and three-index slicing.
func IndexAndSliceForms() (byte, int, int, string, int, bool) {
	text := "north"
	array := [4]int{1, 2, 3, 4}
	pointer := &array
	slice := array[:]
	mapValue, present := map[string]int{"north": 1}["north"]
	return text[0], pointer[1], slice[1:3][0], text[1:3], mapValue, present
}

func GenericIndexAndSlice[T ByteString](value T) (any, T, int) {
	return value[0], value[:1], len(value)
}

// SharedSyntax uses call, conversion, instantiation, and built-in forms that
// all share the T(x)-shaped surface syntax.
func SharedSyntax(value int) (int, string, int, []int) {
	type LocalNumber int
	converted := LocalNumber(value)
	called := Add(int(converted), 1)
	instantiated := Identity[int](called)
	allocated := make([]int, instantiated)
	return int(converted), strconv.Itoa(called), instantiated, allocated
}

func ShadowedCallAndConversion(value int) int {
	result := 0
	{
		T := func(number int) int { return number + 1 }
		result = T(value)
	}
	{
		type T int
		result = int(T(result))
	}
	return result
}

func multiResult() (int, string) {
	return 2, "north"
}

// MultipleAssignments combines tuple-producing expressions and parallel
// assignment, including blank targets and repeated assignment targets.
func MultipleAssignments(channel <-chan string, value any) (int, string, bool, bool) {
	first, text := multiResult()
	lookup, present := map[string]int{"north": 3}["north"]
	received, open := <-channel
	asserted, assertionOK := value.(string)
	first, lookup = lookup, first
	_, text = first, "updated"
	return first + lookup, text + received + asserted, present && open, assertionOK
}

func SwapValues(left, right int) (int, int) {
	left, right = right, left
	return left, right
}

// EvaluationOrder records specified call order while retaining an operand
// whose relative order is intentionally not interpreted by this fixture.
func EvaluationOrder() []int {
	order := []int{}
	mark := func(value int) int {
		order = append(order, value)
		return value
	}
	_ = append([]int{}, mark(1), mark(2))
	_ = map[int]int{mark(5): mark(6)}
	result := []int{mark(3), mark(4)}
	return result
}

func CommunicationOrder(channel chan int) int {
	go func() { channel <- 1 }()
	value := <-channel
	return value
}

// NamedResultMutation shows explicit and naked returns with deferred mutation.
func NamedResultMutation() (first int, second string) {
	defer func() {
		first++
		second += ":deferred"
	}()
	first = 1
	second = "north"
	return
}

func ExplicitNamedReturn() (first int, second int) {
	first = 2
	second = 3
	return first, second
}

func ShadowedNamedResult() (value int) {
	value = 1
	if value := value + 1; value > 0 {
		_ = value
	}
	return
}

// ClosureFactory captures a value, while AddressClosure captures an address.
func ClosureFactory(value int) func() int {
	captured := value
	return func() int {
		captured++
		return captured
	}
}

func AddressClosure(value int) func() int {
	captured := value
	return func() int {
		captured++
		return captured
	}
}

func LoopClosure(values []string) []func() string {
	result := make([]func() string, 0, len(values))
	for _, value := range values {
		result = append(result, func() string { return strings.ToUpper(value) })
	}
	return result
}

func ClosureArgument(value int) func() int {
	return func() int { return value }
}

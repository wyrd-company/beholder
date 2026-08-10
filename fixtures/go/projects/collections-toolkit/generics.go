package toolkit

import (
	"strconv"

	"example.invalid/collections-toolkit/catalog"
)

// NumberLike and OrderedLike use approximation terms and unions.
type NumberLike interface {
	~int | ~int32 | ~int64
}

type OrderedLike interface {
	~int | ~string
}

type Sized interface {
	Size() int
}

type SizedValue interface {
	Sized
}

type ComparableValue interface {
	comparable
}

type AnyValue interface {
	any
}

func AddNumbers[T NumberLike](left, right T) T {
	return left + right
}

func Minimum[T OrderedLike](left, right T) T {
	if left < right {
		return left
	}
	return right
}

func EqualValues[T comparable](left, right T) bool {
	return left == right
}

func CompareAny(left, right any) bool {
	// `any` satisfies comparable under the Go 1.20 rule. The operation remains
	// capable of a runtime panic when its dynamic values are not comparable.
	return EqualValues(left, right)
}

type SizedText string

func (text SizedText) Size() int {
	return len(text)
}

func SizeOf[T SizedValue](value T) int {
	return value.Size()
}

func MapValue[A, B any](value A, convert func(A) B) B {
	return convert(value)
}

func Identity[T any](value T) T {
	return value
}

func Zero[T any]() T {
	var zero T
	return zero
}

func PairValues[A, B any](left A, right B) (A, B) {
	return left, right
}

// GenericBox is a defined generic type with a method-owning receiver.
type GenericBox[T any] struct {
	Value T
}

func (box GenericBox[T]) Get() T {
	return box.Value
}

// GenericBoxAlias is an alias of an instantiated generic target. It does not
// declare fresh methods; Get remains owned by GenericBox.
type GenericBoxAlias[T any] = GenericBox[T]

// ExternalEnvelopeAlias crosses a package boundary while preserving identity.
type ExternalEnvelopeAlias[T any] = catalog.Envelope[T]

func GenericAliases(value string) (string, string) {
	local := GenericBoxAlias[string]{Value: value}
	external := ExternalEnvelopeAlias[string]{Value: value, Label: "label"}
	return local.Get(), external.String()
}

type ByteString interface {
	~[]byte | ~string
}

func CommonIndex[T ByteString](value T) any {
	return value[0]
}

func CommonSlice[T ByteString](value T) T {
	return value[:1]
}

func CommonRangeBytes[T ~[]byte](value T) int {
	count := 0
	for range value {
		count++
	}
	return count
}

func CommonRangeString[T ~string](value T) int {
	count := 0
	for range value {
		count++
	}
	return count
}

type ChannelLike[T any] interface {
	~chan T
}

func ReceiveValue[T any, C ChannelLike[T]](channel C) T {
	return <-channel
}

func ConvertString[T ~string](value T) string {
	return string(value)
}

func GenericInstantiationExamples() (int, string, int, int) {
	first := Identity[int](3)
	second := Identity(4)
	third := MapValue[int](5, strconv.Itoa)
	left, right := PairValues("north", 6)
	return first + second, third + left, right, Zero[int]()
}

func InferenceContexts() (int, int, string) {
	argument := Identity(7)
	var assigned int = Identity(8)
	resultLeft, resultRight := PairValues("north", argument)
	return argument, assigned, resultLeft + strconv.Itoa(resultRight)
}

// TypeParameterAssignments shows underlying type, conversion, and type
// parameter operations without requiring a runtime test.
type LocalInt int
type LocalIntAlias = LocalInt
type LocalWords []string
type LocalMap map[string]int
type LocalPointer *int
type LocalChannel chan int

func TypeParameterAssignments(value LocalInt, words LocalWords, channel LocalChannel) (int, string, int) {
	var plain int = int(value)
	converted := string(CommonSlice([]byte("north")))
	channel <- plain
	return plain, converted, len(words)
}

func CatalogConstraints(value int) int {
	return int(AddNumbers[catalog.NamedInt](catalog.NamedInt(value), catalog.NamedInt(1)))
}

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

type TextLike interface {
	~string
}

// EmptyIntersection is a valid constraint with no member type: NumberLike and
// TextLike have disjoint underlying types. It is never used as an ordinary
// value, as constraint-only interfaces cannot be values.
type EmptyIntersection interface {
	NumberLike
	TextLike
}

func EmptyIntersectionValue[T EmptyIntersection](value T) {
	_ = value
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

// GenericBoxAlias is a generic alias. It does not declare fresh methods; Get
// remains owned by GenericBox.
type GenericBoxAlias[T any] = GenericBox[T]

// Instantiated aliases preserve the method set of a concrete generic target.
type IntBoxAlias = GenericBox[int]
type IntEnvelopeAlias = catalog.Envelope[int]

// ExternalEnvelopeAlias crosses a package boundary while preserving identity.
type ExternalEnvelopeAlias[T any] = catalog.Envelope[T]

func GenericAliases(value string) (string, string) {
	local := GenericBoxAlias[string]{Value: value}
	external := ExternalEnvelopeAlias[string]{Value: value, Label: "label"}
	instantiated := IntBoxAlias{Value: len(value)}
	instantiatedExternal := IntEnvelopeAlias{Value: len(value), Label: "number"}
	return local.Get() + strconv.Itoa(instantiated.Get()), external.String() + instantiatedExternal.String()
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

type SendChannelLike[T any] interface {
	~chan<- T
}

type ReceiveChannelLike[T any] interface {
	~<-chan T
}

func ReceiveValue[T any, C ChannelLike[T]](channel C) T {
	return <-channel
}

func SendThrough[T any, C SendChannelLike[T]](channel C, value T) {
	channel <- value
}

func ReceiveThrough[T any, C ReceiveChannelLike[T]](channel C) T {
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
	var identity func(int) int = Identity
	var result func() string = Zero
	assigned := identity(8)
	resultLeft, resultRight := PairValues("north", argument)
	return argument, assigned, resultLeft + result() + strconv.Itoa(resultRight)
}

func ResultContextInference() func(int) int {
	return Identity
}

type MethodCarrier[T any] struct{}

func NewMethodCarrier[T any](value T) MethodCarrier[T] {
	_ = value
	return MethodCarrier[T]{}
}

func (carrier MethodCarrier[T]) Keep(value T) T {
	return value
}

func (carrier MethodCarrier[T]) Apply(function func(T) T, value T) T {
	return function(value)
}

func MethodInference() int {
	carrier := NewMethodCarrier(3)
	return carrier.Apply(Identity, carrier.Keep(4))
}

func ZipValues[A, B any](left []A, right []B) (A, B) {
	return left[0], right[0]
}

func UnificationInference() (int, string) {
	return ZipValues([]int{3}, []string{"north"})
}

// TypeParameterAssignments shows underlying type, conversion, and type
// parameter operations without requiring a runtime test.
type LocalInt int
type LocalIntAlias = LocalInt
type LocalWords []string
type LocalMap map[string]int
type LocalPointer *int
type LocalChannel chan int
type LocalName string

func (name LocalName) Name() string {
	return string(name)
}

type LocalInterface interface {
	Name() string
}

func TypeParameterAssignments(value LocalInt, words LocalWords, channel LocalChannel) (int, string, int) {
	var plain int = int(value)
	namedMap := LocalMap{"north": plain}
	unnamedMap := map[string]int(namedMap)
	pointer := new(int)
	*pointer = plain
	namedPointer := LocalPointer(pointer)
	unnamedPointer := (*int)(namedPointer)
	name := LocalName("north")
	var namedInterface LocalInterface = name
	var unnamedInterface interface{ Name() string } = namedInterface
	converted := string(CommonSlice([]byte("north")))
	channel <- *unnamedPointer
	return unnamedMap["north"] + len(namedInterface.Name()) - len(unnamedInterface.Name()), converted, len(words)
}

func CatalogConstraints(value int) int {
	return int(AddNumbers[catalog.NamedInt](catalog.NamedInt(value), catalog.NamedInt(1)))
}

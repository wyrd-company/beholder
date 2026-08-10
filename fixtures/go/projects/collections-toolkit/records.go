package toolkit

import (
	"reflect"

	"example.invalid/collections-toolkit/catalog"
)

// FixedPair and FixedTriple have lengths that participate in type identity.
type FixedPair [2]int
type FixedTriple [3]int

type copiedArray [3]string

// ArrayOperations uses inferred-length literals, copying, comparison, and a
// pointer-to-array slice expression.
func ArrayOperations() (FixedPair, [3]string, bool, []string) {
	inferred := [...]string{"north", "south", "east"}
	copied := inferred
	left := FixedPair{1, 2}
	right := FixedPair{1, 2}
	arrayPointer := &inferred
	return left, copied, left == right, arrayPointer[1:]
}

// TaggedRecord demonstrates tags, field order, named and anonymous fields,
// blank fields, and keyed and unkeyed literals.
type TaggedRecord struct {
	Name  string `json:"name"`
	Count int    `json:"count,omitempty"`
	_     byte
}

type AnonymousRecord struct {
	string
	int
}

func StructLiterals() (TaggedRecord, TaggedRecord, AnonymousRecord, string) {
	keyed := TaggedRecord{Name: "north", Count: 2}
	unkeyed := TaggedRecord{"south", 3, 0}
	anonymous := AnonymousRecord{"east", 4}
	tag := reflect.TypeOf(keyed).Field(0).Tag.Get("json")
	return keyed, unkeyed, anonymous, tag
}

// NestedRecord embeds fields through several depths. The direct Base wins over
// the same selector at the deeper level, while explicit paths remain valid.
type NestedRecord struct {
	catalog.Base
	DeepRecord
	*catalog.Detail
}

type DeepRecord struct {
	catalog.Base
}

func EmbeddedSelectors() (string, string, string) {
	record := NestedRecord{
		Base:       catalog.Base{Name: "shallow"},
		DeepRecord: DeepRecord{Base: catalog.Base{Name: "deep"}},
		Detail:     &catalog.Detail{Count: 3},
	}
	return record.Name, record.DeepRecord.Base.Name, record.Detail.Describe()
}

// PointerRecord embeds a pointer field and exposes promoted pointer methods.
type PointerRecord struct {
	*catalog.Detail
}

func PointerEmbedding() string {
	record := PointerRecord{Detail: &catalog.Detail{Count: 5}}
	return record.Describe()
}

// SliceOperations keeps all operations in one natural collection helper.
func SliceOperations() (bool, bool, int, int, []int, [2]int, *[2]int) {
	var nilSlice []int
	emptySlice := []int{}
	values := []int{1, 2, 3, 4}
	shared := values[:2:4]
	shared[0] = 9
	reused := append(shared, 7)
	reallocated := append([]int{1, 2}, 3, 4, 5)
	var copied = make([]int, len(values))
	copy(copied, values)
	clear(copied)
	arrayValue := [2]int(values[:2])
	arrayPointer := (*[2]int)(values)
	return nilSlice == nil, emptySlice != nil, len(reused), cap(reallocated), copied, arrayValue, arrayPointer
}

// MapOperations covers nil reads, shared map identity, deletion, clearing,
// comma-ok lookup, and iteration without depending on map order.
func MapOperations() (int, bool, int) {
	var nilMap map[string]int
	read := nilMap["missing"]
	shared := map[string]int{"north": 1}
	alias := shared
	alias["south"] = 2
	value, present := shared["north"]
	delete(shared, "north")
	clear(shared)
	for key, number := range alias {
		read += len(key) + number
	}
	return read + value, present, len(shared)
}

// InterfaceKeyOperation leaves the dynamic uncomparable-key panic isolated in
// a returned operation, so the fixture remains valid and buildable.
func InterfaceKeyOperation() func() {
	return func() {
		keys := map[any]int{}
		var key []int
		keys[key] = 1
	}
}

// PointerOperations demonstrates addressability, new, dereferencing, and
// pointer comparison.
func PointerOperations() (int, bool) {
	value := new(int)
	*value = 4
	other := value
	return *other, value == other
}

// ChannelDirections narrows a bidirectional channel to both one-way forms.
func ChannelDirections() (chan<- int, <-chan int) {
	bidirectional := make(chan int)
	var sendOnly chan<- int = bidirectional
	var receiveOnly <-chan int = bidirectional
	return sendOnly, receiveOnly
}

// Counter owns a pointer receiver and is used to show automatic address-taking.
type Counter int

func (counter *Counter) Increment() {
	if counter != nil {
		*counter++
	}
}

func AddressableMethodCall() int {
	var counter Counter
	counter.Increment()
	return int(counter)
}

// StringSliceAndArrayConversions records copy-producing conversions without
// asserting a compiler's optional allocation elision.
func StringSliceAndArrayConversions() (string, []byte, []rune, [2]byte, *[2]byte) {
	text := string([]byte{'o', 'k'})
	bytes := []byte(text)
	runes := []rune(string([]byte{0xff, 'x'}))
	arrayValue := [2]byte(bytes)
	arrayPointer := (*[2]byte)(bytes)
	return text, bytes, runes, arrayValue, arrayPointer
}

// IntegerTextConversion uses an explicit rune conversion so vet can distinguish
// a deliberate code-point conversion from a suspicious string(int) operation.
func IntegerTextConversion(value int32) string {
	return string(rune(value))
}

func StringImmutability(text string) []byte {
	bytes := []byte(text)
	if len(bytes) > 0 {
		bytes[0] = 'x'
	}
	return []byte(text)
}

package pipeline

import (
	"reflect"
	"sync/atomic"
	"unsafe"
)

// TypedCounter uses the typed atomic API. Its zero value is ready for use and
// the value is never copied after the first operation.
type TypedCounter struct {
	value atomic.Int64
}

func (c *TypedCounter) TypedOperations() (loaded int64, swapped bool) {
	c.value.Store(4)
	c.value.Add(2)
	swapped = c.value.CompareAndSwap(6, 9)
	loaded = c.value.Load()
	return loaded, swapped
}

// LegacyAligned puts the 64-bit word first in an allocated struct. On 32-bit
// targets the first word of an allocated struct is the required alignment
// position for the legacy 64-bit atomic functions.
type LegacyAligned struct {
	value int64
	tail  byte
}

func LegacyAtomicOperations() int64 {
	value := new(LegacyAligned)
	atomic.StoreInt64(&value.value, 7)
	atomic.AddInt64(&value.value, 1)
	atomic.CompareAndSwapInt64(&value.value, 8, 11)
	return atomic.LoadInt64(&value.value)
}

// LegacyMisaligned names the layout that must not be passed to a legacy 64-bit
// atomic operation on 32-bit targets.
type LegacyMisaligned struct {
	prefix byte
	value  int64
}

func LegacyLayout() (aligned uintptr, unsafeOffset uintptr) {
	return unsafe.Alignof(LegacyAligned{}), unsafe.Offsetof(LegacyMisaligned{}.value)
}

// CopyAtomicWrapper demonstrates the copy boundary without assigning the
// wrapper directly. A typed atomic value must not be copied after first use;
// reflection makes the otherwise-hidden copy explicit for this fixture.
func CopyAtomicWrapper(source *atomic.Uint64) *atomic.Uint64 {
	destination := new(atomic.Uint64)
	reflect.ValueOf(destination).Elem().Set(reflect.ValueOf(source).Elem())
	return destination
}

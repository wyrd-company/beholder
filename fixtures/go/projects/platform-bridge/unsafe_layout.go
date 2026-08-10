package bridge

import "unsafe"

// LayoutProbe gives the layout helpers an object with padding and alignment.
type LayoutProbe struct {
	Prefix byte
	Count  uint32
	Stamp  uint64
}

// LayoutShape reports size, alignment, and field offset for the selected word
// size.
func LayoutShape() (size, alignment, countOffset uintptr) {
	var probe LayoutProbe
	return unsafe.Sizeof(probe), unsafe.Alignof(probe), unsafe.Offsetof(probe.Count)
}

// LayoutCount reads a field through pointer conversion and arithmetic within
// the same allocated object.
func LayoutCount(value uint32) uint32 {
	probe := LayoutProbe{Count: value}
	address := unsafe.Add(unsafe.Pointer(&probe), unsafe.Offsetof(probe.Count))
	return *(*uint32)(address)
}

// ByteAt reads one element through an in-object unsafe pointer offset.
func ByteAt(data []byte, index int) byte {
	if index < 0 || index >= len(data) {
		return 0
	}
	return *(*byte)(unsafe.Add(unsafe.Pointer(&data[0]), index))
}

// SliceView uses the documented unsafe.Slice helper for a non-empty byte span.
func SliceView(data []byte) []byte {
	if len(data) == 0 {
		return nil
	}
	return unsafe.Slice(&data[0], len(data))
}

// StringView uses the documented unsafe.String helper for a non-empty byte
// span. The caller keeps data alive for the returned string's use.
func StringView(data []byte) string {
	if len(data) == 0 {
		return ""
	}
	return unsafe.String(&data[0], len(data))
}

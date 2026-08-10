//go:build cgo

package bridge

/*
#cgo CFLAGS: -DBRIDGE_MEMORY=1
#cgo linux LDFLAGS: -Wl,--no-as-needed
#cgo darwin LDFLAGS: -Wl,-dead_strip
#cgo windows LDFLAGS: -static
#include <stdint.h>
#include <stdlib.h>

static uint32_t bridge_sum(const void *data, size_t size) {
	const unsigned char *bytes = (const unsigned char *)data;
	uint32_t total = 0;
	for (size_t index = 0; index < size; index++) {
		total += bytes[index];
	}
	return total;
}

static void *bridge_alloc(size_t size) {
	return malloc(size);
}
*/
import "C"

import (
	"runtime"
	"unsafe"
)

// ForeignSum passes a pointer to Go memory for the duration of one C call.
func ForeignSum(data []byte) uint32 {
	if len(data) == 0 {
		return 0
	}
	total := uint32(C.bridge_sum(unsafe.Pointer(&data[0]), C.size_t(len(data))))
	runtime.KeepAlive(data)
	return total
}

// ForeignCopy obtains C-owned memory, copies it into Go memory, and releases
// the C allocation before returning.
func ForeignCopy(data []byte) []byte {
	if len(data) == 0 {
		return nil
	}
	foreign := C.bridge_alloc(C.size_t(len(data)))
	if foreign == nil {
		return nil
	}
	defer C.free(foreign)
	result := unsafe.Slice((*byte)(foreign), len(data))
	copy(result, data)
	return append([]byte(nil), result...)
}

//go:build cgo

package bridge

/*
#cgo CFLAGS: -DBRIDGE_WORD=1
#cgo linux CFLAGS: -DBRIDGE_POSIX=1
#cgo darwin CFLAGS: -DBRIDGE_DARWIN=1
#cgo windows CFLAGS: -DBRIDGE_WINDOWS=1
#include <stdint.h>

enum { BRIDGE_MAGIC = 41 };

typedef struct {
	uint32_t value;
} bridge_word;

static bridge_word bridge_make_word(uint32_t value) {
	bridge_word word = {value};
	return word;
}
*/
import "C"

// ForeignWord uses a C type, C function, and C struct field from the preamble.
func ForeignWord(value uint32) uint32 {
	word := C.bridge_make_word(C.uint32_t(value))
	return uint32(word.value) + uint32(C.BRIDGE_MAGIC)
}

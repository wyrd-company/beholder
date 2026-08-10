//go:build 386

package toolkit

import (
	"strconv"
	"unsafe"
)

const nativeBoundary = uint32(1 << 31)

// NativeWidths describes the target-sized aliases on 32-bit builds.
func NativeWidths() (int, uint, uintptr, int, int) {
	var signed int = 1
	var unsigned uint = 2
	var address uintptr = unsafe.Sizeof(signed)
	return signed, unsigned, address, strconv.IntSize, int(nativeBoundary >> 29)
}

func FixedNumericKinds() (int32, uint32, byte, rune) {
	return 32, 64, 8, 'R'
}

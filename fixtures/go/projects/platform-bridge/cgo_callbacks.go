//go:build cgo

package bridge

/*
#cgo CFLAGS: -DBRIDGE_CALLBACK=1
#include <stdint.h>

extern void bridgeCallback(uint32_t value, uintptr_t handle);
void bridge_invoke(uintptr_t handle);
*/
import "C"

import (
	"fmt"
	"runtime/cgo"
)

// bridgeCallback is the Go half of the callback declared in callbacks.c.
//
//export bridgeCallback
func bridgeCallback(value C.uint32_t, handle C.uintptr_t) {
	target := cgo.Handle(handle).Value().(*string)
	*target = fmt.Sprintf("callback-%d", uint32(value))
}

// InvokeCallback passes a cgo.Handle through C without retaining a Go pointer.
func InvokeCallback() string {
	result := ""
	handle := cgo.NewHandle(&result)
	defer handle.Delete()
	C.bridge_invoke(C.uintptr_t(handle))
	return result
}

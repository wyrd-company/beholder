//go:build go1.21 && (linux || darwin || windows || wasm)

package bridge

// LoweredLoopCapture is compiled with Go 1.21 loop semantics even though the
// module and compiler are newer. The range variable is intentionally captured
// by each closure to preserve the lower-version behavior.
func LoweredLoopCapture(values []int) []func() int {
	result := make([]func() int, 0, len(values))
	for _, value := range values {
		result = append(result, func() int { return value })
	}
	return result
}

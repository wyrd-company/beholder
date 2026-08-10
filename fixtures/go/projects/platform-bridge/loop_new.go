//go:build go1.23 && (linux || darwin || windows || wasm)

package bridge

// LoopCapture uses per-iteration loop variables in the language version raised
// by its release constraint.
func LoopCapture(values []int) []func() int {
	result := make([]func() int, 0, len(values))
	for _, value := range values {
		result = append(result, func() int { return value })
	}
	return result
}

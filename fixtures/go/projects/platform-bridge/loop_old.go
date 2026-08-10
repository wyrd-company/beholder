//go:build !go1.23

package bridge

// LoopCapture is the pre-per-iteration counterpart for older toolchains.
func LoopCapture(values []int) []func() int {
	result := make([]func() int, 0, len(values))
	for index := range values {
		value := values[index]
		result = append(result, func() int { return value })
	}
	return result
}

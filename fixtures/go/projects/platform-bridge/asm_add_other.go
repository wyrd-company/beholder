//go:build !amd64

package bridge

// FastCombine is the portable provider on architectures without the fixture's
// amd64 assembly implementation.
func FastCombine(a, b uint64) uint64 { return a + b }

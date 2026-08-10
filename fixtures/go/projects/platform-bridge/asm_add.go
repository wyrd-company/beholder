//go:build amd64

package bridge

import "example.com/portablebridge/internal/fast"

// FastCombine calls the assembly provider on amd64.
func FastCombine(a, b uint64) uint64 { return fast.Add(a, b) }

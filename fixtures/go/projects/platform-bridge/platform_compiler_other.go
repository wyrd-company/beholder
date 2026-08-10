//go:build !gc

package bridge

// CompilerFlavor is the alternate-compiler counterpart.
func CompilerFlavor() string { return "alternate" }

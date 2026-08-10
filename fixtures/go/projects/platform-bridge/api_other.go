//go:build !linux && !darwin && !windows

package bridge

// TargetToken records the portable API when no native syscall surface is
// selected.
type TargetToken struct{ Name string }

// PlatformSurface exposes the portable API selected by the import graph.
func PlatformSurface() string { return "portable-surface" }

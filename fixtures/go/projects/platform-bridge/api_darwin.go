//go:build darwin

package bridge

import "syscall"

// TargetToken records a Darwin signal-shaped declaration.
type TargetToken struct{ Signal syscall.Signal }

// PlatformSurface exposes the Darwin API selected by the import graph.
func PlatformSurface() string { return "darwin-syscall-signal" }

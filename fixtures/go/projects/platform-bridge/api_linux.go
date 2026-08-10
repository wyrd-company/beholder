//go:build linux

package bridge

import "syscall"

// TargetToken records a Unix descriptor-shaped declaration.
type TargetToken struct{ Descriptor syscall.Errno }

// PlatformSurface exposes the Linux API selected by the import graph.
func PlatformSurface() string { return "linux-syscall-errno" }

//go:build windows

package bridge

import "syscall"

// TargetToken records a Windows handle-shaped declaration.
type TargetToken struct{ Handle syscall.Handle }

// PlatformSurface exposes the Windows API selected by the import graph.
func PlatformSurface() string { return "windows-syscall-handle" }

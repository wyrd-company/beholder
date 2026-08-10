// Package bridge exposes one small interface whose implementation follows the
// selected operating system, compiler, and foreign-code boundary.
package bridge

import "runtime"

// Snapshot is the stable, portable part of the package surface.
type Snapshot struct {
	OS           string
	Architecture string
	Compiler     string
	Cgo          string
	Endpoint     string
	Feature      string
}

// SnapshotFor returns the declarations selected for the current build.
func SnapshotFor() Snapshot {
	return Snapshot{
		OS:           runtime.GOOS,
		Architecture: runtime.GOARCH,
		Compiler:     CompilerFlavor(),
		Cgo:          CgoFlavor(),
		Endpoint:     PlatformEndpoint(),
		Feature:      FeatureLevel(),
	}
}

// NativeBlend exercises the assembly provider when the selected architecture
// has one and uses the portable provider elsewhere.
func NativeBlend(a, b uint64) uint64 {
	return FastCombine(a, b)
}

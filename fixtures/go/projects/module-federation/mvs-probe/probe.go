package probe

import (
	"example.invalid/graph-root"
	graphrootlow "example.invalid/graph-root-low"
)

// SelectedVersions exposes graphdep through roots that require different
// versions. Minimal version selection resolves both imports to the higher one.
func SelectedVersions() (string, string) {
	return graphroot.Version(), graphrootlow.Version()
}

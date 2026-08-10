package graph

import "example.invalid/graph-root"

// SelectedVersion exposes the higher transitive graphdep version selected by MVS.
func SelectedVersion() string {
	return graphroot.Version()
}

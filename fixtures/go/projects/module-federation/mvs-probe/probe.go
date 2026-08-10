package probe

import "example.invalid/graph-root"

// SelectedVersion makes the lower direct requirement and higher transitive
// requirement observable from a small independent module.
func SelectedVersion() string { return graphroot.Version() }

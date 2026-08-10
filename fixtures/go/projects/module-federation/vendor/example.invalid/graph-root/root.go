package graphroot

import "example.invalid/graphdep"

// Version comes from the higher transitive graphdep requirement.
func Version() string { return graphdep.Version() }

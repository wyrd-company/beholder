package graphrootlow

import "example.invalid/graphdep"

// Version comes from the lower graphdep requirement before selection.
func Version() string { return graphdep.Version() }

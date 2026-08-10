package pantry

// ---
// relationships:
//   references:
//     - coverage
// ---

import (
	"example.com/fixture/package-structure/branches/evening"
	"example.com/fixture/package-structure/branches/morning"
	"example.com/fixture/package-structure/registry"
)

func recordInitialization(name string) string {
	initTrace = append(initTrace, name)
	return name
}

func initializeThroughFunction() string {
	initTrace = append(initTrace, "through-function")
	return directSource + " result"
}

// InitializationTrace exposes the local and imported initialization markers.
func InitializationTrace() []string {
	local := append([]string(nil), initTrace...)
	local = append(local, morning.Name, evening.Name)
	local = append(local, registry.Snapshot()...)
	return local
}

func cycleValues() (int, int) {
	return cycleLeft, cycleRight
}

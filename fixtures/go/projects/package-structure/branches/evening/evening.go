package evening

// ---
// relationships:
//   references:
//     - coverage
// ---

import "example.com/fixture/package-structure/registry"

var Name = "evening"

func init() {
	if registry.Ready {
		registry.Register(Name)
	}
}

package pantry

// ---
// relationships:
//   references:
//     - coverage
// ---

import . "example.com/fixture/package-structure/patterns"

func dotImportExample(name string) Pattern {
	return New(name)
}

package pantry

// ---
// relationships:
//   references:
//     - coverage
// ---

import "example.com/fixture/package-structure/measure"

func defaultImportExample(units int) quantity.Amount {
	return quantity.New(units)
}

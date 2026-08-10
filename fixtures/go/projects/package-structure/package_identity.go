package pantry

// ---
// relationships:
//   references:
//     - coverage
// ---

import (
	brightLabel "example.com/fixture/package-structure/labels/bright"
	dimLabel "example.com/fixture/package-structure/labels/dim"
	measureQuantity "example.com/fixture/package-structure/measure"
)

func packageIdentityExample() string {
	bright := brightLabel.New("bright")
	dim := dimLabel.New("dim")
	amount := measureQuantity.New(8)
	return bright.Name() + "/" + dim.Name() + "/" + amount.String()
}

package pantry

// ---
// relationships:
//   references:
//     - coverage
// ---

import (
	brightLabel "example.com/fixture/package-structure/labels/bright"
	dimLabel "example.com/fixture/package-structure/labels/dim"
	measureAlias "example.com/fixture/package-structure/measure"
	"fmt"
)

func aliasImportExample(units int) string {
	amount := measureAlias.New(units)
	bright := brightLabel.New("warm")
	dim := dimLabel.New("cool")
	return fmt.Sprintf("%d:%s:%s", amount.Units(), bright.Name(), dim.Name())
}

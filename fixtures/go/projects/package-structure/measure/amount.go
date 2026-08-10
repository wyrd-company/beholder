package quantity

// ---
// relationships:
//   references:
//     - coverage
// ---

import "strconv"

// Amount is a quantity stored in whole units.
type Amount struct {
	units int
}

func New(units int) Amount {
	return Amount{units: units}
}

func (a Amount) Units() int {
	return a.units
}

func (a Amount) String() string {
	return strconv.Itoa(a.units)
}

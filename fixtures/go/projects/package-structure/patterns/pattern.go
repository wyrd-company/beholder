package pattern

// ---
// relationships:
//   references:
//     - coverage
// ---

// Pattern is a named pantry arrangement.
type Pattern struct {
	Name string
}

func New(name string) Pattern {
	return Pattern{Name: name}
}

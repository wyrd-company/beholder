package pantry

// ---
// relationships:
//   references:
//     - coverage
// ---

func init() {
	initTrace = append(initTrace, "first local init")
	cycleRight = cycleLeft + 1
}

func init() {
	initTrace = append(initTrace, "second local init")
}

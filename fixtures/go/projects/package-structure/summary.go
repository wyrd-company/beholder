package pantry

// ---
// relationships:
//   references:
//     - coverage
// ---

// Summary returns a compact value from the package's exported surface.
func Summary() string {
	shelf := Shelf{Public: "upper", private: "lower"}
	return shelf.Open() + ":" + BeforeDeclaration()
}

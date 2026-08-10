package pantry

// ---
// relationships:
//   references:
//     - coverage
// ---

func shortDeclaration() int {
	first := 1
	first, second := 2, 3
	left, err := typedPair()
	if err != nil {
		return 0
	}
	left, extra := left, second
	{
		first, nested := first+extra, left+second
		return first + nested
	}
}

func partialRedeclaration(value int) int {
	result := value
	result, added := result+1, value+2
	return result + added
}

func initializerScope(value int) int {
	if value, ok := chooseValue(value); ok {
		return value
	}
	return value
}

func chooseValue(value int) (int, bool) {
	return value + 1, true
}

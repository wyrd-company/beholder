package pantry

// ---
// relationships:
//   references:
//     - coverage
// ---

var packageScopedValue = 10

func scopeParameters(input int) (result int) {
	result = input
	{
		input := input + 1
		result += input
	}
	return
}

func nestedScopes() int {
	value := packageScopedValue
	{
		value := value + 2
		{
			value := value + 3
			packageScopedValue = value
		}
	}
	return packageScopedValue
}

func functionScope(value int) int {
	local := value
	return local
}

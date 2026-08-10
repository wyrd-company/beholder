package pantry

// ---
// relationships:
//   references:
//     - coverage
// ---

const unusedPackageConstant = 12

type unusedPackageType struct {
	value int
}

var unusedPackageVariable = unusedPackageType{value: unusedPackageConstant}

func unusedParameters(unusedInput string) (unusedResult int) {
	return 0
}

func suppressedLocal() int {
	value := 4
	_ = value
	assignmentOnly := 0
	assignmentOnly = 1
	_ = assignmentOnly
	return value
}

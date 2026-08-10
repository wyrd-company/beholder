package go121

// SharedThreeClauseClosures keeps a loop variable shared under go 1.21.
func SharedThreeClauseClosures() []func() int {
	var result []func() int
	for index := 0; index < 3; index++ {
		result = append(result, func() int { return index })
	}
	return result
}

// SharedRangeAddresses keeps range variables shared under go 1.21.
func SharedRangeAddresses(values []string) []*string {
	var result []*string
	for _, value := range values {
		result = append(result, &value)
	}
	return result
}

// OutsideLoopVariable remains shared in every language version.
func OutsideLoopVariable(values []string) []*string {
	var value string
	var result []*string
	for _, value = range values {
		result = append(result, &value)
	}
	return result
}

func Carry[T any](value T) T {
	return value
}

func Zero[T any]() T {
	var zero T
	return zero
}

func ReturnCarry() func(int) int {
	return Carry
}

// VersionedInference records inference available under the go 1.21 module gate.
func VersionedInference() (int, string) {
	assigned := ReturnCarry()
	var result func() string = Zero
	text := Carry("north")
	return assigned(3), result() + text
}

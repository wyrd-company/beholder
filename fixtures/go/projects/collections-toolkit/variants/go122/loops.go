package go122

// PerIterationClosures use the go 1.22 per-iteration variable semantics.
func PerIterationClosures() []func() int {
	var result []func() int
	for index := 0; index < 3; index++ {
		result = append(result, func() int { return index })
	}
	return result
}

// PerIterationRangeAddresses get distinct range variables under go 1.22.
func PerIterationRangeAddresses(values []string) []*string {
	var result []*string
	for _, value := range values {
		result = append(result, &value)
	}
	return result
}

// IntegerRangeForms includes signed, unsigned, typed, untyped, and zero ranges.
func IntegerRangeForms() (int, uint, int, int) {
	var signed, zero, negative int
	var unsigned uint
	for value := range 3 {
		signed += value
	}
	for value := range uint(2) {
		unsigned += value
	}
	for range 0 {
		zero++
	}
	for range -1 {
		negative++
	}
	return signed, unsigned, zero, negative
}

func Carry[T any](value T) T {
	return value
}

// VersionedInference records inference under the go 1.22 module gate.
func VersionedInference() (int, string) {
	assigned := Carry(4)
	text := Carry("south")
	return assigned, text
}

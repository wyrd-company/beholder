//go:build go1.25

package versioned

// Values uses a release-constrained file selected by the installed toolchain.
func Values() []int {
	values := []int{}
	for value := range sequence {
		values = append(values, value)
	}
	return values
}

func sequence(yield func(int) bool) {
	for value := 1; value <= 2; value++ {
		if !yield(value) {
			return
		}
	}
}

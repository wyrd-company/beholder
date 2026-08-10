package go123

// ZeroValueIterator yields no values and demonstrates the zero-argument form.
func ZeroValueIterator(yield func() bool) {
	if !yield() {
		return
	}
}

// OneValueIterator stops when its yield function returns false.
func OneValueIterator(yield func(int) bool) {
	for value := 0; value < 3; value++ {
		if !yield(value) {
			return
		}
	}
}

// TwoValueIterator demonstrates the key/value iterator signature.
func TwoValueIterator(yield func(string, int) bool) {
	items := []string{"north", "south"}
	for index, item := range items {
		if !yield(item, index) {
			return
		}
	}
}

// Collect uses range over zero-, one-, and two-value iterator functions.
func Collect() []string {
	var result []string
	for range ZeroValueIterator {
		result = append(result, "zero")
	}
	for value := range OneValueIterator {
		result = append(result, string(rune('0'+value)))
	}
outer:
	for item, index := range TwoValueIterator {
		result = append(result, item)
		if index == 0 {
			continue
		}
		break outer
	}
	return result
}

// GenericBox is the generic alias target used by the versioned alias example.
type GenericBox[T any] struct {
	Value T
}

func (b GenericBox[T]) ValueOr(zero T) T {
	_ = zero
	return b.Value
}

// GenericBoxAlias preserves GenericBox identity across a package boundary.
type GenericBoxAlias[T any] = GenericBox[T]

func AliasValue(value int) int {
	var box GenericBoxAlias[int] = GenericBox[int]{Value: value}
	return box.ValueOr(0)
}

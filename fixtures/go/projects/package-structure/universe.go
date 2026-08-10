package pantry

// ---
// relationships:
//   references:
//     - coverage
// ---

var unshadowedAny any = "unshadowed"

func acceptsComparable[T comparable](value T) T {
	return value
}

func unshadowedUniverse() int {
	values := map[string]int{"one": 1, "two": 2}
	clear(values)
	return min(4, max(1, len(values)))
}

func shadowedUniverse() int {
	type any struct {
		value int
	}
	var localAny any
	comparable := "local comparable"
	clear := func(values map[string]int) {
		for key := range values {
			delete(values, key)
		}
	}
	min := func(left, right int) int {
		if left < right {
			return left
		}
		return right
	}
	max := func(left, right int) int {
		if left > right {
			return left
		}
		return right
	}
	len := func(string) int { return 1 }
	true := "local truth"
	nil := "local nil"
	values := map[string]int{"one": 1}
	clear(values)
	_ = localAny
	_ = comparable
	_ = len(nil)
	_ = true
	_ = nil
	return max(min(3, 2), 1)
}

package go120

func Equal[T comparable](left, right T) bool {
	return left == right
}

// InterfaceComparable records the Go 1.20 change that permits any to satisfy
// a comparable constraint. A non-comparable dynamic value may still panic.
func InterfaceComparable(value any) bool {
	return Equal(value, value)
}

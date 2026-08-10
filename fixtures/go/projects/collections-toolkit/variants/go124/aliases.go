package go124

// Box is a generic type whose methods are inherited by its aliases.
type Box[T any] struct {
	Value T
}

func (b Box[T]) ValueOr(zero T) T {
	_ = zero
	return b.Value
}

// BoxAlias demonstrates the generally available generic alias syntax.
type BoxAlias[T any] = Box[T]

// CrossPackageAliasUse is a stable locator for the generic alias variant.
func CrossPackageAliasUse(value string) string {
	var box BoxAlias[string] = Box[string]{Value: value}
	return box.ValueOr("")
}

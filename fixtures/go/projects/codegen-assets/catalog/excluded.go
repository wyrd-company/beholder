//go:build ignore

// Package catalog excluded documentation.
//
// This file is excluded from the build and demonstrates build-excluded docs.
// It contains package-level documentation that is not part of the normal
// build, satisfying the build-excluded docs variant.
//
// # Excluded Section
//
//   - This heading is inside an ignored file.
//   - Links like [Item] would still be parsed as doc links if included.
package catalog

// ExcludedItem is documented but never compiled.
//
// Deprecated: This type is part of excluded docs and not used.
type ExcludedItem struct {
	ID string
}

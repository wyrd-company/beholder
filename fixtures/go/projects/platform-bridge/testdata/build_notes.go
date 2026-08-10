// Package testdata preserves source that directory-aware tools can see while
// the ordinary package loader ignores this directory.
package testdata

// The following examples are inert prose, not Go build constraints:
// go:build linux (the space makes this ordinary comment text)
// malformed expression: (linux &&
// no-files-match configuration: fixture_never_selected
func ConstraintNotes() string { return "notes" }

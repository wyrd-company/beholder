//go:build (linux || darwin) && cgo && !fixture_disabled
// +build linux darwin
// +build cgo
// +build !fixture_disabled

package bridge

// LegacyTagged keeps an equivalent pre-1.17 spelling beside the modern form.
func LegacyTagged() string { return "legacy" }

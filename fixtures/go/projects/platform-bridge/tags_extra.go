//go:build ((linux && amd64 && cgo) || (windows && 386 && !cgo)) && (go1.23 || fixture_extra) && !fixture_disabled

package bridge

// ExtraTagged is selected by the custom fixture_extra tag on the default
// context. The other branch documents a valid Windows pure-Go selection.
func ExtraTagged() string { return "extra" }

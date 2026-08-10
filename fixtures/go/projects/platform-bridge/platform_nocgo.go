//go:build !cgo

package bridge

// CgoFlavor records that the pure-Go boundary is selected.
func CgoFlavor() string { return "pure-go" }

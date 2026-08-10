//go:build cgo

package bridge

// CgoFlavor records that the C boundary is available.
func CgoFlavor() string { return "cgo" }

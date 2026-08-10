//go:build windows

package bridge

// PlatformEndpoint is the Windows implementation of the logical interface.
func PlatformEndpoint() string { return "local-windows" }

//go:build darwin

package bridge

// PlatformEndpoint is the Darwin implementation of the logical interface.
func PlatformEndpoint() string { return "local-darwin" }

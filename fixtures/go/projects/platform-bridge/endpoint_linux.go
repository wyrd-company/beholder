//go:build linux

package bridge

// PlatformEndpoint is one of several mutually exclusive definitions of the
// same logical interface.
func PlatformEndpoint() string { return "local-unix" }

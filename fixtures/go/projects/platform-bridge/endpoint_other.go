//go:build !linux && !darwin && !windows

package bridge

// PlatformEndpoint covers targets without one of the named platform files.
func PlatformEndpoint() string { return "local-other" }

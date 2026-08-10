//go:build !go1.23

package bridge

// ReleaseMarker is the lower-version counterpart for older toolchains.
func ReleaseMarker() string { return "before-go1.23" }

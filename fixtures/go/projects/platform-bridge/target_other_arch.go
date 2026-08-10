//go:build !amd64 && !386

package bridge

func architectureMarker() string { return "other-arch-suffix" }

//go:build !linux && !windows && !darwin

package bridge

func platformOSMarker() string { return "other-os-suffix" }

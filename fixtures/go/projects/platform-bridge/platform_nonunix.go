//go:build !unix

package bridge

func familyMarker() string { return "non-unix" }

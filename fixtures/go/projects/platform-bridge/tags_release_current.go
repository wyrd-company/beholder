//go:build go1.23

package bridge

// ReleaseMarker is compiled with an effective language version raised by the
// go1.23 release constraint relative to the module's go directive.
func ReleaseMarker() string { return "go1.23-or-newer" }

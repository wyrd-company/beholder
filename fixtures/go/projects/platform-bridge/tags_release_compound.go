//go:build (go1.23 && !go1.24) || go1.26

package bridge

// CompoundReleaseMarker records a compound positive, negated, and release-tag
// expression. It remains distinct from the positive release marker.
func CompoundReleaseMarker() string { return "compound-release" }

//go:build !amd64

package bridge

// FeatureLevel separates GOARCH from the amd64 feature tags.
func FeatureLevel() string { return "not-amd64" }

//go:build amd64 && amd64.v1 && !amd64.v2

package bridge

// FeatureLevel is the baseline amd64 feature selection.
func FeatureLevel() string { return "amd64.v1" }

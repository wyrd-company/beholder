//go:build amd64 && amd64.v2

package bridge

// FeatureLevel is selected for amd64.v2 and higher feature settings.
func FeatureLevel() string { return "amd64.v2-or-newer" }

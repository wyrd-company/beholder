// +build fixture_legacy

package bridge

// Pre17Tagged preserves a legacy-only build constraint for older toolchains.
func Pre17Tagged() string { return "pre-1.17" }

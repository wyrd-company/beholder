package leaf

// Marker is exported from a nested internal package but visible only to its parent tree.
func Marker() string { return "nested-internal" }

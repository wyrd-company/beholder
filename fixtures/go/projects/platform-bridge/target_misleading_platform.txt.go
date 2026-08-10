package bridge

// MisleadingPlatformSuffix is intentionally generic: only canonical suffixes
// such as _linux.go and _amd64.go constrain a file.
func MisleadingPlatformSuffix() string { return "generic-suffix" }

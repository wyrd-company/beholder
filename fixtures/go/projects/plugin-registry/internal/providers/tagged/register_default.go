//go:build !tagprovider

package tagged

// Available reports whether the optional provider was compiled in.
func Available() bool {
	return false
}

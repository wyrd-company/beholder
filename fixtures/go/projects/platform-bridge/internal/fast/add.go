//go:build amd64

package fast

//go:noescape
func Add(a, b uint64) uint64

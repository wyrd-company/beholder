//go:build tagprovider

package tagged

import (
	"context"

	"example.invalid/garden-ledger/pkg/registry"
)

var initializationCount int

func init() {
	initializationCount++
	if err := registry.Register("night", night); err != nil {
		panic(err)
	}
}

func night(context.Context) string {
	return "quiet stars"
}

// Available reports whether the optional provider was compiled in.
func Available() bool {
	return true
}

package core

import (
	"context"

	"example.invalid/garden-ledger/pkg/registry"
)

var initializationCount int

func init() {
	initializationCount++
	if err := registry.Register("morning", morning); err != nil {
		panic(err)
	}
	if err := registry.Register("evening", evening); err != nil {
		panic(err)
	}
}

func morning(context.Context) string {
	return "soft light"
}

func evening(context.Context) string {
	return "long shadows"
}

// InitializationCount reports the number of executions in this process.
func InitializationCount() int {
	return initializationCount
}

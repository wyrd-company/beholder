package main

import (
	"context"

	"example.invalid/garden-ledger/pkg/registry"
)

// Version is resolved as a string-named plugin variable.
var Version = "seasonal-v1"

// InitCount makes plugin initialization observable without a second init call.
var InitCount int

func init() {
	InitCount++
	if err := registry.Register("seasonal", func(context.Context) string {
		return "seasonal breeze"
	}); err != nil {
		panic(err)
	}
}

// Provide is resolved as a string-named plugin function.
func Provide(seed string) string {
	return seed + ":" + Version
}

func main() {}

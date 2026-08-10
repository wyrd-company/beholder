package runtimeapp

import (
	"context"

	"example.invalid/garden-ledger/internal/pluginloader"
	_ "example.invalid/garden-ledger/internal/providers/core"
	_ "example.invalid/garden-ledger/internal/providers/tagged"
	"example.invalid/garden-ledger/pkg/registry"
)

// Select resolves and calls a provider selected at runtime.
func Select(key string) (string, error) {
	provider, err := registry.Lookup(key)
	if err != nil {
		return "", err
	}
	return provider(context.Background()), nil
}

// RegisterManual contrasts explicit registration with init-time registration.
func RegisterManual(key string) error {
	return registry.Register(key, func(context.Context) string {
		return "hand-entered"
	})
}

// TryDuplicate demonstrates the duplicate-key path without replacing a provider.
func TryDuplicate(key string) error {
	return registry.Register(key, func(context.Context) string {
		return "replacement"
	})
}

// LoadPlugin is the host entry point for the optional plugin build.
func LoadPlugin(path string) (pluginloader.Report, error) {
	return pluginloader.Load(path, "Provide", "Version")
}

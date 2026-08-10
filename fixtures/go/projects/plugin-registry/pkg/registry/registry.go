package registry

import (
	"context"
	"fmt"
	"sort"
	"sync"
)

// Provider supplies one short almanac entry for a request.
type Provider func(context.Context) string

var (
	mu        sync.RWMutex
	providers = map[string]Provider{}
)

// Register adds a provider under a runtime key.
func Register(key string, provider Provider) error {
	if key == "" {
		return fmt.Errorf("provider key is empty")
	}
	if provider == nil {
		return fmt.Errorf("provider %q is nil", key)
	}

	mu.Lock()
	defer mu.Unlock()
	if _, exists := providers[key]; exists {
		return fmt.Errorf("provider %q is already registered", key)
	}
	providers[key] = provider
	return nil
}

// Lookup resolves a provider using a runtime key.
func Lookup(key string) (Provider, error) {
	mu.RLock()
	provider, exists := providers[key]
	mu.RUnlock()
	if !exists {
		return nil, fmt.Errorf("provider %q is not registered", key)
	}
	return provider, nil
}

// Names returns registered keys in stable order.
func Names() []string {
	mu.RLock()
	keys := make([]string, 0, len(providers))
	for key := range providers {
		keys = append(keys, key)
	}
	mu.RUnlock()
	sort.Strings(keys)
	return keys
}

// TemplateFunctions returns functions that a name-driven template may call.
func TemplateFunctions() map[string]any {
	return map[string]any{
		"decorate": func(value string) string {
			return "<" + value + ">"
		},
	}
}

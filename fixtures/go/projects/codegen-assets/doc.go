// Package codegenassets provides inventory catalog management with
// embedded assets and generated source.
//
// # Overview
//
// This package demonstrates package documentation, embedded files, and
// generated code. It links to [Inventory] and [catalog.Item] for
// cross-package navigation and to [Store] within this package.
//
// Features:
//
//   - Catalog handling via [catalog.Item]
//   - Asset embedding via [embed.FS]
//   - Generated inventory via [generated.InventoryKind]
//
// See also [net/http.Request] for a standard library link example.
package codegenassets

import (
	_ "example.com/codegen-assets/catalog"
	_ "example.com/codegen-assets/embedpkg"
	_ "example.com/codegen-assets/generated"
)

// Store holds a collection of inventory entries.
//
// Deprecated: Use [CatalogStore] instead. It provides extended lookup.
type Store struct {
	Name string
}

// CatalogStore is the replacement for [Store].
//
// It references [catalog.Item] and [Inventory] to demonstrate intra- and
// cross-package doc links.
type CatalogStore struct {
	Items []Inventory
}

// Inventory represents a stock item in the top-level package.
//
// # Fields
//
//   - ID identifies the entry.
//   - Label is a human-readable description.
//
// Inventory links to [catalog.Item] for cross-package documentation.
type Inventory struct {
	ID    int
	Label string
}

// ListInventories returns all inventories.
//
//   - It iterates over the store.
//   - It returns a copy to avoid mutation.
//
// See [CatalogStore] and [Inventory].
func ListInventories(s CatalogStore) []Inventory {
	out := make([]Inventory, len(s.Items))
	copy(out, s.Items)
	return out
}

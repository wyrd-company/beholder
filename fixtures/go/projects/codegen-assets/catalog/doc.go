// Package catalog manages discrete inventory records for the asset library.
//
// # Catalog Package
//
// The catalog holds item definitions used across the fixture.
//
// Items:
//
//   - [Item] is the primary record.
//   - [Registry] aggregates items.
//
// Cross-package links include [net/http.Request] as an external example
// and local links like [Item].
package catalog

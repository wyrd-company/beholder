package catalog

// Item represents a single catalog entry.
//
// # Item Details
//
//   - SKU is the stock keeping unit.
//   - Title is the display name.
//   - Quantity tracks available units.
//
// Item is linked from [Registry] and from package docs via [Item].
type Item struct {
	SKU      string
	Title    string
	Quantity int
}

// Registry holds a set of [Item] values.
//
// Deprecated: Use [Collection] instead. It supports additional metadata.
type Registry struct {
	Items []Item
}

// Collection is the replacement for [Registry].
//
// It groups items and provides helpers. See [Item] and [Registry].
type Collection struct {
	Entries map[string]Item
}

// Add inserts an [Item] into the collection.
//
// It demonstrates declaration comments with doc links to [Item].
func (c *Collection) Add(it Item) {
	if c.Entries == nil {
		c.Entries = make(map[string]Item)
	}
	c.Entries[it.SKU] = it
}

// Lookup retrieves an item by SKU. It links to [Item] and [Collection].
func (c *Collection) Lookup(sku string) (Item, bool) {
	it, ok := c.Entries[sku]
	return it, ok
}

// Count returns the number of entries. See [Collection.Add].
func (c *Collection) Count() int {
	return len(c.Entries)
}

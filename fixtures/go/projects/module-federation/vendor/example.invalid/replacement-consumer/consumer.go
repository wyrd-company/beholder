package consumer

import "example.invalid/replacement-target/target"

// Value shows that the main module's replacement wins over this dependency's.
func Value() string { return target.Value() }

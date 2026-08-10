package secret

import _ "unsafe"

//go:linkname hiddenValue example.com/portablebridge/internal/secret.hiddenValue
func hiddenValue() string { return "linked" }

// ExportedValue is the ordinary exported alternative to the linkname path.
func ExportedValue() string { return hiddenValue() }

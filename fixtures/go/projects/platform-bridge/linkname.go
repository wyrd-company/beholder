package bridge

import (
	_ "example.com/portablebridge/internal/secret"
	_ "unsafe"
)

//go:linkname linkedValue example.com/portablebridge/internal/secret.hiddenValue
func linkedValue() string

// LinkedValue reaches the self-contained object symbol through linkname.
func LinkedValue() string { return linkedValue() }

package visibility

import "example.invalid/module-federation/internal/visibility/internal/leaf"

const privateMarker = "private"

// PublicMarker is exported, while the package remains protected by its path.
func PublicMarker() string { return leaf.Marker() + ":" + privateMarker }

func privateValue() string { return privateMarker }

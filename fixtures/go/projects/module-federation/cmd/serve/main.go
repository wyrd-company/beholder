package main

import (
	"fmt"

	"example.invalid/module-federation/internal/visibility"
	"example.invalid/module-federation/pkg/stdlib"
	"example.invalid/module-federation/src/plain"
)

func main() {
	legacy, modern := stdlib.Samples()
	fmt.Println(visibility.PublicMarker(), plain.Name(), legacy, modern)
}

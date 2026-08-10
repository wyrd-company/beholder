package main

import (
	"fmt"

	"example.com/portablebridge"
)

// BuildLabel is eligible for -ldflags -X injection in each build mode.
var BuildLabel = "default"

func main() {
	snapshot := bridge.SnapshotFor()
	fmt.Printf("%s %s %s %s %s %s\n", BuildLabel, snapshot.OS, snapshot.Architecture, snapshot.Compiler, snapshot.Cgo, snapshot.Endpoint)
}

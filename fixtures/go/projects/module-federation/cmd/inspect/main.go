package main

import (
	"fmt"

	"example.invalid/module-federation/pkg/federation"
	"example.invalid/module-federation/pkg/identity"
)

func main() {
	snapshot := federation.SnapshotValues()
	fmt.Println(identity.PackagePath, snapshot.GraphVersion, snapshot.V1Name, snapshot.V2Name)
}

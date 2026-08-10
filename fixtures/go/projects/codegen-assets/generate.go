package codegenassets

//go:generate go run ./cmd/geninventory -input schema/inventory.json -output generated/inventory_gen.go
//go:generate go run ./cmd/geninventory -input "schema/inventory.json" -output generated/inventory_gen.go -pkg $GOPACKAGE
//go:generate go run ./cmd/geninfo -input schema/info.json -output generated/info_gen.go

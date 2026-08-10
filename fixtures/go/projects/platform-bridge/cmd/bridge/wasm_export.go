//go:build wasm

package main

//go:wasmexport command_export
func commandExport(value uint32) uint32 { return value + 2 }

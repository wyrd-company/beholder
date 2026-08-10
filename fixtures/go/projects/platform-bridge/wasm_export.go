//go:build wasm

package bridge

//go:wasmexport bridge_export
func wasmExport(value uint32) uint32 { return value + 1 }

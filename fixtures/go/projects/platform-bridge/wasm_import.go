//go:build wasm

package bridge

//go:wasmimport fixture_host add
func importedAdd(a, b uint32) uint32

// WasmImportedBlend retains a wasm import declaration without selecting it for
// native targets.
func WasmImportedBlend(a, b uint32) uint32 { return importedAdd(a, b) }

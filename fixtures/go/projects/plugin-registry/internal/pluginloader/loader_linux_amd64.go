//go:build linux && amd64

package pluginloader

import (
	"fmt"
	"path/filepath"
	"plugin"
)

// Report records symbol resolution and duplicate-open observations.
type Report struct {
	Version        string
	CallResult     string
	DuplicateOpen  bool
	InitCount      int
	MissingLookup  bool
	WrongTypeCheck bool
}

// Load resolves names supplied by the host and opens one plugin twice.
func Load(path, functionName, variableName string) (Report, error) {
	absolutePath, err := filepath.Abs(path)
	if err != nil {
		return Report{}, fmt.Errorf("make plugin path absolute: %w", err)
	}

	first, err := plugin.Open(absolutePath)
	if err != nil {
		return Report{}, fmt.Errorf("open plugin: %w", err)
	}
	second, err := plugin.Open(absolutePath)
	if err != nil {
		return Report{}, fmt.Errorf("open plugin twice: %w", err)
	}

	functionSymbol, err := first.Lookup(functionName)
	if err != nil {
		return Report{}, fmt.Errorf("lookup function %q: %w", functionName, err)
	}
	function, ok := functionSymbol.(func(string) string)
	if !ok {
		return Report{}, fmt.Errorf("function %q has unexpected type", functionName)
	}
	_, wrongTypeCheck := functionSymbol.(func(int) string)

	variableSymbol, err := first.Lookup(variableName)
	if err != nil {
		return Report{}, fmt.Errorf("lookup variable %q: %w", variableName, err)
	}
	version, ok := variableSymbol.(*string)
	if !ok {
		return Report{}, fmt.Errorf("variable %q has unexpected type", variableName)
	}

	missingLookup := false
	if _, err := first.Lookup("MissingSymbol"); err != nil {
		missingLookup = true
	}
	initCount := 0
	if symbol, err := first.Lookup("InitCount"); err == nil {
		if count, ok := symbol.(*int); ok {
			initCount = *count
		}
	}

	return Report{
		Version:        *version,
		CallResult:     function("seed"),
		DuplicateOpen:  first == second,
		InitCount:      initCount,
		MissingLookup:  missingLookup,
		WrongTypeCheck: !wrongTypeCheck,
	}, nil
}

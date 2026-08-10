//go:build !linux || !amd64

package pluginloader

import "fmt"

// Report records that plugin support was not compiled for this host.
type Report struct{}

// Load preserves the host API on unsupported targets.
func Load(path, functionName, variableName string) (Report, error) {
	return Report{}, fmt.Errorf("plugins are unsupported on this host")
}

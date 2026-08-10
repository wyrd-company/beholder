package api

import "example.invalid/workspace/alpha/api"

// Name creates a source edge from beta back to alpha.
func Name() string { return "beta-api:" + api.Name() }

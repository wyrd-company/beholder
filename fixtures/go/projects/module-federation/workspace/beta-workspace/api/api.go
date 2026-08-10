package api

import "example.invalid/workspace/alpha/api"

// Name is selected by the workspace-level replacement.
func Name() string { return "beta-workspace:" + api.Name() }

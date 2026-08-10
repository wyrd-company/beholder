package bridge

import "example.invalid/workspace/beta/api"

// Name resolves through the workspace replacement when the workspace is on.
func Name() string { return "alpha-bridge:" + api.Name() }

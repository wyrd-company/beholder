package api

import "example.invalid/workspace/alpha/api"

// Name is the off-workspace replacement variant.
func Name() string { return "beta-local:" + api.Name() }

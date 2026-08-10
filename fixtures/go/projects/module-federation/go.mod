module example.invalid/module-federation

go 1.24

toolchain go1.26.5

require (
	example.invalid/graph-root v1.0.0
	example.invalid/nested v0.1.0
	example.invalid/replacement-consumer v1.0.0
	example.invalid/retracted v1.1.0
	example.invalid/semver v1.4.0
	example.invalid/semver/v2 v2.3.0
	example.invalid/testonly v1.0.0
	example.invalid/toolkit v0.1.0
)

require (
	example.invalid/graphdep v1.2.0 // indirect
	example.invalid/replacement-target v1.0.0 // indirect
)

tool example.invalid/toolkit/cmd/forge

exclude example.invalid/retracted v1.0.0

replace (
	example.invalid/graph-root v1.0.0 => ./deps/graph-root
	example.invalid/graphdep => ./deps/graphdep-high
	example.invalid/nested v0.1.0 => ./nested-module
	example.invalid/replacement-consumer v1.0.0 => ./deps/replacement-consumer
	example.invalid/replacement-target v1.0.0 => ./deps/replacement-local
	example.invalid/retracted v1.1.0 => ./deps/retracted
	example.invalid/semver v1.4.0 => ./deps/semver-v1
	example.invalid/semver/v2 v2.3.0 => ./deps/semver-v2
	example.invalid/testonly v1.0.0 => ./deps/testonly
	example.invalid/toolkit v0.1.0 => ./deps/toolkit
)

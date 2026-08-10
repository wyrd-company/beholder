module example.invalid/mvs-probe

go 1.24

require example.invalid/graph-root v1.0.0

require example.invalid/graphdep v1.0.0 // indirect

replace (
	example.invalid/graph-root v1.0.0 => ../deps/graph-root
	example.invalid/graphdep => ../deps/graphdep-high
)

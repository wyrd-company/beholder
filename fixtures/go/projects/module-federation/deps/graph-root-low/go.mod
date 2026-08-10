module example.invalid/graph-root-low

go 1.20

require example.invalid/graphdep v1.0.0

replace example.invalid/graphdep v1.0.0 => ../graphdep-low

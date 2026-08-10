module example.invalid/graph-root

go 1.20

require example.invalid/graphdep v1.2.0

replace example.invalid/graphdep v1.2.0 => ../graphdep-high

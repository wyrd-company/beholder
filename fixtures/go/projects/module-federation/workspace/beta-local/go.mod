module example.invalid/workspace/beta

go 1.24

require example.invalid/workspace/alpha v0.1.0

replace example.invalid/workspace/alpha v0.1.0 => ../alpha

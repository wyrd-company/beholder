module example.invalid/workspace/alpha

go 1.24

require example.invalid/workspace/beta v0.1.0

replace example.invalid/workspace/beta v0.1.0 => ../beta-local

module example.invalid/replacement-consumer

go 1.20

require example.invalid/replacement-target v1.0.0

replace example.invalid/replacement-target v1.0.0 => ../replacement-upstream

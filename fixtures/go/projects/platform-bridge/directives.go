package bridge

//go:nosplit
func noSplitValue(value uint32) uint32 { return value + 1 }

//go:noinline
func noInlineValue(value uint32) uint32 { return value + 2 }

//go:norace
func noRaceValue(value uint32) uint32 { return value + 3 }

// DirectiveValue keeps the accepted pragmas in the source call graph.
func DirectiveValue(value uint32) uint32 {
	return noRaceValue(noInlineValue(noSplitValue(value)))
}

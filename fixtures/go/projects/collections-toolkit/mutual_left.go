package toolkit

// MutualLeft and MutualRight form legal mutual recursion across source files.
type MutualLeft struct {
	Right *MutualRight
}

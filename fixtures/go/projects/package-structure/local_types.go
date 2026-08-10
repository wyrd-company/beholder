package pantry

// ---
// relationships:
//   references:
//     - coverage
// ---

type PackagePoint struct {
	X int
	Y int
}

type PackagePointAlias = PackagePoint

func localNamedTypes(value int) int {
	type localPoint PackagePoint
	local := localPoint{X: value, Y: value + 1}
	packageValue := PackagePoint(local)
	aliasValue := PackagePointAlias(packageValue)
	return local.X + packageValue.X + aliasValue.Y
}

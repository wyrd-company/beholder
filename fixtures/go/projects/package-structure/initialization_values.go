package pantry

// ---
// relationships:
//   references:
//     - coverage
// ---

var initTrace []string

var directSource = recordInitialization("direct")
var directValue = directSource + " value"
var throughValue = initializeThroughFunction()
var independentValue = "independent"

// cycleLeft reads cycleRight's zero value; init assigns cycleRight after all
// package variables have been initialized.
var cycleLeft = cycleRight
var cycleRight int

func directInitializationSummary() string {
	return directValue + "/" + throughValue + "/" + independentValue
}

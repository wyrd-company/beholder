package pantry

// ---
// relationships:
//   references:
//     - coverage
// ---

const LooseCount = 4

const (
	GroupedCount  = LooseCount + 2
	groupedOffset = 3
)

var looseLabel = "sunny"

var (
	groupedLabel  = looseLabel + " shelf"
	GroupedPhrase = "shared package scope"
)

type LooseRecord struct {
	Name  string
	Count int
}

type (
	GroupedRecord struct {
		Name string
	}
	Earlier = Later
)

// BeforeDeclaration uses a package declaration that appears later in this
// file. Package scope is not ordered like a function block.
func BeforeDeclaration() string {
	return AfterDeclaration()
}

func mutualLeft(value string) string {
	return mutualRight(value)
}

func mutualRight(value string) string {
	return value + "!"
}

func AfterDeclaration() string {
	return mutualLeft("ready")
}

type Later struct {
	Value int
}

func groupedSummary() string {
	return groupedLabel + " " + GroupedPhrase
}

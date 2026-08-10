package pantry

// ---
// relationships:
//   references:
//     - coverage
// ---

// Shelf is an exported record with both exported and unexported members.
type Shelf struct {
	Public  string
	private string
}

type shelf struct {
	value string
}

// Étage is exported because its first rune is an uppercase Unicode letter.
type Étage struct {
	Name string
}

type πράγμα struct {
	name string
}

var PublicValue = "visible"
var privateValue = "hidden"

// PublicFunction returns the public face of a shelf.
func PublicFunction(s Shelf) string {
	return s.Open()
}

func privateFunction() string {
	return privateValue
}

// Open returns the shelf's exported view.
func (s Shelf) Open() string {
	return s.Public
}

func (s Shelf) close() string {
	return s.private
}

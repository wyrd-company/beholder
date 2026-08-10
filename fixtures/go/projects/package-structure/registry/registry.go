package registry

// ---
// relationships:
//   references:
//     - coverage
// ---

var entries []string

// Ready becomes true during package initialization.
var Ready bool

func Register(name string) {
	entries = append(entries, name)
}

func Snapshot() []string {
	return append([]string(nil), entries...)
}

func init() {
	Register("registry")
	Ready = true
}

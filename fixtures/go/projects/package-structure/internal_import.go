package pantry

// ---
// relationships:
//   references:
//     - coverage
// ---

import "example.com/fixture/package-structure/internal/notes"

func internalToken(text string) notes.Token {
	return notes.New(text)
}

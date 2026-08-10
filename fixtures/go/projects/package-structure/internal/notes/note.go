package notes

// ---
// relationships:
//   references:
//     - coverage
// ---

// Token is exported from an internal package for its permitted parent.
type Token struct {
	text string
}

func New(text string) Token {
	return Token{text: text}
}

func (t Token) Text() string {
	return t.text
}

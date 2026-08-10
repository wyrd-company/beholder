package label

// ---
// relationships:
//   references:
//     - coverage
// ---

type Badge struct {
	tone string
}

func New(tone string) Badge {
	return Badge{tone: tone}
}

func (b Badge) Name() string {
	return b.tone
}

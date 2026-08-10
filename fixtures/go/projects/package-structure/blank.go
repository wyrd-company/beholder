package pantry

// ---
// relationships:
//   references:
//     - coverage
// ---

type Parcel struct {
	_    int
	Code string
}

type shelfOpener interface {
	Open() string
}

var _ shelfOpener = Shelf{}

func typedPair() (int, error) {
	return 7, nil
}

func blankAssignments() int {
	value, err := typedPair()
	_ = value
	_ = err
	return 1
}

func blankReturns() (_ int, _ error) {
	return 7, nil
}

func blankRange(values []int) int {
	total := 0
	for _, value := range values {
		total += value
	}
	for index, _ := range values {
		total += index
	}
	return total
}

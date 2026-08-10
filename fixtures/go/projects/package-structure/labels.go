package pantry

// ---
// relationships:
//   references:
//     - coverage
// ---

func labelDance() int {
	label := 0

label:
	for step := 0; step < 3; step++ {
		if step == 0 {
			continue label
		}
		label += step
	}

breakLabel:
	for {
		break breakLabel
	}

	inner := func() int {
		innerValue := 1
	innerLabel:
		for innerValue < 2 {
			innerValue++
			continue innerLabel
		}
		return innerValue
	}

	label += inner()
	goto finish

finish:
	return label
}

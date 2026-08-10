//go:build vetfixtures

package vetfixtures

func loopClosureTarget() {
	for _, value := range []string{"red", "blue"} {
		defer func() { println(value) }()
	}
}

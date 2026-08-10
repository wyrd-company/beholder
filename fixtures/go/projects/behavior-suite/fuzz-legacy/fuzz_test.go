package fuzzparcel

import "testing"

func FuzzMirror(f *testing.F) {
	f.Add("alpha")
	f.Add("bravo")
	f.Fuzz(func(t *testing.T, input string) {
		if got := Mirror(input); got != input {
			t.Fatalf("mirror(%q) = %q", input, got)
		}
	})
}

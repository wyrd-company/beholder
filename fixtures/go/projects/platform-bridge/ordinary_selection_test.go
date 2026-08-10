package bridge

import "testing"

// TestOrdinarySelection is selected by go test but ignored by go build.
func TestOrdinarySelection(t *testing.T) {
	if NativeWordBits == 0 {
		t.Fatal("native word size is not selected")
	}
}

package graph

import (
	"testing"

	"example.invalid/testonly/testsupport"
)

func TestTestOnlyDependency(t *testing.T) {
	if got := testsupport.Marker(); got != "test-only" {
		t.Fatalf("test-only dependency marker = %q", got)
	}
}

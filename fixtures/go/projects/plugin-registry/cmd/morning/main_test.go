package main

import "testing"

func TestMainPackageIsTestable(t *testing.T) {
	if commandName() != "morning" {
		t.Fatalf("unexpected command name: %s", commandName())
	}
}

package support

import "example.com/parcel"

// Snapshot reads a value through a project-local support package.
func Snapshot(counter parcel.Counter) int {
	return counter.Value()
}

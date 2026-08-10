package parcel_test

import (
	"fmt"
	"testing"

	"example.com/parcel"
	"example.com/parcel/support"
)

func TestExternalPackage(t *testing.T) {
	counter := parcel.NewCounter(4)
	counter.Increment()
	if got, want := support.Snapshot(counter), 5; got != want {
		t.Fatalf("snapshot = %d, want %d", got, want)
	}
}

func TestExternalPackageGraph(t *testing.T) {
	chain := &parcel.Node{Value: 4, Next: &parcel.Node{Value: 5}}
	if got, want := parcel.Sum(chain), 9; got != want {
		t.Fatalf("sum = %d, want %d", got, want)
	}
}

func Example() {
	fmt.Println(parcel.Add(2, 3))
	// Output: 5
}

func ExampleCounter() {
	counter := parcel.NewCounter(4)
	fmt.Println(counter.Value())
	// Output: 4
}

func ExampleAdd() {
	fmt.Println(parcel.Add(4, 5))
	// Output: 9
}

func ExampleCounter_Increment() {
	counter := parcel.NewCounter(3)
	counter.Increment()
	fmt.Println(counter.Value())
	// Output: 4
}

func ExampleAdd_unordered() {
	fmt.Println("red")
	fmt.Println("blue")
	// Unordered output:
	// blue
	// red
}

func ExampleCounter_documentation() {
	counter := parcel.NewCounter(8)
	fmt.Println(counter.Value())
}

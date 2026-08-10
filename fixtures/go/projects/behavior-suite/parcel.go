package parcel

var initialized bool

func init() {
	initialized = true
}

// Counter stores a small mutable count.
type Counter struct {
	value int
}

// NewCounter creates a counter with an initial value.
func NewCounter(start int) Counter {
	return Counter{value: start}
}

// Add combines two values.
func Add(left, right int) int {
	return left + right
}

// Increment advances the counter.
func (c *Counter) Increment() {
	c.value++
}

// Value reports the current count.
func (c Counter) Value() int {
	return c.value
}

var defaultStart = 2

func seededCounter() Counter {
	return NewCounter(defaultStart)
}

func packageInitialized() bool {
	return initialized
}

// Node is a recursive linked value.
type Node struct {
	Value int
	Next  *Node
}

// Sum recursively totals a linked value.
func Sum(node *Node) int {
	if node == nil {
		return 0
	}
	return node.Value + Sum(node.Next)
}

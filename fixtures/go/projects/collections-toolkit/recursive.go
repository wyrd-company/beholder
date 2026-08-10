package toolkit

// LinkedNode recurses through a pointer.
type LinkedNode struct {
	Value int
	Next  *LinkedNode
}

// SliceNode and MapNode recurse through collection indirection.
type SliceNode struct {
	Children []SliceNode
}

type MapNode struct {
	Children map[string]MapNode
}

// FunctionNode and InterfaceNode recurse through function and interface types.
type FunctionNode struct {
	Next func() FunctionNode
}

type InterfaceNode interface {
	Next() InterfaceNode
}

// GenericNode demonstrates recursive generic declaration indirection.
type GenericNode[T any] struct {
	Value T
	Next  *GenericNode[T]
}

type RecursivePair struct {
	Left  *RecursivePair
	Right []RecursivePair
}

func NewRecursiveNode(value int) *LinkedNode {
	return &LinkedNode{Value: value}
}

func LinkGeneric[T any](value T) *GenericNode[T] {
	return &GenericNode[T]{Value: value}
}

func RecursiveFunction(node *LinkedNode) int {
	if node == nil {
		return 0
	}
	if node.Next == nil {
		return node.Value
	}
	return node.Value + RecursiveFunction(node.Next)
}

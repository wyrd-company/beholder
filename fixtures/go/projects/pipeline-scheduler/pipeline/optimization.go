package pipeline

import (
	"bytes"
	"runtime"
	"sync"
)

var initializationTrace = []string{}
var firstIndependentValue = initializationMarker("first")
var secondIndependentValue = initializationMarker("second")

func initializationMarker(value string) string {
	initializationTrace = append(initializationTrace, value)
	return value
}

func init() {
	initializationTrace = append(initializationTrace, "init")
}

// InitializationTrace exposes independent package initialization edges
// without claiming an order for independent expressions.
func InitializationTrace() []string {
	return append([]string{}, initializationTrace...)
}

// MapIterationOrder returns the observed map order. Callers must admit every
// order permitted by the language and runtime.
func MapIterationOrder(values map[string]int) []string {
	order := make([]string, 0, len(values))
	for key := range values {
		order = append(order, key)
	}
	return order
}

// SchedulingOrder returns the order in which two released goroutines report.
func SchedulingOrder() []string {
	release := make(chan struct{})
	results := make(chan string, 2)
	var started sync.WaitGroup
	started.Add(2)
	go func() {
		started.Done()
		<-release
		results <- "first"
	}()
	go func() {
		started.Done()
		<-release
		results <- "second"
	}()
	started.Wait()
	close(release)
	return []string{<-results, <-results}
}

type finalizerMarker struct {
	done chan struct{}
}

func finalizeMarker(marker *finalizerMarker) {
	close(marker.done)
}

// FinalizerObservation registers a finalizer and keeps the target reachable
// through registration. The returned notification may never close before
// process exit.
func FinalizerObservation() <-chan struct{} {
	marker := &finalizerMarker{done: make(chan struct{})}
	runtime.SetFinalizer(marker, finalizeMarker)
	runtime.KeepAlive(marker)
	return marker.done
}

func evaluationMarker(value int) int {
	return value
}

// IndependentArguments evaluates independent arguments and returns a value
// that does not depend on their relative evaluation order.
func IndependentArguments() int {
	return evaluationMarker(1) + evaluationMarker(2)
}

// EscapingBuffer returns a pointer to a local allocation, making its lifetime
// escape the function regardless of stack or heap placement decisions.
func EscapingBuffer() *bytes.Buffer {
	buffer := &bytes.Buffer{}
	buffer.WriteString("payload")
	return buffer
}

// InlineCandidate is intentionally small enough to be considered for inlining
// without making inlining a portable assertion.
func InlineCandidate(value int) int {
	return value + 1
}

type Reducer interface {
	Reduce(int) int
}

type incrementReducer struct {
	amount int
}

func (r incrementReducer) Reduce(value int) int {
	return value + r.amount
}

// InterfaceReduction is an interface call with one concrete implementation;
// the compiler may devirtualize it under suitable optimization settings.
func InterfaceReduction(value int) int {
	var reducer Reducer = incrementReducer{amount: 1}
	return reducer.Reduce(value)
}

func Identity[T any](value T) T {
	return value
}

func Pair[T any](left, right T) [2]T {
	return [2]T{left, right}
}

// GenericInstantiations gives the compiler multiple concrete instantiations
// without making generated function names or symbols part of the fixture.
func GenericInstantiations() ([2]int, [2]string) {
	return Pair(Identity(1), Identity(2)), Pair(Identity("left"), Identity("right"))
}

const featureEnabled = false

func deadPath(value int) int {
	return value * value
}

// DeadCodeBoundary has a compile-time unreachable branch. Its behavior is the
// returned value, not any optimizer-specific object or symbol.
func DeadCodeBoundary(value int) int {
	if featureEnabled {
		return deadPath(value)
	}
	return value
}

// OptimizationObservation gathers portable values while leaving map order,
// scheduling, finalizer timing, evaluation order, and optimizer choices open.
type OptimizationObservation struct {
	MapOrder       []string
	SelectBranch   string
	ScheduleOrder  []string
	IndependentSum int
}

func OptimizationObservationFor(values map[string]int) OptimizationObservation {
	return OptimizationObservation{
		MapOrder:       MapIterationOrder(values),
		SelectBranch:   ReadySelectChoice(),
		ScheduleOrder:  SchedulingOrder(),
		IndependentSum: IndependentArguments(),
	}
}

package pipeline

import (
	"sync"
	"sync/atomic"
)

// SynchronizedState contains one value reached through each standard
// happens-before mechanism used by the pipeline.
type SynchronizedState struct {
	mu          sync.Mutex
	once        sync.Once
	workers     sync.WaitGroup
	value       int
	atomicValue atomic.Int64
}

// PublishOverChannel writes before a receive observes the send.
func (s *SynchronizedState) PublishOverChannel(value int) int {
	handoff := make(chan struct{})
	go func() {
		s.value = value
		handoff <- struct{}{}
	}()
	<-handoff
	return s.value
}

// PublishByClose writes before close, and the receive that observes a closed
// channel is the synchronization edge for the following read.
func (s *SynchronizedState) PublishByClose(value int) int {
	closed := make(chan struct{})
	go func() {
		s.value = value
		close(closed)
	}()
	<-closed
	return s.value
}

// ProtectWithMutex gives the read and write a mutex happens-before edge.
func (s *SynchronizedState) ProtectWithMutex(value int) int {
	s.mu.Lock()
	s.value = value
	s.mu.Unlock()
	s.mu.Lock()
	defer s.mu.Unlock()
	return s.value
}

// InitializeOnce publishes the value through sync.Once's completion contract.
func (s *SynchronizedState) InitializeOnce(value int) int {
	s.once.Do(func() { s.value = value })
	return s.value
}

// JoinWorkers waits for the write goroutine before reading its ordinary field.
func (s *SynchronizedState) JoinWorkers(value int) int {
	s.workers.Add(1)
	go func() {
		defer s.workers.Done()
		s.value = value
	}()
	s.workers.Wait()
	return s.value
}

// AtomicSnapshot uses typed atomic store and load as both the operation and
// synchronization edge.
func (s *SynchronizedState) AtomicSnapshot(value int64) int64 {
	s.atomicValue.Store(value)
	return s.atomicValue.Load()
}

// BufferedCapacityBoundary records that a send into a channel with spare
// capacity completes before a receiver is required to take the value. It does
// not use that fact as a substitute for a receive synchronization edge.
func BufferedCapacityBoundary() (capacity, buffered int) {
	values := make(chan struct{}, 1)
	values <- struct{}{}
	return cap(values), len(values)
}

// UnsynchronizedState intentionally exposes ordinary read and write methods.
// Calling them from separate goroutines without an external edge is a data
// race; the methods are kept next to the synchronized alternatives as a
// counterexample.
type UnsynchronizedState struct {
	Value int
}

func (s *UnsynchronizedState) Write(value int) {
	s.Value = value
}

func (s *UnsynchronizedState) Read() int {
	return s.Value
}

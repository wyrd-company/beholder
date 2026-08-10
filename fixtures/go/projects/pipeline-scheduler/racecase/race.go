package racecase

import "sync"

// UnsafeCounter is intentionally unsynchronized. Concurrent Add calls race on
// Value when RaceExample is invoked.
type UnsafeCounter struct {
	Value int
}

func (c *UnsafeCounter) Add() {
	c.Value++
}

func RaceExample() int {
	var counter UnsafeCounter
	var workers sync.WaitGroup
	workers.Add(2)
	go func() {
		defer workers.Done()
		counter.Add()
	}()
	go func() {
		defer workers.Done()
		counter.Add()
	}()
	workers.Wait()
	return counter.Value
}

// SafeCounter is the synchronized near-neighbor of UnsafeCounter.
type SafeCounter struct {
	mu    sync.Mutex
	Value int
}

func (c *SafeCounter) Add() {
	c.mu.Lock()
	c.Value++
	c.mu.Unlock()
}

func SynchronizedExample() int {
	var counter SafeCounter
	var workers sync.WaitGroup
	workers.Add(2)
	go func() {
		defer workers.Done()
		counter.Add()
	}()
	go func() {
		defer workers.Done()
		counter.Add()
	}()
	workers.Wait()
	return counter.Value
}

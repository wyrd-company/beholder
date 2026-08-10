// Package pipeline provides a small concurrent worker pipeline.
package pipeline

import (
	"context"
	"errors"
	"sort"
	"sync"
	"sync/atomic"
)

// Job is one unit of work accepted by Scheduler.
type Job struct {
	Name string
	Run  func(context.Context) error
}

// Scheduler runs jobs with a fixed set of workers.
type Scheduler struct {
	workers  int
	started  atomic.Bool
	once     sync.Once
	registry *CallbackRegistry
}

// NewScheduler creates a scheduler. A non-positive worker count uses one worker.
func NewScheduler(workers int, registry *CallbackRegistry) *Scheduler {
	if workers <= 0 {
		workers = 1
	}
	return &Scheduler{workers: workers, registry: registry}
}

// Run starts workers, waits for every worker, and returns completion names in a
// stable order. Worker completion order itself remains unconstrained.
func (s *Scheduler) Run(ctx context.Context, jobs []Job) ([]string, error) {
	if ctx == nil {
		ctx = context.Background()
	}

	s.once.Do(func() { s.started.Store(true) })
	queue := make(chan Job)
	completed := make(chan completion, len(jobs))
	var workers sync.WaitGroup

	for index := 0; index < s.workers; index++ {
		workers.Add(1)
		go runWorker(ctx, index, queue, completed, &workers, s.registry)
	}

	go func() {
		defer close(queue)
		for _, job := range jobs {
			select {
			case queue <- job:
			case <-ctx.Done():
				return
			}
		}
	}()

	workers.Wait()
	close(completed)

	completedNames := make([]string, 0, len(jobs))
	errs := make([]error, 0)
	for result := range completed {
		if result.err != nil {
			errs = append(errs, result.err)
			continue
		}
		completedNames = append(completedNames, result.name)
	}
	sort.Strings(completedNames)
	if err := contextError(ctx, errs); err != nil {
		return completedNames, err
	}
	return completedNames, errors.Join(errs...)
}

// Started reports whether Run has been called at least once.
func (s *Scheduler) Started() bool {
	return s.started.Load()
}

func contextError(ctx context.Context, jobErrors []error) error {
	if ctx.Err() != nil {
		return errors.Join(context.Cause(ctx), errors.Join(jobErrors...))
	}
	return nil
}

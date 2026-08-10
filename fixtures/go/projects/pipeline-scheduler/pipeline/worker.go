package pipeline

import (
	"context"
	"fmt"
	"sync"
	"sync/atomic"
)

type completion struct {
	name string
	err  error
}

type worker struct {
	index int
}

func runWorker(ctx context.Context, index int, jobs <-chan Job, completed chan<- completion, workers *sync.WaitGroup, registry *CallbackRegistry) {
	defer workers.Done()
	worker{index: index}.process(ctx, jobs, completed, registry)
}

func (w worker) process(ctx context.Context, jobs <-chan Job, completed chan<- completion, registry *CallbackRegistry) {
	for {
		select {
		case <-ctx.Done():
			return
		case job, ok := <-jobs:
			if !ok {
				return
			}
			if job.Run == nil {
				completed <- completion{name: job.Name, err: fmt.Errorf("worker %d: %w", w.index, ErrMissingWork)}
				continue
			}
			err := job.Run(ctx)
			completed <- completion{name: job.Name, err: err}
			if registry != nil && err == nil {
				registry.Dispatch(job.Name)
			}
		}
	}
}

var argumentEvaluations atomic.Int64

func evaluatedArgument(label string) string {
	argumentEvaluations.Add(1)
	return label
}

func namedWorker(label string, results chan<- string, workers *sync.WaitGroup) {
	defer workers.Done()
	results <- label
}

type methodWorker struct {
	prefix string
}

func (w methodWorker) run(label string, results chan<- string, workers *sync.WaitGroup) {
	defer workers.Done()
	results <- w.prefix + label
}

// LaunchExamples exercises named functions, closures, and methods. The result
// is sorted only after every goroutine has joined, so scheduling and completion
// order are not used as a semantic result.
func LaunchExamples() []string {
	results := make(chan string, 4)
	var workers sync.WaitGroup

	workers.Add(1)
	go namedWorker(evaluatedArgument("named"), results, &workers)

	for _, label := range []string{"closure-a", "closure-b"} {
		label := label
		workers.Add(1)
		go func() {
			defer workers.Done()
			results <- label
		}()
	}

	workers.Add(1)
	go methodWorker{prefix: "method-"}.run(evaluatedArgument("argument"), results, &workers)

	workers.Wait()
	close(results)
	values := make([]string, 0, cap(results))
	for value := range results {
		values = append(values, value)
	}
	sortStrings(values)
	return values
}

// LaunchRecoveredPanic shows that recovery belongs to the goroutine in which
// the panic occurs. The returned channel closes after that goroutine recovers.
func LaunchRecoveredPanic() <-chan struct{} {
	done := make(chan struct{})
	go func() {
		defer close(done)
		defer func() { _ = recover() }()
		panic("worker failure")
	}()
	return done
}

func sortStrings(values []string) {
	for left := 0; left < len(values); left++ {
		for right := left + 1; right < len(values); right++ {
			if values[right] < values[left] {
				values[left], values[right] = values[right], values[left]
			}
		}
	}
}

package pipeline

import "sync/atomic"

// SelectSnapshot captures expression evaluation and the selected branch. The
// two ready branches are deliberately allowed to produce different outcomes.
type SelectSnapshot struct {
	Evaluations int64
	Branch      string
	NilDisabled bool
}

func selectInput(values <-chan string, evaluations *atomic.Int64) <-chan string {
	evaluations.Add(1)
	return values
}

func selectOutput(values chan<- string, evaluations *atomic.Int64) chan<- string {
	evaluations.Add(1)
	return values
}

func selectValue(evaluations *atomic.Int64) string {
	evaluations.Add(1)
	return "sent"
}

// SelectSnapshotExample has a ready receive and a ready buffered send. The
// default case is present for the no-ready-case boundary.
func SelectSnapshotExample() SelectSnapshot {
	input := make(chan string, 1)
	output := make(chan string, 1)
	input <- "received"
	var evaluations atomic.Int64
	snapshot := SelectSnapshot{}
	select {
	case <-selectInput(input, &evaluations):
		snapshot.Branch = "receive"
	case selectOutput(output, &evaluations) <- selectValue(&evaluations):
		snapshot.Branch = "send"
	default:
		snapshot.Branch = "default"
	}
	snapshot.Evaluations = evaluations.Load()
	snapshot.NilDisabled = NilChannelDisabled()
	return snapshot
}

// ReadySelectChoice admits either ready branch. Closed channels remain ready,
// so this function does not promise a stable result.
func ReadySelectChoice() string {
	left := make(chan struct{})
	right := make(chan struct{})
	close(left)
	close(right)
	select {
	case <-left:
		return "left"
	case <-right:
		return "right"
	}
}

// EmptySelectBlocks is the empty select boundary and blocks forever when
// called.
func EmptySelectBlocks() {
	select {}
}

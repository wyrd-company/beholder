package pipeline

// ChannelSnapshot records observations that are defined by channel semantics,
// including the zero value returned by a closed receive.
type ChannelSnapshot struct {
	Capacity        int
	Values          []int
	ClosedValue     int
	ClosedOK        bool
	NilCaseDisabled bool
}

// BufferedChannelSnapshot sends into a buffered channel, closes it, ranges
// over it, and performs a comma-ok receive after the range has ended.
func BufferedChannelSnapshot() ChannelSnapshot {
	values := make(chan int, 2)
	values <- 3
	values <- 5
	snapshot := ChannelSnapshot{Capacity: cap(values)}
	close(values)
	for value := range values {
		snapshot.Values = append(snapshot.Values, value)
	}
	snapshot.ClosedValue, snapshot.ClosedOK = <-values
	snapshot.NilCaseDisabled = NilChannelDisabled()
	return snapshot
}

func sendDirectional(values chan<- string, value string) {
	values <- value
}

func receiveDirectional(values <-chan string) string {
	return <-values
}

// DirectionalExchange keeps send-only and receive-only channel types at the
// boundary while an unbuffered channel supplies the rendezvous.
func DirectionalExchange() string {
	values := make(chan string)
	done := make(chan struct{})
	var received string
	go func() {
		received = receiveDirectional(values)
		close(done)
	}()
	sendDirectional(values, "handoff")
	close(values)
	<-done
	return received
}

// NilChannelBlocks contains a receive that cannot proceed until the nil
// channel is replaced. It is intentionally a library example and is not
// called by the command package.
func NilChannelBlocks() {
	var values chan int
	<-values
}

// NilChannelDisabled shows that a nil channel case is disabled and default is
// selected instead of blocking.
func NilChannelDisabled() bool {
	var values <-chan int
	select {
	case <-values:
		return false
	default:
		return true
	}
}

func catchClosedSend(values chan int) (panicked bool) {
	defer func() {
		panicked = recover() != nil
	}()
	values <- 1
	return false
}

func catchDoubleClose(values chan int) (panicked bool) {
	defer func() {
		panicked = recover() != nil
	}()
	close(values)
	return false
}

// ClosedChannelPanics quarantines the two operations that panic after close.
// The recover calls make the examples valid when a caller chooses to inspect
// them.
func ClosedChannelPanics() (sendPanicked, closePanicked bool) {
	values := make(chan int)
	close(values)
	return catchClosedSend(values), catchDoubleClose(values)
}

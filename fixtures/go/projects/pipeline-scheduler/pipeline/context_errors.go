package pipeline

import (
	"context"
	"errors"
	"fmt"
	"time"
)

var (
	ErrStopped     = errors.New("pipeline stopped")
	ErrDeadline    = errors.New("pipeline deadline")
	ErrMissingWork = errors.New("missing work function")
)

// StageError is a typed, wrapped error that keeps the stage identity in the
// chain while preserving its cause for errors.Is and errors.As.
type StageError struct {
	Stage string
	Err   error
}

func (e *StageError) Error() string {
	return fmt.Sprintf("stage %s: %v", e.Stage, e.Err)
}

func (e *StageError) Unwrap() error {
	return e.Err
}

// CodeError supplies custom Is and As behavior in addition to a single unwrap
// edge.
type CodeError struct {
	Code string
	Err  error
}

func (e *CodeError) Error() string {
	return fmt.Sprintf("code %s: %v", e.Code, e.Err)
}

func (e *CodeError) Is(target error) bool {
	other, ok := target.(*CodeError)
	return ok && e.Code == other.Code
}

func (e *CodeError) As(target any) bool {
	destination, ok := target.(**CodeError)
	if !ok {
		return false
	}
	*destination = e
	return true
}

func (e *CodeError) Unwrap() error {
	return e.Err
}

// JoinedError exposes multiple unwrap paths through the errors.Join contract.
type JoinedError struct {
	Message string
	Errors  []error
}

func (e *JoinedError) Error() string {
	return e.Message
}

func (e *JoinedError) Unwrap() []error {
	return e.Errors
}

// BuildErrorChain combines sentinel identity, typed errors, wrapping, joining,
// and custom matching in one ordinary pipeline failure.
func BuildErrorChain() error {
	typed := &StageError{Stage: "mix", Err: ErrStopped}
	coded := &CodeError{Code: "retryable", Err: typed}
	wrapped := fmt.Errorf("worker failed: %w", coded)
	return errors.Join(wrapped, ErrDeadline)
}

// ErrorIdentity contrasts errors.Is identity with equal text. A wrapped
// sentinel may match identity while its complete message differs.
func ErrorIdentity(err error) (sameIdentity, sameMessage bool) {
	if err == nil {
		return false, false
	}
	return errors.Is(err, ErrStopped), err.Error() == ErrStopped.Error()
}

// InspectError exercises both a single unwrap path and errors.As through the
// custom As method.
func InspectError(err error) (stopped bool, stage string, coded bool, unwrapped error) {
	stopped = errors.Is(err, ErrStopped)
	var stageError *StageError
	if errors.As(err, &stageError) {
		stage = stageError.Stage
	}
	var codeError *CodeError
	coded = errors.As(err, &codeError)
	unwrapped = errors.Unwrap(err)
	return stopped, stage, coded, unwrapped
}

// CancellationState carries both the context error and its versioned cause.
type CancellationState struct {
	Err      error
	Cause    error
	Deadline time.Time
}

func NewCancellableContext(parent context.Context) (context.Context, context.CancelCauseFunc) {
	return context.WithCancelCause(parent)
}

func NewDeadlineContext(parent context.Context, deadline time.Time) (context.Context, context.CancelFunc) {
	return context.WithDeadlineCause(parent, deadline, ErrDeadline)
}

func CancellationStateOf(ctx context.Context) CancellationState {
	return CancellationState{Err: ctx.Err(), Cause: context.Cause(ctx), Deadline: deadlineOf(ctx)}
}

func deadlineOf(ctx context.Context) time.Time {
	deadline, _ := ctx.Deadline()
	return deadline
}

// NotifyCancellation invokes a callback after cancellation. Callback timing
// is intentionally a goroutine scheduling boundary.
func NotifyCancellation(ctx context.Context, callback func(error)) <-chan struct{} {
	done := make(chan struct{})
	go func() {
		defer close(done)
		<-ctx.Done()
		callback(context.Cause(ctx))
	}()
	return done
}

// LoseCancellation drops the parent's cancellation signal. The lost
// cancellation is a deliberate counterexample for callers to inspect.
func LoseCancellation(parent context.Context) context.Context {
	return context.WithoutCancel(parent)
}

func DetachCancellation(parent context.Context) context.Context {
	return context.WithoutCancel(parent)
}

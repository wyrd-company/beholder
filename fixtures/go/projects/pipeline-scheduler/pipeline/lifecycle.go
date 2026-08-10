package pipeline

import (
	"runtime"
	"sync"
	"sync/atomic"
)

// CallbackRegistry is a deterministic library callback boundary. It is
// separate from runtime finalizers and cleanups, whose timing is not promised.
type CallbackRegistry struct {
	mu        sync.Mutex
	callbacks []func(string)
	canceled  atomic.Bool
}

func (r *CallbackRegistry) Add(callback func(string)) {
	r.mu.Lock()
	defer r.mu.Unlock()
	r.callbacks = append(r.callbacks, callback)
}

func (r *CallbackRegistry) Dispatch(event string) {
	r.mu.Lock()
	callbacks := append([]func(string){}, r.callbacks...)
	r.mu.Unlock()
	if r.canceled.Load() {
		return
	}
	for _, callback := range callbacks {
		callback(event)
	}
}

func (r *CallbackRegistry) Cancel() {
	r.canceled.Store(true)
}

// FinalizableResource is released by a runtime finalizer only after ordinary
// users have stopped reaching it. No caller should rely on when that happens.
type FinalizableResource struct {
	ID       string
	callback func(string)
	once     sync.Once
}

func NewFinalizableResource(id string, callback func(string)) *FinalizableResource {
	resource := &FinalizableResource{ID: id, callback: callback}
	runtime.SetFinalizer(resource, finalizeResource)
	return resource
}

func finalizeResource(resource *FinalizableResource) {
	resource.release("finalizer")
}

func (r *FinalizableResource) release(kind string) {
	r.once.Do(func() {
		if r.callback != nil {
			r.callback(kind)
		}
	})
}

// UseFinalizableResource marks the finalizer target reachable through the
// last operation that needs it.
func UseFinalizableResource(resource *FinalizableResource) string {
	id := resource.ID
	runtime.KeepAlive(resource)
	return id
}

// CleanupResource uses the versioned runtime cleanup API. The cleanup argument
// is an independent string, so it does not retain the pointer being cleaned.
type CleanupResource struct {
	ID      string
	cleanup runtime.Cleanup
}

func NewCleanupResource(id string, callback func(string)) *CleanupResource {
	resource := &CleanupResource{ID: id}
	resource.cleanup = runtime.AddCleanup(resource, callback, id)
	return resource
}

// StopCleanup cancels a cleanup before it becomes eligible to run.
func StopCleanup(resource *CleanupResource) {
	resource.cleanup.Stop()
}

// DropResource makes the reachability boundary explicit after the last use.
func DropResource(resource *FinalizableResource) {
	_ = UseFinalizableResource(resource)
	resource = nil
}

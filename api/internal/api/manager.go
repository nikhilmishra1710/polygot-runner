package api

import (
	"context"
	"sync"
	"time"

	"github.com/google/uuid"
	pb "runtime-platform/api/gen/execution/v1"
)

type ExecutionManager struct {
	Store ExecutionStore

	mu      sync.Mutex
	cancels map[string]context.CancelFunc
	
	// Fan-out wakeup signals for multiple browser tabs
	listeners map[string][]chan struct{}
}

func NewExecutionManager(store ExecutionStore) *ExecutionManager {
	return &ExecutionManager{
		Store:     store,
		cancels:   make(map[string]context.CancelFunc),
		listeners: make(map[string][]chan struct{}),
	}
}

// ... Start() remains exactly the same, but remove the m.streams initialization ...

func (m *ExecutionManager) Start(ctx context.Context, req *pb.ExecuteRequest) (string, context.Context) {
	id := uuid.New().String()
	execCtx, cancel := context.WithCancel(ctx)

	m.mu.Lock()
	m.cancels[id] = cancel
	// Initialize the listener array
	m.listeners[id] = make([]chan struct{}, 0)
	m.mu.Unlock()

	exec := &Execution{
		ID:        id,
		State:     StateCreated,
		Request:   req,
		CreatedAt: time.Now(),
	}
	m.Store.Create(ctx, exec)

	return id, execCtx
}

// RouteEvent saves to DB, then wakes up all active WebSockets
func (m *ExecutionManager) RouteEvent(id string, event *pb.ExecutionEvent) {
	m.Store.AppendEvent(context.Background(), id, event)

	m.mu.Lock()
	defer m.mu.Unlock()
	
	// Broadcast a simple "new data available" signal to all listeners
	for _, ch := range m.listeners[id] {
		select {
		case ch <- struct{}{}:
		default:
		}
	}
}

// Subscribe returns a personal wakeup channel for a single WebSocket
func (m *ExecutionManager) Subscribe(id string) chan struct{} {
	m.mu.Lock()
	defer m.mu.Unlock()
	ch := make(chan struct{}, 1)
	m.listeners[id] = append(m.listeners[id], ch)
	return ch
}

// Unsubscribe cleans up the personal channel when the user closes the tab
func (m *ExecutionManager) Unsubscribe(id string, ch chan struct{}) {
	m.mu.Lock()
	defer m.mu.Unlock()
	
	listeners := m.listeners[id]
	for i, listener := range listeners {
		if listener == ch {
			// Remove the channel from the slice
			m.listeners[id] = append(listeners[:i], listeners[i+1:]...)
			close(ch)
			break
		}
	}
}

// ... Cancel() and Cleanup() remain the same, just clean up listeners instead of streams ...
func (m *ExecutionManager) Cleanup(id string) {
	m.mu.Lock()
	defer m.mu.Unlock()
	delete(m.cancels, id)
	
	if listeners, exists := m.listeners[id]; exists {
		for _, ch := range listeners {
			close(ch)
		}
		delete(m.listeners, id)
	}
}

// Cancel safely looks up and triggers the gRPC context cancellation for a specific job
func (m *ExecutionManager) Cancel(id string) {
	m.mu.Lock()
	defer m.mu.Unlock()
	
	if cancel, exists := m.cancels[id]; exists {
		cancel()
	}
}
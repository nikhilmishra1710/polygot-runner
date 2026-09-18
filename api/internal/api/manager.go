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

	// Transient state (cannot be stored in DB)
	mu      sync.Mutex
	cancels map[string]context.CancelFunc

	// Live event channels for active WebSockets
	streams map[string]chan *pb.ExecutionEvent
}

func NewExecutionManager(store ExecutionStore) *ExecutionManager {
	return &ExecutionManager{
		Store:   store,
		cancels: make(map[string]context.CancelFunc),
		streams: make(map[string]chan *pb.ExecutionEvent),
	}
}

// Start spawns the execution and saves it to the Store.
func (m *ExecutionManager) Start(ctx context.Context, req *pb.ExecuteRequest) (string, context.Context) {
	id := uuid.New().String()
	execCtx, cancel := context.WithCancel(ctx)

	m.mu.Lock()
	m.cancels[id] = cancel
	m.streams[id] = make(chan *pb.ExecutionEvent, 100)
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

// RouteEvent saves the event to the Store and broadcasts it to live WebSockets.
func (m *ExecutionManager) RouteEvent(id string, event *pb.ExecutionEvent) {
	m.Store.AppendEvent(context.Background(), id, event)

	m.mu.Lock()
	defer m.mu.Unlock()
	if ch, exists := m.streams[id]; exists {
		// Non-blocking send
		select {
		case ch <- event:
		default:
		}
	}
}

func (m *ExecutionManager) Subscribe(id string) chan *pb.ExecutionEvent {
	m.mu.Lock()
	defer m.mu.Unlock()
	return m.streams[id]
}

func (m *ExecutionManager) Cancel(id string) {
	m.mu.Lock()
	defer m.mu.Unlock()
	if cancel, exists := m.cancels[id]; exists {
		cancel()
		delete(m.cancels, id)
	}
	m.Store.UpdateState(context.Background(), id, StateCancelled, nil)
}

func (m *ExecutionManager) Cleanup(id string) {
	m.mu.Lock()
	defer m.mu.Unlock()
	delete(m.cancels, id)
	if ch, exists := m.streams[id]; exists {
		close(ch)
		delete(m.streams, id)
	}
}

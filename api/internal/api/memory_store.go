package api

import (
	"context"
	"errors"
	"sync"

	pb "runtime-platform/api/gen/execution/v1"
)

var ErrNotFound = errors.New("execution not found")

type InMemoryStore struct {
	mu         sync.RWMutex
	executions map[string]*Execution
	events     map[string][]*pb.ExecutionEvent
}

func NewInMemoryStore() *InMemoryStore {
	return &InMemoryStore{
		executions: make(map[string]*Execution),
		events:     make(map[string][]*pb.ExecutionEvent),
	}
}

func (s *InMemoryStore) Create(_ context.Context, exec *Execution) error {
	s.mu.Lock()
	defer s.mu.Unlock()
	s.executions[exec.ID] = exec
	s.events[exec.ID] = make([]*pb.ExecutionEvent, 0)
	return nil
}

func (s *InMemoryStore) Get(_ context.Context, id string) (*Execution, error) {
	s.mu.RLock()
	defer s.mu.RUnlock()
	exec, exists := s.executions[id]
	if !exists {
		return nil, ErrNotFound
	}
	return exec, nil
}

func (s *InMemoryStore) UpdateState(_ context.Context, id string, state JobState, result *pb.JobResult) error {
	s.mu.Lock()
	defer s.mu.Unlock()
	if exec, exists := s.executions[id]; exists {
		// STRICT STATE MACHINE: Terminal states are immutable!
		// Prevents a lagging goroutine from overwriting a fast cancellation.
		if exec.State == StateCompleted || exec.State == StateFailed || exec.State == StateCancelled {
			return nil
		}

		exec.State = state
		if result != nil {
			exec.Result = result
		}
		return nil
	}
	return ErrNotFound
}

func (s *InMemoryStore) AppendEvent(_ context.Context, id string, event *pb.ExecutionEvent) error {
	s.mu.Lock()
	defer s.mu.Unlock()
	if _, exists := s.events[id]; exists {
		s.events[id] = append(s.events[id], event)
		return nil
	}
	return ErrNotFound
}

func (s *InMemoryStore) GetEvents(_ context.Context, id string) ([]*pb.ExecutionEvent, error) {
	s.mu.RLock()
	defer s.mu.RUnlock()
	if events, exists := s.events[id]; exists {
		dst := make([]*pb.ExecutionEvent, len(events))
		copy(dst, events)
		return dst, nil
	}
	return nil, ErrNotFound
}

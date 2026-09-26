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

func (s *InMemoryStore) Get(_ context.Context, userID string, id string) (*Execution, error) {
	s.mu.RLock()
	defer s.mu.RUnlock()
	exec, exists := s.executions[id]
	if !exists || exec.UserID != userID {
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

func (s *InMemoryStore) List(_ context.Context, userID string) ([]*Execution, error) {
	s.mu.RLock()
	defer s.mu.RUnlock()

	var history []*Execution
	for _, exec := range s.executions {
		if exec.UserID == userID {
			history = append(history, exec)
		}
	}

	return history, nil
}

var (
	ErrUserNotFound  = errors.New("user not found")
	ErrEmailTaken    = errors.New("email already in use")
	ErrUserNameTaken = errors.New("user name already in use")
	ErrIDTaken       = errors.New("id already in use")
)

type InMemoryUserStore struct {
	mu    sync.RWMutex
	users map[string]*User
}

func NewInMemoryUserStore() *InMemoryUserStore {
	return &InMemoryUserStore{
		users: make(map[string]*User),
	}
}

func (us *InMemoryUserStore) Create(ctx context.Context, user *User) error {
	us.mu.Lock()
	defer us.mu.Unlock()
	if _, exists := us.users[user.ID]; exists {
		return ErrIDTaken
	}

	for _, existingUser := range us.users {
		if existingUser.UserName == user.UserName {
			return ErrUserNameTaken
		}
	}

	us.users[user.ID] = user
	return nil
}

func (us *InMemoryUserStore) GetById(ctx context.Context, id string) (*User, error) {
	us.mu.RLock()
	defer us.mu.RUnlock()
	if user, exists := us.users[id]; exists {
		return user, nil
	}
	return nil, ErrUserNotFound
}

func (us *InMemoryUserStore) GetByUserName(ctx context.Context, userName string) (*User, error) {
	us.mu.RLock()
	defer us.mu.RUnlock()
	for _, existingUser := range us.users {
		if existingUser.UserName == userName {
			return existingUser, nil
		}
	}
	return nil, ErrUserNotFound
}

var (
	ErrSessionNotFound = errors.New("session not found")
	ErrSessionExpired  = errors.New("session expired, please login again")
)

type InMemorySessionStore struct {
	mu       sync.RWMutex
	sessions map[string]*Session
}

func NewInMemorySessionStore() *InMemorySessionStore {
	return &InMemorySessionStore{
		sessions: make(map[string]*Session),
	}
}

func (ss *InMemorySessionStore) Create(ctx context.Context, session *Session) error {
	ss.mu.Lock()
	defer ss.mu.Unlock()

	ss.sessions[session.HashedSessionToken] = session
	return nil
}
func (ss *InMemorySessionStore) Get(ctx context.Context, hashedSessionToken string) (*Session, error) {
	ss.mu.RLock()
	defer ss.mu.RUnlock()

	if session, exists := ss.sessions[hashedSessionToken]; exists {
		return session, nil
	}

	return nil, ErrSessionNotFound
}
func (ss *InMemorySessionStore) Delete(ctx context.Context, hashedSessionToken string) error {
	ss.mu.Lock()
	defer ss.mu.Unlock()

	delete(ss.sessions, hashedSessionToken)
	return nil
}

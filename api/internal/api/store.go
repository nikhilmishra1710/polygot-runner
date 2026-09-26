package api

import (
	"context"
	"time"

	pb "runtime-platform/api/gen/execution/v1"
)

type JobState string

const (
	StateCreated   JobState = "CREATED"
	StateRunning   JobState = "RUNNING"
	StateCompleted JobState = "COMPLETED"
	StateFailed    JobState = "FAILED"
	StateCancelled JobState = "CANCELLED"
)

type User struct {
	ID             string
	UserName       string
	HashedPassword string
	CreatedAt      time.Time
	ModifiedAt     time.Time
}

type Session struct {
	ID                 string
	UserID             string
	HashedSessionToken string
	CreatedAt          time.Time
	ExpiresAt          time.Time
}

type Execution struct {
	ID        string
	UserID    string
	State     JobState
	Request   *pb.ExecuteRequest
	Result    *pb.JobResult
	CreatedAt time.Time
}

type UserStore interface {
	Create(ctx context.Context, user *User) error
	GetById(ctx context.Context, id string) (*User, error)
	GetByUserName(ctx context.Context, userName string) (*User, error)
}

type SessionStore interface {
	Create(ctx context.Context, session *Session) error
	Get(ctx context.Context, hashedSessionToken string) (*Session, error)
	Delete(ctx context.Context, hashedSessionToken string) error
}

// ExecutionStore abstracts the storage layer.
// Adding Redis later means writing a RedisStore struct that implements these 5 methods.
type ExecutionStore interface {
	Create(ctx context.Context, exec *Execution) error
	Get(ctx context.Context, userID string, id string) (*Execution, error)
	UpdateState(ctx context.Context, id string, state JobState, result *pb.JobResult) error
	AppendEvent(ctx context.Context, id string, event *pb.ExecutionEvent) error
	GetEvents(ctx context.Context, id string) ([]*pb.ExecutionEvent, error)
	List(ctx context.Context, userID string) ([]*Execution, error)
}

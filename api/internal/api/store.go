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

type Execution struct {
	ID        string
	State     JobState
	Request   *pb.ExecuteRequest
	Result    *pb.JobResult
	CreatedAt time.Time
}

// ExecutionStore abstracts the storage layer.
// Adding Redis later means writing a RedisStore struct that implements these 5 methods.
type ExecutionStore interface {
	Create(ctx context.Context, exec *Execution) error
	Get(ctx context.Context, id string) (*Execution, error)
	UpdateState(ctx context.Context, id string, state JobState, result *pb.JobResult) error
	AppendEvent(ctx context.Context, id string, event *pb.ExecutionEvent) error
	GetEvents(ctx context.Context, id string) ([]*pb.ExecutionEvent, error)
}

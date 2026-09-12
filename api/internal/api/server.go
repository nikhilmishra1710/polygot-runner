package api

import (
	"context"
	"errors"

	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"

	pb "runtime-platform/api/gen/execution/v1"
	"runtime-platform/api/internal/worker"
)

// Server represents the gRPC API server
type Server struct {
	pb.UnimplementedExecutionServiceServer
	workerClient WorkerClient
}

// WorkerClient defines the interface for communicating with the worker
type WorkerClient interface {
	Execute(ctx context.Context, req worker.ExecutionRequest) (worker.ExecutionResult, error)
	Close() error
}

// NewServer creates a new API server with the given worker client
func NewServer(wc WorkerClient) *Server {
	return &Server{
		workerClient: wc,
	}
}

// Execute implements the ExecutionService RPC.
func (s *Server) Execute(ctx context.Context, req *pb.ExecuteRequest) (*pb.ExecuteResponse, error) {
	// Validate request
	if err := ValidateExecutionRequest(req); err != nil {
		return nil, status.Error(codes.InvalidArgument, err.Error())
	}

	// Convert proto request to worker request
	wreq := ProtoRequestToWorker(req)

	// Execute via worker client (with ctx timeout already set by caller)
	wres, err := s.workerClient.Execute(ctx, wreq)
	if err != nil {
		// Map worker errors to gRPC codes
		if err == worker.ErrClientClosed {
			return nil, status.Error(codes.Unavailable, "worker client closed")
		}
		var unavailable worker.ErrWorkerUnavailable
		if errors.As(err, &unavailable) {
			return nil, status.Error(codes.Unavailable, unavailable.Error())
		}
		if _, ok := err.(worker.ErrWorkerError); ok {
			return nil, status.Error(codes.Internal, err.Error())
		}
		// Default to internal error
		return nil, status.Error(codes.Internal, err.Error())
	}

	// Convert worker result to proto response
	wresp := WorkerResultToProtoResponse(wres.ID, wres)
	return wresp, nil
}

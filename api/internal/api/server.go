package api

import (
	"context"
	"net/http"

	"runtime-platform/api/internal/worker"
)

// Server represents the HTTP API server
type Server struct {
	workerClient WorkerClient
	router       *http.ServeMux
}

// WorkerClient defines the interface for communicating with the worker
type WorkerClient interface {
	Execute(ctx context.Context, req worker.ExecutionRequest) (worker.ExecutionResult, error)
	Close() error
}

// NewServer creates a new API server with the given worker client
func NewServer(wc WorkerClient) *Server {
	s := &Server{
		workerClient: wc,
		router:       http.NewServeMux(),
	}
	s.setupRoutes()
	return s
}

// setupRoutes defines the HTTP endpoints
func (s *Server) setupRoutes() {
	s.router.HandleFunc("POST /v1/executions", s.handleExecutions)
}

// Router returns the HTTP handler mux
func (s *Server) Router() http.Handler {
	return s.router
}
package worker

import (
	"context"
	"fmt"
	"net"
)

// ExecutionContext represents the context for execution (simplified)
type ExecutionContext struct {
	context.Context
}

// ExecutionRequest represents a request to execute code
type ExecutionRequest struct {
	Language string
	Files    []File
	Stdin    string
}

// File represents a file in the execution request
type File struct {
	Path     string
	Contents string
}

// ExecutionResult represents the result of execution
type ExecutionResult struct {
	ID      string
	Status  string
	Stdout  string
	Stderr  string
	ExitCode int
}

// WorkerClient defines the interface for communicating with the worker
type WorkerClient interface {
	Execute(ctx context.Context, req ExecutionRequest) (ExecutionResult, error)
	Close() error
}

// UnixWorkerClient implements WorkerClient using Unix domain sockets
type UnixWorkerClient struct {
	conn   net.Conn
	addr   string
	closed bool
}

// NewUnixClient creates a new Unix domain socket worker client
func NewUnixClient(socketPath string) (*UnixWorkerClient, error) {
	// For now, we'll return a mock client since workerd isn't running yet
	// In a real implementation, this would dial the Unix socket
	return &UnixWorkerClient{
		addr: socketPath,
	}, nil
}

// Execute sends an execution request to the worker via Unix socket
func (c *UnixWorkerClient) Execute(ctx context.Context, req ExecutionRequest) (ExecutionResult, error) {
	if c.closed {
		return ExecutionResult{}, fmt.Errorf("client is closed")
	}

	// TODO: Implement actual Unix socket communication with workerd
	// For now, return a mock response to test the API layer
	return ExecutionResult{
		ID:      "execution-123",
		Status:  "success",
		Stdout:  "Hello\n",
		Stderr:  "",
		ExitCode: 0,
	}, nil
}

// Close closes the worker client connection
func (c *UnixWorkerClient) Close() error {
	if c.closed {
		return nil
	}
	c.closed = true
	if c.conn != nil {
		return c.conn.Close()
	}
	return nil
}